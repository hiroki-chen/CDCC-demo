#![allow(non_camel_case_types)]

use crate::crypto::PcdCryptoAlgo;
use crate::identity::PcdIdentity;

pub const PCD_DELEGATOR_ADDR_MAX_LEN: usize = 64usize;
pub const PCD_PROTOCOL_VERSION: usize = 0usize;

pub type PcdProtocolVersion = u64;
pub type PcdDelegatorAddr = [u8; 64usize];

/// Capsulated encrypted data
#[repr(C, packed)]
pub struct PcdEncData {
    pub protocol_version: PcdProtocolVersion,
    pub owner_id: PcdIdentity,
    pub enc_algo: PcdCryptoAlgo,
    pub enc_size: u64,
    pub delegator_addr: PcdDelegatorAddr,

    pub encrypted_payload: *mut u8,
}

/// Plaintext payload before encryption
#[repr(C)]
pub struct PcdPayload {
    /// Data + Policy + Tags + Attributes
    pub data_size: u64,
    pub policy_size: u64,
    pub tag_size: u64,
    pub attribute_size: u64,

    pub payload: *mut u8,
}
