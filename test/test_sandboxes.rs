use std::collections::HashMap;

use policy_styx_lib::app::PcdWasmRuntimeBuilder;
use policy_styx_lib::proxy::pcd_dataset_data_access_host;
use policy_styx_lib::types::PcdWasmRawPtr;
use uuid::Uuid;
use wasi_common::sync::WasiCtxBuilder;

#[cfg(test)]
#[test]
fn test_sandbox_app() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    let mock_session_id = Uuid::new_v4();

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

    let key = std::fs::read("../data/key").unwrap();
    rt.load_predefined_session(mock_session_id, &key)
        .expect("Failed to load predefined session");

    // 2. Now use the fully-configured runtime.
    // The linker inside `rt` is now complete.
    rt.load_policy_engine("../data/policy_engine.wasm").unwrap();
    let app = rt.load_new_application("../data/polars_demo.wasm").unwrap();

    // The rest of your test remains the same...
    let data = bincode::deserialize_from(std::fs::File::open("../data/data.enc").unwrap()).unwrap();
    let data_uuid = rt.pcd_dataset_add_data(&mock_session_id, data).unwrap();

    let args = HashMap::from([("data_uuid".to_string(), data_uuid.as_bytes().to_vec())]);
    let args = rt
        .write_memory(Some(app), &bincode::serialize(&args).unwrap())
        .unwrap();

    let ptr = rt
        .write_memory(Some(app), mock_session_id.as_bytes())
        .unwrap();
    rt.execute_typed_function::<PcdWasmRawPtr, ()>(app, "set_session_id", ptr.into())
        .unwrap();
    let ret = rt
        .execute_typed_function::<PcdWasmRawPtr, PcdWasmRawPtr>(app, "entry", args.into())
        .unwrap();
}
