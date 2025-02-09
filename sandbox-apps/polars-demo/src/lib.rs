use std::sync::OnceLock;

use uuid::Uuid;

#[allow(unused)]

struct PcdRuntimeCtx {
    handle: i64,
}

static PCD_RUNTIME_CTX: OnceLock<PcdRuntimeCtx> = OnceLock::new();

extern "C" {
    // Get the target data.
    fn pcd_dataset_access(ctx: i64, data: *const u8) -> i32;
    // Release the given data.
    fn pcd_dataset_release(ctx: i64, data: *const u8) -> i32;
}

#[no_mangle]
pub unsafe extern "C" fn polars_demo(ctx: i64) -> i32 {
    println!("Registering runtime handle {ctx}!");
    let runtime = PcdRuntimeCtx { handle: ctx };

    if let Err(_) = PCD_RUNTIME_CTX.set(runtime) {
        eprintln!("Failed to set the runtime context! Set twice.");
        // We ignore the error here.
    }

    let uuid = Uuid::new_v4();
    let uuid_ptr = uuid.as_bytes().as_ptr();
    let ret = pcd_dataset_access(ctx, uuid_ptr);

    0
}
