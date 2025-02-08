use std::sync::{Arc, LazyLock, RwLock};

use crate::runtime::{PcdInstance, PcdModule};

/// A global lock for the application module.
const PCD_APP_MODULE: LazyLock<Arc<RwLock<PcdModule>>> =
    LazyLock::new(|| Arc::new(RwLock::new(std::ptr::null_mut())));

/// A global lock for the application instance.
const PCD_APP_INSTANCE: LazyLock<Arc<RwLock<PcdInstance>>> =
    LazyLock::new(|| Arc::new(RwLock::new(std::ptr::null_mut())));

pub fn pcd_register_app_module(module: PcdModule) {
    let lock = PCD_APP_MODULE;
    let mut lock = lock.write().unwrap();
    *lock = module;
}

pub fn pcd_register_app_instance(instance: PcdInstance) {
    let lock = PCD_APP_INSTANCE;
    let mut lock = lock.write().unwrap();
    *lock = instance;
}

pub fn pcd_app_get_instance() -> PcdInstance {
    let lock = PCD_APP_INSTANCE;
    let lock = lock.read().unwrap();
    (*lock) as PcdInstance
}

pub fn pcd_app_get_module() -> PcdModule {
    let lock = PCD_APP_MODULE;
    let lock = lock.read().unwrap();
    (*lock) as PcdModule
}
