#![feature(once_cell_get_mut)]

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use policy_styx_lib::data::PcdDataset;
use policy_styx_lib::types::{PcdWasmPtr, PcdWasmRawPtr};
use uuid::Uuid;

type DataRegistry = HashMap<Uuid, PcdDataset>;

static DATA_REGISTRY: LazyLock<Arc<Mutex<DataRegistry>>> =
    LazyLock::new(|| Arc::new(Mutex::new(DataRegistry::new())));

#[no_mangle]
pub unsafe extern "C" fn pcd_dataset_add_data(input_data: PcdWasmRawPtr) -> i64 {
    let ptr = PcdWasmPtr::from(input_data);

    println!("[Sandbox] Adding data to dataset...");
    println!("[Sandbox] Input data pointer: {:?}", ptr);

    let input_data = std::slice::from_raw_parts(ptr.ptr() as _, ptr.len() as usize);
    let input_data: PcdDataset = match bincode::deserialize(input_data) {
        Ok(data) => data,
        Err(_) => return -1, // Deserialization failed
    };

    let uuid = Uuid::new_v4();
    let mut registry = DATA_REGISTRY.lock().unwrap();
    registry.insert(uuid, input_data);

    let uuid_bytes = uuid.as_bytes().to_vec();
    let uuid_len = uuid_bytes.len();
    let ptr = uuid_bytes.as_ptr() as i64;
    std::mem::forget(uuid_bytes); // Prevent deallocation of the vector

    ptr << 32 | (uuid_len as i64) // Return pointer and length
}

// ------------- Memory Management APIs ------------- //
#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut u8 {
    let mut buffer = Vec::with_capacity(size);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn deallocate(ptr: PcdWasmRawPtr) {
    let ptr = PcdWasmPtr::from(ptr);

    if ptr.is_null() {
        return; // Nothing to deallocate
    }

    let _ = Vec::from_raw_parts(ptr.ptr() as *mut u8, ptr.len() as usize, ptr.len() as usize);
}

// --------- Data Access APIs --------- //
#[no_mangle]
pub unsafe extern "C" fn pcd_dataset_data_access(data_uuid: PcdWasmRawPtr) -> PcdWasmRawPtr {
    println!("[Sandbox] [Policy Engine] Accessing dataset data...");

    let data_uuid_len = (data_uuid & 0xFFFFFFFF) as usize; // Extract the length part
    let data_uuid = data_uuid >> 32; // Extract the UUID part from the PcdWasmPtr

    let uuid = std::slice::from_raw_parts(data_uuid as *const u8, data_uuid_len);
    let uuid = match Uuid::from_slice(uuid) {
        Ok(uuid) => uuid,
        Err(_) => return 0, // Invalid UUID format
    };

    match DATA_REGISTRY.lock() {
        Ok(registry) => {
            if let Some(dataset) = registry.get(&uuid) {
                println!(
                    "[Sandbox] [Policy Engine] Trying to access dataset with UUID: {}",
                    uuid
                );

                // TODO: Evaluate with the policy.
                println!("[Sandbox] [Policy Engine] Evaluating dataset access with policy...");
                {
                    // Here you would typically call a method on the policy engine
                    // to handle the dataset data access.
                    // For now, we just print a message.
                    println!("[Sandbox] [[Policy Engine]] Policy Evaluation OK :)");
                }

                // Serialize the dataset to bytes.
                let dataset_bytes = match bincode::serialize(dataset) {
                    Ok(bytes) => bytes,
                    Err(_) => return 0, // Serialization failed
                };

                let dataset_len = dataset_bytes.len() as PcdWasmRawPtr;
                let dataset_ptr = dataset_bytes.as_ptr() as PcdWasmRawPtr;

                std::mem::forget(dataset_bytes);
                // Prevent deallocation of the vector: the caller must handle de-allocation!

                // Return the pointer and length as a FAT pointer
                dataset_ptr << 32 | dataset_len
            } else {
                0 // Dataset not found
            }
        },
        Err(_) => 0, // Lock failed
    }
}

#[no_mangle]
pub unsafe extern "C" fn pcd_dataset_data_release(data_uuid: PcdWasmRawPtr) -> PcdWasmRawPtr {
    todo!()
}
