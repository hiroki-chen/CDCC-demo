use wasmtime::Caller;

use crate::app::PcdRuntimeState;

// --------- Host Proxy Functions --------- //
pub fn pcd_dataset_data_access_host(mut caller: Caller<'_, PcdRuntimeState>) -> i32 {
    // This function serves as a placeholder for the host proxy.
    // Actual implementation will depend on the specific use case.
    println!("Host proxy function called");

    0
}

pub fn pcd_dataset_data_release_host(mut caller: Caller<'_, PcdRuntimeState>) -> i32 {
    // This function serves as a placeholder for the host proxy.
    // Actual implementation will depend on the specific use case.
    println!("Host proxy function called");

    0
}
