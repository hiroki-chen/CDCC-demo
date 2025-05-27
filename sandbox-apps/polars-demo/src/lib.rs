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

mod consts;

use consts::*;

struct PcdRuntimeCtx {
    handle: i64,
}

static PCD_RUNTIME_CTX: OnceLock<PcdRuntimeCtx> = OnceLock::new();

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

fn build_charlson_expressions() -> Vec<Expr> {
    let mut charlson_conditions_exprs = Vec::new();

    for (condition_name, codes_map) in CHARLSON_CONDITIONS.iter() {
        let icd9_codes_iter = codes_map.get("9").unwrap().iter().map(|s| lit(s.as_str()));
        let icd10_codes_iter = codes_map.get("10").unwrap().iter().map(|s| lit(s.as_str()));

        // Polars doesn't have a direct equivalent of `in` on a list of expressions.
        // Instead, we use `is_in` with a literal list or use `any` with `eq`.
        // For efficiency, especially with `phf::Set`, converting to a `Vec<String>` first is good.
        let icd9_codes_vec = codes_map
            .get("9")
            .unwrap()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let icd10_codes_vec = codes_map
            .get("10")
            .unwrap()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let condition_expr = when(
            (col("vocabulary_id")
                .eq(lit("ICD9CM"))
                .and(col("diagnosis_code").is_in(lit(Series::new("", icd9_codes_vec))))) // Check ICD9 codes
            .or(col("vocabulary_id")
                .eq(lit("ICD10CM"))
                .and(col("diagnosis_code").is_in(lit(Series::new("", icd10_codes_vec))))), // Check ICD10 codes
        )
        .then(lit(1)) // If condition met, assign 1
        .otherwise(lit(0)) // Otherwise, assign 0
        .max() // Aggregate to get the maximum value (1 if any condition met, 0 otherwise)
        .alias(condition_name); // Alias the new column with the condition name

        charlson_conditions_exprs.push(condition_expr);
    }

    charlson_conditions_exprs
}

/// Loads a DataFrame from a byte vector representing an Apache Arrow IPC file.
fn load_table(tables: &HashMap<String, Vec<u8>>, table_name: &str) -> Result<DataFrame> {
    let data = tables
        .get(table_name)
        .ok_or_else(|| anyhow!("Missing {} table", table_name))?
        .clone(); // Clone is needed as Cursor takes ownership

    IpcReader::new(Cursor::new(data))
        .finish()
        .map_err(|e| anyhow!(e))
}

/// Generates the 'screening_events' DataFrame.
fn generate_screening_events(procedure_table: DataFrame) -> LazyFrame {
    procedure_table
        .lazy()
        .filter(
            // WHERE p.procedure_code IN ("45378", "45380", "45384", "45385")
            col("procedure_code")
                .cast(DataType::String)
                .is_in(lit(Series::new(
                    "",
                    vec![
                        "45378", "45380", "45384", "45385", // Colonoscopy codes
                    ],
                ))),
        )
        // GROUP BY p.patient_id
        .group_by(["patient_id"])
        .agg([
            col("start_datetime").min().alias("first_screening_date"),
            // CASE WHEN COUNT(*) > 0 THEN 1 ELSE 0 END as event
            when(len().gt(lit(0))) // Check if the group has more than 0 rows
                .then(lit(1i32)) // Return 1 (as i32)
                .otherwise(lit(0i32)) // Else return 0 (as i32)
                .alias("event"), // Assign the column name 'event'
        ])
}

/// Calculates the last visit date for each patient.
fn get_last_visits(encounter_table: DataFrame) -> LazyFrame {
    encounter_table
        .lazy()
        .group_by(["patient_id"])
        .agg([col("end_date").max().alias("last_visit_date")])
}
// This produces a duration; we need to fetch days from this.
fn date_sub(a: Expr, b: Expr) -> Expr {
    let stropt = StrptimeOptions {
        format: Some("%Y-%m-%d".to_string()),
        strict: false,
        exact: false,
        cache: false,
    };

    a.str()
        .to_date(stropt.clone())
        .sub(b.str().to_date(stropt))
        .dt()
        .total_days()
        .cast(DataType::Float64)
        .div(lit(365.25)) // Subtract 45
        .sub(lit(45.0))
        // Round to 2 decimal places
        .round(2)
}

/// Returns the expression for calculating 'T'.
fn get_t_expression() -> Expr {
    when(col("event").eq(lit(1)))
        .then(
            date_sub(col("first_screening_date"), col("birth_date")), // Divide by 365.25
        )
        .when(col("last_visit_date").is_not_null())
        .then(date_sub(col("last_visit_date"), col("birth_date")))
        .otherwise(date_sub(lit("2024-01-01"), col("birth_date")))
        .alias("T")
}

fn perform_imputation(df: &DataFrame, comorbidity_cols: &[&str]) -> Result<DataFrame> {
    let df = df.clone().lazy();
    // lengths don't match: unable to add a column of length 2 to a DataFrame of height 100 ?
    let encoding_expressions = CATEGORICAL_COLS
        .iter()
        .map(|col_name| {
            col(col_name)
                .cast(DataType::Categorical(None, CategoricalOrdering::Physical))
                .cast(DataType::UInt32)
        })
        .collect::<Vec<_>>();

    let mut df_encoded = df.with_columns(encoding_expressions);

    // Create the mask for patients without healthcare encounter (T == 0)
    // This creates a boolean Series (or conceptually, a boolean expression)
    let mask_expr = col("T").eq(lit(0.0));

    // Count patients matching the mask (equivalent to mask.sum() > 0)
    // We collect to check the count. In a real pipeline, you might not collect here.
    let num_patients_without_encounter = df_encoded
        .clone()
        .lazy()
        .filter(mask_expr.clone()) // Filter to get only the masked rows
        .select([len()]) // Count the rows
        .collect()?
        .get(0)
        .unwrap() // Get the 'count' Series
        .get(0) // Get the first (and only) value
        .and_then(|lv| lv.extract::<u32>()) // Extract as u32
        .unwrap_or(0); // Default to 0 if extraction fails

    if num_patients_without_encounter > 0 {
        println!(
            "Found {} patients without healthcare encounter.",
            num_patients_without_encounter
        );

        // 1. set all comorbidity variables to 0 for patients without healthcare encounter.
        let comorbidity_expressions = comorbidity_cols
            .iter()
            .map(|&col_name| {
                when(mask_expr.clone())
                    .then(lit(0i32))
                    .otherwise(col(col_name))
                    .alias(col_name)
            })
            .collect::<Vec<_>>();
        df_encoded = df_encoded.with_columns(comorbidity_expressions);

        // 2. keep event to 0.
        df_encoded = df_encoded.with_columns([when(mask_expr.clone())
            .then(lit(0i32))
            .otherwise(col("event"))
            .alias("event")]);
        // 3. set observation time.
        // For 'T' column: IF mask_expr THEN calculate_new_T ELSE original_T
        df_encoded = df_encoded.with_columns([when(mask_expr.clone())
            .then(date_sub(lit("2024-01-01"), col("birth_date")))
            .otherwise(col("T"))
            .alias("T")]);
    }

    df_encoded.collect().map_err(|e| anyhow!(e))
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
///
/// TODO: Do we also need to implement data imputation?
fn merge_healthcare_data(tables: &HashMap<String, Vec<u8>>) -> Result<DataFrame> {
    // 1. Load DataFrames
    let procedure_table = load_table(tables, "procedure_table")?;
    let encounter_table = load_table(tables, "encounter")?;
    let geolocation_table = load_table(tables, "geolocation")?;
    let travel_time_table = load_table(tables, "travel_time")?;
    let rucc_table = load_table(tables, "rucc")?;
    let diagnosis_table = load_table(tables, "diagnosis")?;
    let demographics_table = load_table(tables, "demographics")?;

    // Create 'rd' DataFrame for cross join
    let rd = df! {
        "end_date" => ["2024-01-01"]
    }?;

    // 2. Preprocessing steps
    let screening_events = generate_screening_events(procedure_table);
    let last_visits = get_last_visits(encounter_table);

    // 3. Perform Joins
    let mut combined_df_lazy = demographics_table
        .lazy()
        .left_join(screening_events, col("patient_id"), col("patient_id"))
        .left_join(last_visits, col("patient_id"), col("patient_id"))
        .left_join(
            geolocation_table.lazy(),
            col("patient_id"),
            col("patient_id"),
        )
        .left_join(
            travel_time_table.lazy(),
            col("census_block"),
            col("census_block"),
        )
        .left_join(rucc_table.lazy(), col("census_block"), col("census_block"))
        .left_join(diagnosis_table.lazy(), col("patient_id"), col("patient_id"))
        .cross_join(rd.lazy());

    // 4. Add derived columns (T and handle nulls for event)
    let t_expr = get_t_expression();
    combined_df_lazy =
        combined_df_lazy.with_columns([t_expr, col("event").fill_null(lit(0)).alias("event")]);

    // 5. Prepare aggregation expressions
    let mut charlson_agg_exprs = build_charlson_expressions();

    let common_group_by_cols = vec![
        "patient_id",
        "birth_date",
        "event",
        "T",
        "sex",
        "race",
        "ethnicity",
        "SDOH", // Assumes rucc_code is aliased to SDOH in selection if it exists
        "education",
        "income",
    ];

    let select_cols_before_group_by = vec![
        col("patient_id"),
        col("birth_date"),
        col("event"),
        col("T"),
        col("sex"),
        col("race"),
        col("ethnicity"),
        // Alias rucc_code to SDOH if it exists
        col("rucc_code").alias("SDOH"),
        col("education"),
        col("income"),
        col("travel_time_minutes"),
        col("vocabulary_id"),
        col("diagnosis_code"),
    ];

    // Add min_travel_time aggregation
    charlson_agg_exprs.insert(
        0,
        (min("travel_time_minutes") / lit(60.0)).alias("min_travel_time"),
    );

    // 6. Select, Group By, Aggregate, and Sort
    let final_df = combined_df_lazy
        .select(select_cols_before_group_by)
        .group_by(common_group_by_cols)
        .agg(charlson_agg_exprs)
        .sort(["patient_id"], SortMultipleOptions::default());

    println!("plan: {}", final_df.explain(true)?);

    // 7. Collect the result
    final_df
        .set_policy_checking(false) // set to false for debugging
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
    let combined_data = combined_data
        .collect()?
        .columns_to_dummies(CATEGORICAL_COLS.into(), None, false)?
        .lazy();

    let min_travel_time_expr = ((col("min_travel_time") - col("min_travel_time").mean())
        / col("min_travel_time").std(1))
    .alias("min_travel_time");
    let travel_time_squared_expr = col("min_travel_time").pow(2).alias("travel_time_squared");
    let combined_data =
        combined_data.with_columns(&[min_travel_time_expr, travel_time_squared_expr]);

    let schema = combined_data.schema()?;
    let demo_covariates = schema
        .iter_names()
        .filter_map(|c| {
            if CATEGORICAL_COLS.iter().any(|e| c.starts_with(e)) {
                Some(c.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    //  all_covariates = travel_covariates + demo_covariates + charlson_covariates + sdoh_covariates
    let mut all_covariates = [
        TRAVEL_COVARATES,
        demo_covariates.as_slice(),
        CHARLSON_COVARIATES,
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
    fn test_run_cox_analysis() {
        let tables = get_tables();
        // This test assumes that the merge_healthcare_data function works correctly.
        // In a real-world scenario, you would mock the data or use a test dataset.
        // Here we just run the function to ensure it does not panic.
        // Note: The actual Cox analysis is not implemented in this demo.
        // We will just check if the function runs without errors.
        let merged_data = merge_healthcare_data(&tables).expect("Failed to merge data");

        assert_eq!(merged_data.shape(), (100, 36), "Shape mismatch!");

        let comorbidity_cols = CHARLSON_COVARIATES.iter().map(|&s| s).collect::<Vec<_>>();
        let encoded_data = perform_imputation(&merged_data, &comorbidity_cols).unwrap();

        // Run the Cox analysis with the merged data.
        run_cox_analysis_with_privacy(merged_data).expect("Failed to run Cox analysis");
    }
}
