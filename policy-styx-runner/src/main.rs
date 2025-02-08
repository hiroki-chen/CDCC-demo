use policy_styx_lib::app::PcdWasmRuntime;
use wasi_common::sync::WasiCtxBuilder;
use wasmtime::{Result, Val};

const POLARS_ENTRY: &str = "polars_demo";
const POLICY_ENGINE: &str = "../wasm_apps/policy_styx.wasm";
const POLARS_APP: &str = "../wasm_apps/polars_demo.wasm";

fn main() -> Result<()> {
    let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();
    let mut runtime = PcdWasmRuntime::new(wasi_ctx)?;

    runtime.register_native_functions("pcd_data_access", |ptr: i32| -> i64 {
        println!("pcd_data_access called with ptr: {}", ptr);
        0
    })?;

    runtime.register_native_functions("pcd_data_release", |ptr: i32| -> i32 {
        println!("pcd_data_release called with ptr: {}", ptr);
        0
    })?;
    runtime.load_policy_engine(POLICY_ENGINE, 4096..8192)?;
    runtime.load_new_application("polars", POLARS_APP, 4096..8192)?;

    let runtime_ptr = Val::I64(runtime.as_mut_ptr() as i64);
    let return_value = Val::I32(0);
    runtime.execute_function("polars", POLARS_ENTRY, &[runtime_ptr], &mut [return_value])?;

    Ok(())
}
