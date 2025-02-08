use std::ffi::c_void;
use std::sync::OnceLock;

#[allow(unused)]

struct PcdRuntimeCtx(*mut c_void);

unsafe impl Sync for PcdRuntimeCtx {}
unsafe impl Send for PcdRuntimeCtx {}

static PCD_RUNTIME_CTX: OnceLock<PcdRuntimeCtx> = OnceLock::new();

extern "C" {
    // Get the target data.
    fn pcd_dataset_access(data: *const u8) -> u64;
    // Release the given data.
    fn pcd_dataset_release(data: *const u8) -> i32;
}

#[no_mangle]
pub unsafe extern "C" fn polars_demo(ctx: i64) -> i32 {
    println!("Registering runtime handle {ctx}!");
    let ctx = PcdRuntimeCtx(ctx as *mut c_void);

    if let Err(_) = PCD_RUNTIME_CTX.set(ctx) {
        eprintln!("Failed to set the runtime context! Set twice.");
        // We ignore the error here.
    }

    0
}
