#![allow(unused)]

use std::collections::HashMap;
use std::io::Cursor;
use std::ops::{Div, Sub};
use std::sync::OnceLock;

use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use polars::io::mmap::MmapBytesReader;
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
/// The input is a collection of raw Apache Arrow files (in-memory).
fn merge_healthcare_data(tables: &HashMap<String, Vec<u8>>) -> Result<DataFrame> {
    // We first generate the screening events
    let procedure_table = IpcReader::new(Cursor::new(
        tables
            .get("procedure_table")
            .ok_or(anyhow!("Missing procedure table"))?
            .clone(),
    ))
    .finish()?;

    let encouter_table = IpcReader::new(Cursor::new(
        tables
            .get("encounter")
            .ok_or(anyhow!("Missing encounter table"))?
            .clone(),
    ))
    .finish()?;

    let geolocation_table = IpcReader::new(Cursor::new(
        tables
            .get("geolocation")
            .ok_or(anyhow!("Missing geolocation table"))?
            .clone(),
    ))
    .finish()?;

    let travel_time_table = IpcReader::new(Cursor::new(
        tables
            .get("travel_time")
            .ok_or(anyhow!("Missing travel time table"))?
            .clone(),
    ))
    .finish()?;

    let rucc_table = IpcReader::new(Cursor::new(
        tables
            .get("rucc")
            .ok_or(anyhow!("Missing rucc table"))?
            .clone(),
    ))
    .finish()?;

    let diagnosis_table = IpcReader::new(Cursor::new(
        tables
            .get("diagnosis")
            .ok_or(anyhow!("Missing diagnosis table"))?
            .clone(),
    ))
    .finish()?;

    let demographics_table = IpcReader::new(Cursor::new(
        tables
            .get("demographics")
            .ok_or(anyhow!("Missing demographics table"))?
            .clone(),
    ))
    .finish()?;

    // duplicate: column with name 'event' has more than one occurrences: 'group_by' failed: 'filter' input failed to resolve
    let screening_events = procedure_table
        .lazy()
        .filter(
            // WHERE p.procedure_code IN ("45378", "45380", "45384", "45385")
            (col("procedure_code").eq(lit("45378")))
                .or(col("procedure_code").eq(lit("45380")))
                .or(col("procedure_code").eq(lit("45384")))
                .or(col("procedure_code").eq(lit("45385"))),
        )
        // GROUP BY p.patient_id
        .group_by(["patient_id"])
        .agg([
            col("start_datetime").min().alias("first_screening_date"),
            // CASE WHEN COUNT(*) > 0 THEN 1 ELSE 0 END as event
            // Use `len()` for COUNT(*) equivalent within an aggregation
            when(len().gt(lit(0))) // Check if the group has more than 0 rows
                .then(lit(1i32)) // Return 1 (as i32)
                .otherwise(lit(0i32)) // Else return 0 (as i32)
                .alias("event"), // Assign the column name 'event'
        ]);

    println!(
        "screening_events: {:?}",
        screening_events.clone().collect()?
    );

    let last_visits = encouter_table
        .lazy()
        .group_by(["patient_id"])
        .agg([col("end_date").max().alias("last_visit_date")]);

    println!("last_visits: {:?}", last_visits.clone().collect()?);

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

    let t_expr = get_t();

    let combined_df = combined_df
        .select([
            col("patient_id"),
            col("birth_date"),
            t_expr,
            col("sex"),
            col("race"),
            col("ethnicity"),
            col("rucc_code").alias("SDOH"),
            col("education"),
            col("income"),
        ])
        .group_by(["patient_id"])
        .agg([min("travel_time_minutes") / lit(60.0).alias("min_travel_time")]);

    combined_df
        .set_policy_checking(false) // set to false for debugging
        .collect()
        .map_err(|e| anyhow!(e))
}

pub fn get_t() -> Expr {
    when(col("event").eq(lit(1)))
        .then(
            (col("first_screening_date") - col("birth_date")) // This results in a Duration
                .cast(DataType::Float64) // Cast Duration to a float to perform division
                .div(lit(365.25f64)) // Divide by 365 days
                .sub(lit(45.0))
                .round(2),
        )
        .when(col("last_visit_date").is_not_null())
        .then(
            (col("last_visit_date") - col("birth_date")) // This results in a Duration
                .cast(DataType::Float64) // Cast Duration to a float to perform division
                .div(lit(365.25f64)) // Divide by 365 days
                .sub(lit(45.0))
                .round(2),
        )
        .otherwise(
            (lit(
                // Assuming '2024-01-01' is a Date
                polars::series::Series::new("const_date", &[NaiveDate::from_ymd_opt(2024, 1, 1)])
                    .cast(&DataType::Date)
                    .unwrap(), // Ensure it's a Date type
            ) - col("birth_date")) // This results in a Duration
            .cast(DataType::Float64) // Cast Duration to a float to perform division
            .div(lit(365.25f64)) // Divide by 365 days
            .sub(lit(45.0))
            .round(2),
        )
        .alias("T")
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

    // Invoke the CoxPHFitter.

    Ok(())
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;

    const TEST_DATA_PATH: &'static str = "../../data";
    const TABLES: &[&str] = &[
        "procedure_table",
        "encounter",
        "geolocation",
        "travel_time",
        "rucc",
        "diagnosis",
        "demographics",
    ];

    fn get_tables() -> HashMap<String, Vec<u8>> {
        TABLES
            .iter()
            .map(|table| {
                let path = format!("{TEST_DATA_PATH}/{table}.arrow");
                let bytes = std::fs::read(path).expect("Failed to read file");

                (table.to_string(), bytes)
            })
            .collect()
    }

    #[test]
    fn test_merge_healthcare_data() {
        let tables = get_tables();
        let merged_data = merge_healthcare_data(&tables).expect("Failed to merge data");
        assert!(!merged_data.is_empty(), "Merged data should not be empty");
    }

    #[test]
    fn test_run_cox_analysis() {
        let tables = get_tables();
        // This test assumes that the merge_healthcare_data function works correctly.
        // In a real-world scenario, you would mock the data or use a test dataset.
        // Here we just run the function to ensure it does not panic.
        // Note: The actual Cox analysis is not implemented in this demo.
        // We will just check if the function runs without errors.
        let merged_data = merge_healthcare_data(&tables).expect("Failed to merge data");
        // Run the Cox analysis with the merged data.
        run_cox_analysis_with_privacy(merged_data).expect("Failed to run Cox analysis");
    }
}
