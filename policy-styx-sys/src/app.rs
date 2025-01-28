use crate::crypto::PcdSha256;
use crate::identity::PcdIdentity;
use crate::runtime::PcdInstance;

extern "C" {
    /// Gets the SHA-256 hash of the application.
    pub fn pcd_app_get_hash() -> *mut PcdSha256;

    /// Gets the custodian ID of the application.
    pub fn pcd_app_get_custodian_id() -> *mut PcdIdentity;

    /// Gets the instance of the application.
    pub fn pcd_app_get_instance() -> *mut PcdInstance;

    /// Loads an application module.
    pub fn pcd_app_load(
        app_module_buffer: *mut u8,
        app_module_size: u32,
        custodian_id: *mut PcdIdentity,
    ) -> i32;

    /// Runs the application with specified parameters.
    pub fn pcd_app_run(
        dir_allow_list: *const *const u8, // Array of null-terminated C strings
        dir_allow_count: u32,
        stack_size: u32,
        heap_size: u32,
        argv: *mut u32, // Array of arguments
        argc: u32,
    ) -> i32;

    /// Unloads the application module.
    pub fn pcd_app_unload() -> i32;
}
