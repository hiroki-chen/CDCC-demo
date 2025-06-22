use uuid::Uuid;
use wasmtime::Caller;

use crate::app::PcdRuntimeState;
use crate::types::PcdWasmPtr;

// --------- Host Proxy Functions --------- //
pub fn pcd_dataset_data_access_host(
    mut caller: Caller<'_, PcdRuntimeState>,
    data: PcdWasmPtr,
) -> i32 {
    let policy_engine = caller.data().policy_engine.clone();
    let policy_engine = policy_engine.read().unwrap();

    match policy_engine.as_ref() {
        Some(engine) => {
            // Here you would typically call a method on the policy engine
            // to handle the dataset data access.
            println!("[Host] [Proxy] Accessing dataset data through policy engine");

            // Fetch the data uuid parameter from the caller's context
            let data_uuid = data >> 32; // Extract the UUID part from the PcdWasmPtr
            let data_uuid_len = (data & 0xFFFFFFFF) as usize; // Extract the length part

            // Read the caller's memory.
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

            let mut buffer = vec![0u8; data_uuid_len];
            memory
                .read(&caller, data_uuid as usize, &mut buffer)
                .expect("Failed to read memory");

            // Ensure this is valid.
            if let Err(_) = Uuid::from_slice(&buffer) {
                println!("[Host] [Proxy] Invalid UUID format in dataset data access");
                return -1; // Return error code for invalid UUID
            }

            // Write this to the policy engine for further processing.
            let memory = engine
                .instance
                .get_memory(&mut caller, "memory")
                .expect("Memory not found in policy engine");
            memory
                .write(&mut caller, data_uuid as usize, &buffer)
                .expect("Failed to write memory");

            // Call the policy engine's function to check if we can access the data.
            

            0 // Return success code
        },
        None => {
            println!("[Host] [Proxy] Policy engine not initialized or missing");
            -1 // Return error code
        },
    }
}

pub fn pcd_dataset_data_release_host(caller: Caller<'_, PcdRuntimeState>) -> i32 {
    // This function serves as a placeholder for the host proxy.
    // Actual implementation will depend on the specific use case.
    println!("[Host] [Proxy] Host proxy function called");

    0
}
