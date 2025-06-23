use aes_gcm::aead::rand_core::CryptoRngCore;
use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit};
use anyhow::anyhow;
use sha2::{Digest, Sha256};

use crate::types::PcdResult;

/// Encrypts the plaintext using AES-GCM and returns the ciphertext and nonce.
pub fn pcd_crypto_backend_aes_gcm_encrypt(
    plaintext: &[u8],
    key: &[u8],
) -> PcdResult<(Vec<u8>, Vec<u8>)> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let cipher = Aes256Gcm::new(key);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| anyhow!("Failed to encrypt"))?;

    Ok((ciphertext, nonce.to_vec()))
}

pub fn pcd_crypto_backend_aes_gcm_decrypt(
    ciphertext: &[u8],
    key: &[u8],
    nonce: &[u8],
) -> PcdResult<Vec<u8>> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    let nonce = <[u8; 12]>::try_from(nonce)
        .map_err(|_| anyhow!("Invalid nonce"))?
        .into();

    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| anyhow!("Failed to decrypt"))
}

#[inline]
pub fn pcd_crypto_backend_sha256_hash_buffer(buffer: &[u8]) -> PcdResult<Vec<u8>> {
    Ok(Sha256::digest(buffer).to_vec())
}

pub fn pcd_crypto_backend_aes_gcm_randkey_generate() -> PcdResult<Vec<u8>> {
    let mut key = vec![0u8; 32]; // AES-256 key size
    OsRng.as_rngcore().fill_bytes(&mut key);

    Ok(key)
}
