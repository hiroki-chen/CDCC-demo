#![feature(once_cell_get_mut)]

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use policy_styx_lib::dataset::PcdDataset;
use uuid::Uuid;

type DataRegistry = HashMap<Uuid, PcdDataset>;

static DATA_REGISTRY: LazyLock<Arc<Mutex<DataRegistry>>> =
    LazyLock::new(|| Arc::new(Mutex::new(DataRegistry::new())));

#[no_mangle]
pub unsafe extern "C" fn pcd_dataset_add_data(input_data: *const u8, input_data_len: u32) -> i64 {
    if input_data.is_null() || input_data_len == 0 {
        return -1; // Invalid input
    }

    let input_data = std::slice::from_raw_parts(input_data, input_data_len as usize);
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
pub unsafe extern "C" fn deallocate(ptr: *mut u8, size: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }

    let _ = Vec::from_raw_parts(ptr, 0, size);
}

// ------------- Policy Engine APIs ------------- //
#[no_mangle]
pub unsafe extern "C" fn eval_input(
    data: *const u8,
    data_len: u32,
    key: *const u8,
    key_len: u32,
) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn eval_output(
    data: *const u8,
    data_len: u32,
    key: *const u8,
    key_len: u32,
) -> i32 {
    0
}
