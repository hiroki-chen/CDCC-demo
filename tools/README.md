# Tools

This directory contains utility tools for the CDCC demo project.

## pack-cox-data

Packs Apache Arrow healthcare tables into the PcdPackedData format required by the Cox Proportional Hazards analysis program.

### Required Input Files

The tool expects these 7 Arrow tables in the `./data/` directory:

- `demographics.arrow` - Patient demographic information
- `procedure_table.arrow` - Medical procedures performed
- `encounter.arrow` - Healthcare encounter records
- `geolocation.arrow` - Patient geographic locations
- `travel_time.arrow` - Travel time to healthcare facilities
- `rucc.arrow` - Rural-Urban Continuum Codes
- `diagnosis.arrow` - Diagnosis codes (ICD-9/ICD-10)

### Usage

```bash
# Build and run
cargo run --manifest-path tools/policy-data-generator/Cargo.toml --bin pack-cox-data -- ./data/cox-input.bin

# Or build release version first
cargo build --manifest-path tools/policy-data-generator/Cargo.toml --bin pack-cox-data --release
./tools/policy-data-generator/target/release/pack-cox-data ./data/cox-input.bin
```

### Output

Creates a binary file containing:
- `HashMap<String, Vec<u8>>` where each entry is (table_name, arrow_ipc_bytes)
- Serialized with bincode

### Next Steps

After generating the packed data file:

1. Upload it as the "Data" file in the web interface
2. Upload `polars_demo.wasm` as the "Program"
3. Upload `policy_engine.wasm` as the "Policy Engine"
4. Click "Start Secure Computation"

The Cox analysis will:
- Merge all 7 tables
- Perform data imputation
- Run Cox Proportional Hazards regression
- Output results with full logging

## policy-data-generator

Generates mock healthcare data for testing (see source code for details).

### Usage

```bash
cargo run --manifest-path tools/policy-data-generator/Cargo.toml --bin policy-data-generator
