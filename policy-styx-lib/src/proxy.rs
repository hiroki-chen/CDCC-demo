use uuid::Uuid;
use wasmtime::Caller;

use crate::app::PcdRuntimeState;
use crate::types::{PcdWasmPtr, PcdWasmRawPtr};

// --------- Host Proxy Functions --------- //
/// This function serves as a host proxy for the `pcd_dataset_data_access` function.
/// It is called from the WASM module to access dataset data through the policy engine.
///
/// The input is the pointer to the UUID of the dataset data and the return is the
/// pointer to the data in the WASM memory of the caller if the access is allowed.
pub fn pcd_dataset_data_access_host(
    mut caller: Caller<'_, PcdRuntimeState>,
    data_ptr: PcdWasmRawPtr,
) -> PcdWasmRawPtr {
    let policy_engine = caller.data().policy_engine.clone();
    let policy_engine = policy_engine.read().unwrap();
    let data = PcdWasmPtr::from(data_ptr);

    match policy_engine.as_ref() {
        Some(engine) => {
            // Here you would typically call a method on the policy engine
            // to handle the dataset data access.
            println!("[Host] [Proxy] Accessing dataset data through policy engine");

            // Fetch the data uuid parameter from the caller's context

            // Read the caller's memory.
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
            let buffer = data
                .read(&memory, &mut caller)
                .expect("Failed to read memory for dataset data access");

            // Ensure this is valid.
            if let Err(_) = Uuid::from_slice(&buffer) {
                println!("[Host] [Proxy] Invalid UUID format in dataset data access");
                return 0; // Return error code for invalid UUID
            }

            // Write this to the policy engine for further processing.
            let memory = engine
                .instance
                .get_memory(&mut caller, "memory")
                .expect("Memory not found in policy engine");
            memory
                .write(&mut caller, data.ptr() as usize, &buffer)
                .expect("Failed to write memory");

            // Call the policy engine's function to check if we can access the data.

            0 // Return success code
        },
        None => {
            println!("[Host] [Proxy] Policy engine not initialized or missing");
            0 // Return error code
        },
    }
}

pub fn pcd_dataset_data_release_host(_: Caller<'_, PcdRuntimeState>) -> i32 {
    // This function serves as a placeholder for the host proxy.
    // Actual implementation will depend on the specific use case.
    println!("[Host] [Proxy] Host proxy function called");

    0
}
