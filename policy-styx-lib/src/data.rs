#![allow(non_camel_case_types)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::PcdIdentity;

pub const PCD_DELEGATOR_ADDR_MAX_LEN: usize = 64usize;
pub const PCD_PROTOCOL_VERSION: usize = 0usize;

pub type PcdProtocolVersion = u64;
pub type PcdDelegatorAddr = [u8; 64usize];

/// Capsulated encrypted data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PcdEncData {
    pub protocol_version: PcdProtocolVersion,
    pub owner_id: PcdIdentity,
    // pub enc_algo: PcdCryptoAlgo,
    // pub enc_size: u64,
    // pub delegator_addr: PcdDelegatorAddr,
    pub encrypted_payload: Vec<u8>,
}

/// Plaintext payload before encryption
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PcdPayload {
    /// Data + Policy + Tags + Attributes
    pub data_size: u64,
    pub policy_size: u64,
    pub tag_size: u64,
    pub attribute_size: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PcdDataset {
    pub dataset_policy_passed: bool,
    pub data_count: u32,
    pub data_max_count: u32,
    pub policy_type: Uuid,
    pub payload_ptr: PcdPayload,
}
