use crate::identity::PcdIdentity;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PcdSecretReqType {
    Request = 0,
    Register = 1,
    ResponseOk = 2,
    ResponseNotFound = 3,
    ResponseDenied = 4,
    ResponseRegOk = 5,
}

#[repr(C, packed)]
pub struct PcdSecret {
    pub secret_size: u32,
    pub secret: [u8; 0], // Flexible array member in C, represented as zero-length array in Rust
}

#[repr(C, packed)]
pub struct PcdSecretReq {
    pub req_type: PcdSecretReqType,
    pub id: PcdIdentity, // Assuming `pcd_identity_t` is an opaque type
    pub secret: PcdSecret,
}

extern "C" {
    /// Registers a secret for the specified identity.
    pub fn pcd_secret_register(id: *mut PcdIdentity, input_secret: *mut PcdSecret) -> i32;

    /// Releases a registered secret for the specified identity.
    pub fn pcd_secret_release(id: *mut PcdIdentity) -> i32;

    #[cfg(feature = "pcd_config_secret_requester")]
    /// Fetches a secret for the specified identity and delegator address.
    pub fn pcd_secret_fetch(
        id: *mut PcdIdentity,
        delegator_addr: *mut PcdDelegatorAddr,
        output_secret: *mut *mut PcdSecret,
    ) -> i32;

    /// Retrieves a secret for the specified identity.
    pub fn pcd_secret_retrieve(id: *mut PcdIdentity, output_secret: *mut *mut PcdSecret) -> i32;
}