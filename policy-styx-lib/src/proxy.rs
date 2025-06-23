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

    println!(
        "[Host] [Proxy] Accessing dataset data with pointer: {:?}",
        data
    );

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
            let eval_func = engine
                .instance
                .get_typed_func::<PcdWasmRawPtr, PcdWasmRawPtr>(
                    &mut caller,
                    "pcd_dataset_data_access",
                )
                .expect("Function eval_input not found in policy engine");

            let data_ptr = eval_func
                .call(&mut caller, data.into())
                .expect("Failed to call eval_input function in policy engine");

            if data_ptr == 0 {
                println!("[Host] [Proxy] Access not granted for dataset data");
                // Return the error code or handle the denial appropriately
                return 0;
            } else {
                println!("[Host] [Proxy] Access granted for dataset data");
                // Perform the proxy copying.

                // Read the data_ptr.
                let data_ptr = PcdWasmPtr::from(data_ptr);
                let data_buffer = data_ptr
                    .read(&memory, &mut caller)
                    .expect("Failed to read memory for dataset data access");

                data_ptr
                    .dealloc(&mut caller, &engine.instance)
                    .expect("Failed to deallocate memory for dataset data access");

                // Write the data back to the caller's memory.
                let allocate_fn = caller
                    .get_export("allocate")
                    .unwrap()
                    .into_func()
                    .unwrap()
                    .typed::<u32, u32>(&mut caller)
                    .unwrap();
                let data_ptr = allocate_fn
                    .call(&mut caller, data_buffer.len() as u32)
                    .expect("Failed to allocate memory for dataset data access");

                println!(
                    "[Host] [Proxy] Allocated memory for dataset data access at pointer: {:x}",
                    data_ptr
                );

                let mut memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                let data_ptr = PcdWasmPtr::new(data_ptr, data_buffer.len() as u32);
                data_ptr
                    .write(&mut memory, &data_buffer, &mut caller)
                    .expect("Failed to write memory for dataset data access");

                data_ptr.into()
            }
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
