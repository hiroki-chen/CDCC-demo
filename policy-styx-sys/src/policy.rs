use std::ffi::c_void;

use crate::crypto::PcdSha256;
use crate::dataset::PcdDataset;
use crate::identity::PcdIdentity;
use crate::runtime::PcdModule;
use crate::uuid::Uuid;

pub type PcdPolicyType = Uuid;

#[repr(C, packed)]
pub struct PcdPolicy {
    pub policy_type: PcdPolicyType,
    pub policy_size: u64,
    pub policy_buffer: *mut u8, // Flexible array member
}

#[repr(C)]
pub struct PcdPolicyDisc {
    pub disc_id: PcdPolicyType,
    pub disc_module: *mut PcdModule,
}

// Constants
pub const PCD_POLICY_TYPE_MAX_INDEX: u32 = 0x0008;
pub const PCD_POLICY_STACK_SIZE: usize = 4 * 1024 * 1024; // 4 MiB
pub const PCD_POLICY_HEAP_SIZE: usize = 16 * 1024 * 1024; // 16 MiB

extern "C" {
    /// Evaluates a policy over a dataset.
    pub fn pcd_policy_eval_over_dataset(
        dataset: *mut PcdDataset,
        program_owner_id: *mut PcdIdentity,
    ) -> i32;

    /// Loads a policy discovery module.
    pub fn pcd_policy_load_disc(
        module_buffer: *mut u8,
        module_size: usize,
        policy_type: *mut PcdPolicyType,
    ) -> i32;

    /// Evaluates the output of a policy over a dataset.
    pub fn pcd_policy_eval_output_over_dataset(
        dataset: *mut PcdDataset,
        data: *mut c_void,
        data_size: usize,
        data_owner_id: *mut PcdIdentity,
        policy: *mut PcdPolicy,
        attributes: *mut c_void,
        attribute_size: u64,
    ) -> i32;

    pub fn pcd_get_program_hash(output_hash: *mut PcdSha256) -> u32;

    pub fn pcd_get_output_custodian(output_id: *mut PcdIdentity) -> u32;
}
