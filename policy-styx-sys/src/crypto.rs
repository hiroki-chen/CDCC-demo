use std::ffi::c_void;

use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{AeadCore, Aes128Gcm, Key, KeyInit};
use anyhow::anyhow;
use sha2::{Digest, Sha256};

use crate::StyxResult;

pub type PcdCryptoAlgo = u32;
pub type PcdSha256 = [u8; 32usize];

// Crypto algorithm definitions
pub const PCD_CRYPTO_PLAIN: PcdCryptoAlgo = 0x0000;
pub const PCD_CRYPTO_AES_GCM: PcdCryptoAlgo = 0x0001;
pub const PCD_CRYPTO_AES_IV_LEN: usize = 12;
pub const PCD_CRYPTO_ALGO_MAX_INDEX: PcdCryptoAlgo = 0x0001;

pub type PcdCryptoNonce = u64;

/// Define the crypto algorithm structure
#[repr(C)]
pub struct PcdCryptoAlgoStruct {
    pub decrypt: Option<
        extern "C" fn(
            ciphertext: *mut c_void,
            cipher_size: usize,
            plaintext: *mut *mut c_void,
            plain_size: *mut usize,
            metadata: *mut c_void,
            key: *mut c_void,
        ) -> i32,
    >,
    pub encrypt: Option<
        extern "C" fn(
            plaintext: *mut c_void,
            plain_size: usize,
            ciphertext: *mut *mut c_void,
            cipher_size: *mut usize,
            metadata: *mut c_void,
            key: *mut c_void,
        ) -> i32,
    >,
    pub verify: Option<
        extern "C" fn(
            ciphertext: *mut c_void,
            cipher_size: usize,
            metadata: *mut c_void,
            key: *mut c_void,
        ) -> i32,
    >,
    pub generate_metadata: Option<
        extern "C" fn(
            iv: *mut u8,
            aad: *mut u8,
            aad_size: usize,
            mac: *mut c_void,
            output_metadata: *mut *mut c_void,
        ) -> i32,
    >,
    pub get_mac_from_metadata: Option<
        extern "C" fn(
            input_metadata: *mut c_void,
            output_mac: *mut *mut c_void,
            mac_size: *mut usize,
        ) -> i32,
    >,
    pub mac_size: usize,
    pub key_size: usize,
    pub iv_size: usize,
}

pub struct PcdCryptoAesGcmMetadata {
    pub iv: [u8; PCD_CRYPTO_AES_IV_LEN],
    pub aed_size: usize,
    pub aad: [u8],
}

// Function prototypes
extern "C" {
    pub fn pcd_crypto_get_algo(algo_id: PcdCryptoAlgo) -> *mut PcdCryptoAlgoStruct;
    pub fn pcd_crypto_register_algo(algo: *mut PcdCryptoAlgoStruct, algo_id: PcdCryptoAlgo) -> i32;
    pub fn pcd_crypto_init() -> i32;
    pub fn pcd_crypto_sha256_hash_buffer(
        buffer: *mut u8,
        buffer_size: usize,
        hash: *mut PcdSha256,
    ) -> i32;

    pub fn pcd_crypto_aes_gcm_init() -> i32;
    pub fn pcd_crypto_aes_gcm_exit() -> i32;
}

/// Encrypts the plaintext using AES-GCM and returns the ciphertext and nonce.
pub fn pcd_crypto_backend_aes_gcm_encrypt(
    plaintext: &[u8],
    key: &[u8],
) -> StyxResult<(Vec<u8>, Vec<u8>)> {
    let key = Key::<Aes128Gcm>::from_slice(key);
    let nonce = Aes128Gcm::generate_nonce(&mut OsRng);

    let cipher = Aes128Gcm::new(key);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| anyhow!("Failed to encrypt"))?;

    Ok((ciphertext, nonce.to_vec()))
}

pub fn pcd_crypto_backend_aes_gcm_decrypt(
    ciphertext: &[u8],
    key: &[u8],
    nonce: &[u8],
) -> StyxResult<Vec<u8>> {
    let key = Key::<Aes128Gcm>::from_slice(key);
    let cipher = Aes128Gcm::new(key);

    let nonce = <[u8; 12]>::try_from(nonce)
        .map_err(|_| anyhow!("Invalid nonce"))?
        .into();

    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| anyhow!("Failed to decrypt"))
}

pub fn pcd_crypto_backend_sha256_hash_buffer(buffer: &[u8]) -> StyxResult<Vec<u8>> {
    let mut hasher = Sha256::new();

    hasher.update(buffer);

    Ok(hasher.finalize().to_vec())
}
