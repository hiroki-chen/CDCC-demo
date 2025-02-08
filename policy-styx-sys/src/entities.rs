use std::ffi::c_void;

use crate::crypto::PcdCryptoAlgo;
use crate::data::{PcdDelegatorAddr, PcdEncData, PcdPayload};
use crate::identity::PcdIdentity;
use crate::policy::PcdPolicy;
use crate::uuid::Uuid;

extern "C" {
    /// Decrypts an encrypted payload with its MAC.
    pub fn pcd_data_unpacker_decrypt_payload(
        enc_payload_with_mac: *mut c_void,
        enc_size_with_mac: usize,
        owner_id: *mut PcdIdentity,
        crypto_algo: PcdCryptoAlgo,
        key: *mut c_void,
        payload: *mut *mut PcdPayload,
    ) -> i32;

    /// Generates packed data from various input parameters.
    pub fn pcd_data_packer_generate_data(
        data: *mut c_void,
        data_size: usize,
        owner_id: *mut PcdIdentity,
        delegator_addr: *mut PcdDelegatorAddr,
        policy: *mut PcdPolicy,
        tags: *mut Uuid,
        tag_count: u32,
        attributes: *mut c_void,
        attribute_size: usize,
        crypto_algo: PcdCryptoAlgo,
        key: *mut c_void,
        out_data: *mut *mut PcdEncData,
    ) -> i32;

    /// Generates packed encrypted data.
    pub fn pcd_generate_data(
        data: *mut c_void,
        data_size: usize,
        data_owner_id: *mut PcdIdentity,
        delegator_addr: *mut PcdDelegatorAddr,
        policy: *mut PcdPolicy,
        tags: *mut Uuid,
        tag_count: u32,
        attributes: *mut c_void,
        attribute_size: u64,
        crypto_algo: PcdCryptoAlgo,
        output_data: *mut *mut PcdEncData,
    ) -> i32;

    /// Generates packed encrypted data and writes to a specified output path.
    pub fn pcd_consumer_generate_data_to_path(
        data: *mut c_void,
        data_size: usize,
        data_owner_id: *mut PcdIdentity,
        delegator_addr: *mut PcdDelegatorAddr,
        policy: *mut PcdPolicy,
        tags: *mut Uuid,
        tag_count: u32,
        attributes: *mut c_void,
        attribute_size: usize,
        crypto_algo: PcdCryptoAlgo,
        output_path: *const u8, // Null-terminated C-style string
    ) -> i32;

    /// Generates packed encrypted data.
    pub fn pcd_consumer_generate_data(
        data: *mut c_void,
        data_size: usize,
        data_owner_id: *mut PcdIdentity,
        delegator_addr: *mut PcdDelegatorAddr,
        policy: *mut PcdPolicy,
        tags: *mut Uuid,
        tag_count: u32,
        attributes: *mut c_void,
        attribute_size: u64,
        crypto_algo: PcdCryptoAlgo,
        output_data: *mut *mut PcdEncData,
    ) -> i32;
}
