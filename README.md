# An End-to-End Policy Framework

- `policy-styx-sys`: Rust FFI bindings to Styx' framework APIs such as policy encoding, decoding, data unpacking, etc.
- `policy-styx-rt`: A set of wrapper APIs around the native `WAMR` runtime toolkit for hosting apps in `sandbox-apps`.
- `policy-styx-runner`: A launcher that runs the given application in WASM format.
- `sandbox-apps`: Applications that live inside the WASM sandbox hosted by WASM Micro Runtime.
  - `policy-styx`: The styx policy engine.
  - `polars-demo`: A demo that uses polars + picachv
  - `medical-analysis`: An algorithm that implements cox analysis over patient data.


We use wasmtime as the runtime.



Mock dataset is put under `data` directory.

## Workflow of the middleware (a.k.a., the runtime)

1. Application calls `ecall_init_env`
2. `ecall_init_env` calls `pcd_runtime_setup_environment` which calls
  - `wasm_runtime_init`
  - `wasm_runtime_register_natives`?
3. `register_native_symbols` that calls `pcd_wamr_register_native_symbol`
4. 


- Data Producer: the the entity who produces the policy carrying data.
  The layout would be:
  - Metadata: data id;
  - Key delegator address (which we can ignore here)
  - Crypto info (how this data gets encrypted?)
  - encrypted payload of
    + raw data
    + policy
    + attributes

- The algorithm:
  - Database file, including the seven tables laid out in the slides;
  - Step 1 implementation: Jing suggested to implement the SQL code that merges 7 tables into a single large table using sqlite in python. Here the implementation use a python program processing a sql script to merge the tables. To execute the code, you need to pre-install: python, sqlite package and sqlite3 library in python (https://docs.python.org/3/library/sqlite3.html). We need to assume these components as well as the python code to process sql script are all trustworthy. Then you can use pikachu to verify the sql script, which is the only part implemented by the user in our application. The output of the sql code here is in parquet format, which can be loaded into python directly as data frame.
  - Step 2 implementation: we will implement python code to load the output from step 2 (in parquet format), and run Cox survival analysis. It will call a package for cox analysis and output a general output (single line output with several variables, including the p-value). According to our discussion, you will need to re-implement it using Rust. I checked Rust has a library to load parquet format, so it should be straightforward. Rust also has library for Cos analysis – can you just use it assuming it is trustworthy? If so, the verification only needs to be check if the input to the function comply the policy: when it involves sensitive data, it should aggregate enough (>=20) individuals, i.e. the number of rows >= 20.