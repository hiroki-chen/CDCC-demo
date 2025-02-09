use policy_styx_lib::app::PcdWasmRuntime;
use policy_styx_lib::dataset;
use wasi_common::sync::WasiCtxBuilder;
use wasmtime::{Result, Val};

const POLARS_ENTRY: &str = "polars_demo";
const POLICY_ENGINE: &str = "../wasm_apps/policy_styx.wasm";
const POLARS_APP: &str = "../wasm_apps/polars_demo.wasm";

fn main() -> Result<()> {
    let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();
    let mut runtime = PcdWasmRuntime::new(wasi_ctx)?;

    runtime.register_native_functions("pcd_dataset_access", dataset::pcd_dataset_access)?;
    runtime.register_native_functions("pcd_dataset_release", dataset::pcd_dataset_release)?;
    runtime.register_native_functions("pcd_dataset_add_data", dataset::pcd_dataset_add_data)?;
    runtime.load_policy_engine(POLICY_ENGINE)?;

    let idx = runtime.load_new_application(POLARS_APP)?;

    let runtime_ptr = Val::I64(runtime.as_mut_ptr() as i64);
    let return_value = Val::I32(0);
    runtime.execute_function(idx, POLARS_ENTRY, &[runtime_ptr], &mut [return_value])?;

    Ok(())
}
