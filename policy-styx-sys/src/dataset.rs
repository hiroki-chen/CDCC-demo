use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

use wamr_rust_sdk::sys::{wasm_module_inst_t, wasm_runtime_module_dup_data};

use crate::app::pcd_app_get_instance;
use crate::data::PcdPayload;
use crate::identity::PcdIdentity;
use crate::policy::pcd_policy_eval_over_dataset;
use crate::runtime::PcdRuntimePointer;
use crate::uuid::Uuid;

#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct PcdDataset {
    pub dataset_policy_passed: bool,
    pub data_count: u32,
    pub data_max_count: u32,
    pub policy_type: Uuid,
    pub payload_ptr: *const PcdPayload,
}

const PCD_DATA_REGISTRY: LazyLock<Arc<RwLock<HashMap<Uuid, PcdDataset>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

pub(crate) fn pcd_dataset_access(
    instance: wasm_module_inst_t,
    uuid: &Uuid,
    idx: usize,
) -> PcdRuntimePointer {
    let lock = PCD_DATA_REGISTRY;
    let lock = lock.read().unwrap();
    let dataset = match lock.get(uuid) {
        Some(dataset) => dataset,
        None => return std::ptr::null_mut(), // nullptr.
    };

    if idx >= dataset.data_count as _ {
        return std::ptr::null_mut();
    }

    let payload = unsafe {
        &*(dataset
            .payload_ptr
            .add(idx * std::mem::size_of::<PcdPayload>()) as *const PcdPayload)
    };

    unsafe {
        wasm_runtime_module_dup_data(
            instance,
            payload.payload as *const i8,
            payload.data_size as u64,
        ) as *mut u8
    }
}

pub(crate) fn pcd_dataset_release(uuid: &Uuid) -> i32 {
    let lock = PCD_DATA_REGISTRY;
    let mut lock = lock.write().unwrap();
    lock.remove(uuid);

    0
}

pub(crate) fn pcd_dataset_add_data(data_uuid: &Uuid, dataset: &PcdDataset) -> i32 {
    let lock = PCD_DATA_REGISTRY;
    let mut lock = lock.write().unwrap();
    lock.insert(data_uuid.clone(), dataset.clone());

    0
}

pub(crate) fn pcd_dataset_check_policy(uuid: &Uuid, program_owner_id: PcdIdentity) -> i32 {
    let lock = PCD_DATA_REGISTRY;
    let lock = lock.read().unwrap();
    let dataset = match lock.get(uuid) {
        Some(dataset) => dataset,
        None => return -1, // not found.
    };

    return pcd_policy_eval_over_dataset(dataset, program_owner_id);
}

#[no_mangle]
pub unsafe extern "C" fn pcd_dataset_access_wrapper(
    data_uuid: *const u8,
    idx: usize,
) -> PcdRuntimePointer {
    let uuid = std::slice::from_raw_parts(data_uuid, 16)
        .try_into()
        .unwrap();

    let instance = pcd_app_get_instance();
    if instance.is_null() {
        return std::ptr::null_mut();
    }

    pcd_dataset_access(instance, &uuid, idx)
}

#[no_mangle]
pub unsafe extern "C" fn pcd_dataset_release_wrapper(uuid: *const u8) -> i32 {
    let uuid = std::slice::from_raw_parts(uuid, 16).try_into().unwrap();

    pcd_dataset_release(uuid)
}

#[no_mangle]
pub unsafe extern "C" fn pcd_dataset_add_data_wrapper(
    // _exec_env: wasm_exec_env_t,
    data_uuid: *const u8,
    dataset: *const PcdDataset,
) -> i32 {
    let data_uuid = std::slice::from_raw_parts(data_uuid, 16)
        .try_into()
        .unwrap();
    let dataset = &*dataset;

    pcd_dataset_add_data(&data_uuid, dataset)
}

#[no_mangle]
pub unsafe extern "C" fn pcd_dataset_check_policy_wrapper(
    data_uuid: *const u8,
    program_owner_id: *const PcdIdentity,
) -> i32 {
    let data_uuid = std::slice::from_raw_parts(data_uuid, 16)
        .try_into()
        .unwrap();
    let program_owner_id = &*program_owner_id;

    pcd_dataset_check_policy(&data_uuid, *program_owner_id)
}
