//! Policy engine.

use std::os::raw::c_void;

use policy_styx_sys::crypto::PcdSha256;
use policy_styx_sys::data::PcdPayload;
use policy_styx_sys::identity::PcdIdentity;

pub type PolicyResult<T> = anyhow::Result<T>;

/// A demo policy for styx.
///
/// This policy does not care about how will the input policy be used.
#[repr(C, packed)]
#[derive(Default, Debug, Clone)]
pub struct PcdDemoPolicyRule {
    pub rule_type: u8,
}

/// Evaluates the input payloads.
///
/// # Arguments
///
/// * `payload_ptr_array` - A pointer to an array of pointers to `PcdPayload`.
/// * `payload_amount` - The number of payloads in the array.
///
/// # Returns
///
/// * `0` if the evaluation is successful.
/// * `-1` if the evaluation fails.
#[no_mangle]
pub extern "C" fn eval_input(
    payload_ptr_array: *const *const PcdPayload,
    payload_amount: usize,
) -> i32 {
    todo!()
}

/// Evaluates the output payloads.
///
/// # Arguments
///
/// * `payload_ptr_array` - A pointer to an array of pointers to `PcdPayload`.
/// * `payload_amount` - The number of payloads in the array.
/// * `data` - A pointer to the data.
/// * `data_size` - The size of the data.
/// * `data_owner_id` - A pointer to the `PcdIdentity` of the data owner.
/// * `attributes` - A pointer to the attributes.
/// * `attribute_size` - The size of the attributes.
///
/// # Returns
///
/// * `0` if the evaluation is successful.
/// * `-1` if the evaluation fails.
#[no_mangle]
pub extern "C" fn eval_output(
    payload_ptr_array: *const *const PcdPayload,
    payload_amount: usize,
    data: *const c_void,
    data_size: usize,
    data_owner_id: *const PcdIdentity,
    attributes: *const c_void,
    attribute_size: usize,
) -> i32 {
    todo!()
}
