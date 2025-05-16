#![allow(unused)]

use std::collections::HashMap;
use std::sync::OnceLock;

use anyhow::{anyhow, Result};
use polars::prelude::*;
use uuid::Uuid;

struct PcdRuntimeCtx {
    handle: i64,
}

static PCD_RUNTIME_CTX: OnceLock<PcdRuntimeCtx> = OnceLock::new();

/// The columns we are interested in.
const CATEGORICAL_COLS: &[&str] = &["sex", "race", "ethnicity", "education"];
/// Charlson covariates for comorbidity.
const CHARLSON_COVARIATES: &[&str] = &[
    "mi", "chf", "pvd", "cevd", "dementia", "copd", "rheumd", "pud", "mld", "msld", "diab",
    "dia_w_c", "hp", "mrend", "srend", "aids", "hiv", "mst", "mal", "Obesity", "WL", "Alcohol",
    "Drug", "Psycho", "Dep",
];
const TRAVEL_COVARATES: &[&str] = &["travel_time", "travel_time_squared"];

extern "C" {
    // Get the target data.
    fn pcd_dataset_access(ctx: i64, data_uuid: *const u8, buf: *mut u8, buf_len: u32) -> i32;
    // Release the given data.
    fn pcd_dataset_release(ctx: i64, data_uuid: *const u8) -> i32;
    fn pcd_dataset_add_data(
        ctx: i64,
        input_data: *const u8,
        input_data_len: u32,
        data_uuid: *const u8,
    ) -> i32;
}

#[no_mangle]
pub unsafe extern "C" fn polars_demo(ctx: i64) -> i32 {
    println!("Registering runtime handle {ctx}!");
    let runtime = PcdRuntimeCtx { handle: ctx };

    if let Err(_) = PCD_RUNTIME_CTX.set(runtime) {
        eprintln!("Failed to set the runtime context! Set twice.");
        // We ignore the error here.
    }

    let uuid = Uuid::new_v4();
    let uuid_ptr = uuid.as_bytes().as_ptr();
    let mut buf = [0u8; 1024];
    let ret = pcd_dataset_access(ctx, uuid_ptr, buf.as_mut_ptr(), 1024);

    println!("pcd_dataset_access: {ret}");

    0
}

/// Drop NaNs from the given lazy frame.
///
/// The forked version of polars is old so we manually implement this.
fn drop_nans(lf: LazyFrame, subset: Option<Vec<Expr>>) -> LazyFrame {
    if let Some(subset) = subset {
        lf.filter(
            all_horizontal(
                subset
                    .into_iter()
                    .map(|v| v.is_not_nan())
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        )
    } else {
        lf.filter(
            all_horizontal([dtype_cols([DataType::Float32, DataType::Float64]).is_not_nan()])
                .unwrap(),
        )
    }
}

/// Replace the given data.
fn replace(lf: LazyFrame) -> LazyFrame {
    todo!()
}

/// Merge healthcare data.
///
/// The input is a collection of dataframes, each representing a table.
fn merge_healthcare_data(tables: &HashMap<String, DataFrame>) -> Result<DataFrame> {
    // We first generate the screening events
    let procedure_table = tables
        .get("procedure")
        .ok_or(anyhow!("Missing procedure table"))?
        .clone();
    let encouter_table = tables
        .get("encounter")
        .ok_or(anyhow!("Missing encounter table"))?
        .clone();
    let geolocation_table = tables
        .get("geolocation")
        .ok_or(anyhow!("Missing geolocation table"))?
        .clone();
    let travel_time_table = tables
        .get("travel_time")
        .ok_or(anyhow!("Missing travel time table"))?
        .clone();
    let rucc_table = tables
        .get("rucc")
        .ok_or(anyhow!("Missing rucc table"))?
        .clone();
    let diagnosis_table = tables
        .get("diagnosis")
        .ok_or(anyhow!("Missing diagnosis table"))?
        .clone();
    let demographics_table = tables
        .get("demographics")
        .ok_or(anyhow!("Missing demographics table"))?
        .clone();

    let screening_events = procedure_table
        .lazy()
        // GROUP BY p.patient_id
        .group_by(["patient_id"])
        .agg([
            col("start_datetime").min().alias("first_screening_date"),
            // CASE WHEN COUNT(*) > 0 THEN 1 ELSE 0 END as event
            when(col("*").count().gt(lit(0)))
                .then(lit(1))
                .otherwise(lit(0))
                .alias("event"),
        ])
        .filter(
            // WHERE p.procedure_code IN ("45378", "45380", "45384", "45385")
            (col("procedure_code").eq(lit("45378")))
                .or(col("procedure_code").eq(lit("45380")))
                .or(col("procedure_code").eq(lit("45384")))
                .or(col("procedure_code").eq(lit("45385"))),
        );

    let last_visits = encouter_table
        .lazy()
        .group_by(["patient_id"])
        .agg([col("end_date").max().alias("last_visit_date")]);

    let combined_df = demographics_table
        .lazy()
        .join(
            screening_events.lazy(),
            [col("patient_id")],
            [col("patient_id")],
            JoinArgs::default(),
        )
        .join(
            last_visits.lazy(),
            [col("patient_id")],
            [col("patient_id")],
            JoinArgs::default(),
        )
        .join(
            geolocation_table.lazy(),
            [col("patient_id")],
            [col("patient_id")],
            JoinArgs::default(),
        )
        .join(
            travel_time_table.lazy(),
            [col("census_block")],
            [col("census_block")],
            JoinArgs::default(),
        )
        .join(
            rucc_table.lazy(),
            [col("census_block")],
            [col("census_block")],
            JoinArgs::default(),
        )
        .join(
            diagnosis_table.lazy(),
            [col("patient_id")],
            [col("patient_id")],
            JoinArgs::default(),
        );

    combined_df
        .set_policy_checking(true)
        .collect()
        .map_err(|e| anyhow!(e))
}

/// Run Cox analyais with optional privacy enforcement.
///
/// The result would be the fitted CoxPHFitter model and some a dictionary of exlucded columns.
fn run_cox_analysis_with_privacy(combined_data: DataFrame) -> Result<()> {
    let combined_data = combined_data.lazy();

    // Drop NaNs. Since the version we used do not yet support this method,
    // we use a workaround.
    //
    // Delete rows with missing survival time or event.
    let combined_data = drop_nans(combined_data, Some([col("T"), col("event")].into()));

    // We then convert categorical variables into dummy encodings (0/1 variables or indicator values).
    // In polars, we use `to_dummies` method on `Series`.
    let combined_data = combined_data.collect()?.to_dummies(None, false)?.lazy();

    let min_travel_time_expr = ((col("min_travel_time") - col("min_travel_time").mean())
        / col("min_travel_time").std(2))
    .alias("min_travel_time");
    let travel_time_squared_expr = col("min_travel_time").pow(2).alias("travel_time_squared");

    let combined_data =
        combined_data.with_columns(&[min_travel_time_expr, travel_time_squared_expr]);

    let schema = combined_data.schema()?;
    let demo_covariates = schema
        .iter_names()
        .filter_map(|c| {
            if c.starts_with("sex")
                || c.starts_with("race")
                || c.starts_with("ethnicity")
                || c.starts_with("education")
            {
                Some(c.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    //  all_covariates = travel_covariates + demo_covariates + charlson_covariates + sdoh_covariates
    let mut all_covariates = [
        CATEGORICAL_COLS,
        demo_covariates.as_slice(),
        CHARLSON_COVARIATES,
        TRAVEL_COVARATES,
    ]
    .into_iter()
    .flat_map(|e| e.to_vec())
    .collect::<Vec<_>>();
    all_covariates.push("SDOH");

    let cox_data = combined_data.select(
        [all_covariates, vec!["T", "event"]]
            .into_iter()
            .flat_map(|e| e.to_vec())
            .map(|e| col(e))
            .collect::<Vec<_>>(),
    );

    // Ensure no infinite values or NaN.

    let cox_data = drop_nans(cox_data, None);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_merge_healthcare_data() {
        // merge_healthcare_data();
    }
}
