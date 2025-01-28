use std::ffi::c_void;

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
