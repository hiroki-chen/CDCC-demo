use std::sync::{Arc, LazyLock, RwLock};

use sha2::digest::block_buffer::Lazy;

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

const PCD_POLICY_LOADED: LazyLock<Arc<RwLock<usize>>> = LazyLock::new(|| Arc::new(RwLock::new(0)));
const PCD_POLICY_TYPES: LazyLock<Arc<RwLock<Vec<PcdPolicyDisc>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(Vec::new())));

pub(crate) fn pcd_policy_eval_over_dataset(
    dataset: &PcdDataset,
    program_owner_id: PcdIdentity,
) -> i32 {
    let policy_loaded = {
        let lock = PCD_POLICY_LOADED;
        let lock = lock.read().unwrap();
        *lock
    };

    let lock = PCD_POLICY_TYPES;
    let policy_types = lock.read().unwrap();

    for i in 0..policy_loaded {
        if !policy_types[i].disc_id.eq(&dataset.policy_type) {
            continue;
        }
    }

    0
}
