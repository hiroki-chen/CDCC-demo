use std::ffi::c_void;

pub type PcdRuntimePointer = u64; // Assuming a 64-bit pointer equivalent
pub type PcdModule = c_void; // Opaque type for `pcd_module_t`
pub type PcdInstance = c_void; // Opaque type for `pcd_instance_t`

extern "C" {
    // Initializes the runtime environment
    pub fn pcd_runtime_setup_environment() -> u32;

    // Loads a module into the runtime from a buffer
    pub fn pcd_runtime_load_module(module_buffer: *mut u8, module_size: u32) -> *mut PcdModule;

    // Instantiates a loaded module
    pub fn pcd_runtime_instantiate_module(
        module: *mut PcdModule,
        dir_allow_list: *const *const u8, // Array of strings (null-terminated)
        dir_allow_count: u32,
        stack_size: u32,
        heap_size: u32,
    ) -> *mut PcdInstance;

    // Executes a function in a module instance
    pub fn pcd_runtime_execute_function(
        instance: *mut PcdInstance,
        func_name: *mut u8, // Function name as a C-style string
        argv: *mut u32,
        argc: u32,
    ) -> u32;

    // Deinstantiates a module instance
    pub fn pcd_runtime_deinstantiate_module(instance: *mut PcdInstance);

    // Unloads a module from the runtime
    pub fn pcd_runtime_unload_module(module: *mut PcdModule);

    // Destroys the runtime environment
    pub fn pcd_runtime_destroy_environment();

    // Allocates memory in the runtime
    pub fn pcd_runtime_malloc(
        instance: *mut PcdInstance,
        data_size: u32,
        native_ptr: *mut *mut c_void,
    ) -> PcdRuntimePointer;

    // Copies data into the runtime memory
    pub fn pcd_runtime_copy_data_into_runtime(
        instance: *mut PcdInstance,
        data: *const c_void,
        data_size: u32,
    ) -> PcdRuntimePointer;

    // Converts a runtime app pointer to a native pointer
    pub fn pcd_runtime_app_to_native(
        instance: *mut PcdInstance,
        app_addr: PcdRuntimePointer,
    ) -> *mut c_void;

    // Converts a native pointer to a runtime app pointer
    pub fn pcd_runtime_native_to_app(
        instance: *mut PcdInstance,
        native_addr: *mut c_void,
    ) -> PcdRuntimePointer;

    // Frees memory in the runtime
    pub fn pcd_runtime_free(instance: *mut PcdInstance, app_addr: PcdRuntimePointer);
}
