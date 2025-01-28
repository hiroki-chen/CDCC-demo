use std::ffi::c_void;

use crate::data::{PcdEncData, PcdPayload};
use crate::policy::PcdPolicyType;
use crate::runtime::PcdRuntimePointer;

pub const PCD_MAX_AMOUNT_DATASET: usize = 16;

#[repr(C, packed)]
pub struct PcdDataset {
    pub dataset_policy_passed: u8,
    pub data_count: u32,
    pub data_max_count: u32,
    pub policy_type: PcdPolicyType,
    pub payload_pointers: *mut *mut PcdPayload, // Flexible array member in C
}

extern "C" {
    /// Creates a new dataset with the specified maximum size.
    pub fn pcd_dataset_new(dataset_max_size: u32) -> u32;

    /// Adds encrypted data to a dataset.
    pub fn pcd_dataset_add_data(dataset_index: u32, input_data: *mut PcdEncData) -> u32;

    /// Checks the policy for a dataset.
    pub fn pcd_dataset_check_policy(
        dataset_index: u32,
        program_owner_id: *mut c_void, // `pcd_identity_t` as opaque type
    ) -> u32;

    /// Accesses data within a dataset.
    pub fn pcd_dataset_access(dataset_index: u32, data_index: u32) -> PcdRuntimePointer;

    /// Releases a dataset.
    pub fn pcd_dataset_release(dataset_index: u32) -> u32;
}
