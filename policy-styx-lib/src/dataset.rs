// use std::collections::HashMap;
// use std::sync::{Arc, LazyLock, RwLock};

// // use wamr_rust_sdk::sys::{wasm_module_inst_t, wasm_runtime_module_dup_data};

use uuid::Uuid;

use crate::data::PcdPayload;

pub fn pcd_data_access() -> i64 {
    println!("pcd_data_access called");
    0
}

#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct PcdDataset {
    pub dataset_policy_passed: bool,
    pub data_count: u32,
    pub data_max_count: u32,
    pub policy_type: Uuid,
    pub payload_ptr: *const PcdPayload,
}

// pub(crate) fn pcd_dataset_access(
//     instance: &PcdInstance,
//     uuid: &Uuid,
//     idx: usize,
// ) -> PcdRuntimeOffset {
//     let lock = PCD_DATA_REGISTRY;
//     let lock = lock.read().unwrap();
//     let dataset = match lock.get(uuid) {
//         Some(dataset) => dataset,
//         None => return -1, // nullptr.
//     };

//     if idx >= dataset.data_count as _ {
//         return -1;
//     }

//     let payload = unsafe {
//         &*(dataset
//             .payload_ptr
//             .add(idx * std::mem::size_of::<PcdPayload>()) as *const PcdPayload)
//     };

//     // get the shared memory.

//     // unsafe {
//     //     wasm_runtime_module_dup_data(
//     //         instance,
//     //         payload.payload as *const i8,
//     //         payload.data_size as u64,
//     //     ) as *mut u8
//     // }
// }

// pub(crate) fn pcd_dataset_release(uuid: &Uuid) -> i32 {
//     let lock = PCD_DATA_REGISTRY;
//     let mut lock = lock.write().unwrap();
//     lock.remove(uuid);

//     0
// }

// pub(crate) fn pcd_dataset_add_data(data_uuid: &Uuid, dataset: &PcdDataset) -> i32 {
//     let lock = PCD_DATA_REGISTRY;
//     let mut lock = lock.write().unwrap();
//     lock.insert(data_uuid.clone(), dataset.clone());

//     0
// }

// pub(crate) fn pcd_dataset_check_policy(uuid: &Uuid, program_owner_id: PcdIdentity) -> i32 {
//     let lock = PCD_DATA_REGISTRY;
//     let lock = lock.read().unwrap();
//     let dataset = match lock.get(uuid) {
//         Some(dataset) => dataset,
//         None => return -1, // not found.
//     };

//     return pcd_policy_eval_over_dataset(dataset, program_owner_id);
// }

// #[no_mangle]
// pub unsafe extern "C" fn pcd_dataset_access_wrapper(
//     data_uuid: *const u8,
//     idx: usize,
// ) -> PcdRuntimeOffset {
//     let uuid = std::slice::from_raw_parts(data_uuid, 16)
//         .try_into()
//         .unwrap();

//     let instance = match pcd_app_get_instance() {
//         Some(instance) => instance,
//         None => return -1,
//     };

//     let instance = instance.lock().unwrap();

//     pcd_dataset_access(&*instance, &uuid, idx)
// }

// #[no_mangle]
// pub unsafe extern "C" fn pcd_dataset_release_wrapper(uuid: *const u8) -> i32 {
//     let uuid = std::slice::from_raw_parts(uuid, 16).try_into().unwrap();

//     pcd_dataset_release(uuid)
// }

// #[no_mangle]
// pub unsafe extern "C" fn pcd_dataset_add_data_wrapper(
//     // _exec_env: wasm_exec_env_t,
//     data_uuid: *const u8,
//     dataset: *const PcdDataset,
// ) -> i32 {
//     let data_uuid = std::slice::from_raw_parts(data_uuid, 16)
//         .try_into()
//         .unwrap();
//     let dataset = &*dataset;

//     pcd_dataset_add_data(&data_uuid, dataset)
// }

// #[no_mangle]
// pub unsafe extern "C" fn pcd_dataset_check_policy_wrapper(
//     data_uuid: *const u8,
//     program_owner_id: *const PcdIdentity,
// ) -> i32 {
//     let data_uuid = std::slice::from_raw_parts(data_uuid, 16)
//         .try_into()
//         .unwrap();
//     let program_owner_id = &*program_owner_id;

//     pcd_dataset_check_policy(&data_uuid, *program_owner_id)
// }
