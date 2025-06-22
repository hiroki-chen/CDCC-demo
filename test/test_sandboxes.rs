use std::collections::HashMap;

use policy_styx_lib::app::PcdWasmRuntimeBuilder;
use policy_styx_lib::proxy::pcd_dataset_data_access_host;
use policy_styx_lib::types::PcdWasmPtr;
use uuid::Uuid;
use wasi_common::sync::WasiCtxBuilder;

#[test]
fn test_sandbox_app() {
    // 1. Configure all host functions and build the runtime
    let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();
    let mut rt = PcdWasmRuntimeBuilder::new()
        .unwrap()
        .with_host_function(
            "pcd_dataset_data_access",
            pcd_dataset_data_access_host, // Your original host function
        )
        .unwrap()
        .build(wasi_ctx);

    // 2. Now use the fully-configured runtime.
    // The linker inside `rt` is now complete.
    let session_id = Uuid::new_v4();

    rt.load_policy_engine("../data/policy_engine.wasm").unwrap();
    let app = rt.load_new_application("../data/cox_demo.wasm").unwrap();

    // The rest of your test remains the same...
    let data = bincode::deserialize_from(std::fs::File::open("../data/data.enc").unwrap()).unwrap();
    rt.pcd_dataset_add_data(&session_id, data).unwrap();

    let args = HashMap::from([(
        "data_uuid".to_string(),
        "123e4567-e89b-12d3-a456-426614174000".to_string(),
    )]);
    let args = rt
        .write_memory(Some(app), &bincode::serialize(&args).unwrap())
        .unwrap();

    let ptr = rt
        .write_memory(Some(app), &bincode::serialize(&session_id).unwrap())
        .unwrap();
    rt.execute_typed_function::<PcdWasmPtr, ()>(app, "set_session_id", ptr)
        .unwrap();
    let ret = rt
        .execute_typed_function::<PcdWasmPtr, PcdWasmPtr>(app, "entry", args)
        .unwrap();
}
