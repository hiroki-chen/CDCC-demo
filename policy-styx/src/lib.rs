//! Policy engine.

use policy_styx_sys::crypto::PcdSha256;
use policy_styx_sys::data::PcdPayload;
use policy_styx_sys::identity::PcdIdentity;
use policy_styx_sys::policy::{pcd_get_output_custodian, pcd_get_program_hash};

pub type PolicyResult<T> = anyhow::Result<T>;

/// A demo policy for styx.
///
/// This policy does not care about how will the input policy be used.
#[repr(C, packed)]
#[derive(Default, Debug, Clone)]
pub struct PcdDemoPolicyRule {
    pub rule_type: u8,
}

/// Evaluate the input policy.
pub fn eval_input(payload_ptr_array: &[&PcdPayload], payload_amount: usize) -> PolicyResult<()> {
    let mut output_hash = PcdSha256::default();
    let mut output_custodian = PcdIdentity::default();

    let mut ret = unsafe { pcd_get_program_hash(&mut output_hash as _) };

    if ret != 0 {
        return Err(anyhow::anyhow!("Failed to evaluate the policy"));
    }

    ret = unsafe { pcd_get_output_custodian(&mut output_custodian as _) };

    if ret != 0 {
        return Err(anyhow::anyhow!("Failed to evaluate the policy"));
    }

    todo!("Implement the policy evaluation logic");

    Ok(())
}

/// Evaluate the output policy.
pub fn eval_output(
    payload_ptr_array: &[&PcdPayload],
    payload_amount: usize,
    data: &[u8],
    data_owner_id: &PcdIdentity,
    attributes: &[u8],
) -> PolicyResult<()> {
  Ok(())
}
