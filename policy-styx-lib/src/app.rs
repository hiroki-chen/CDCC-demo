use std::collections::HashMap;
use std::ffi::c_void;
use std::fs;

use anyhow::anyhow;
use uuid::Uuid;
use wasi_common::snapshots::preview_0::wasi_unstable::WasiUnstable;
use wasi_common::snapshots::preview_1::wasi_snapshot_preview1::WasiSnapshotPreview1;
use wasi_common::sync::add_to_linker;
use wasmtime::{Engine, Func, IntoFunc, Linker, Memory, Result, Store, Val};

use crate::crypto::pcd_crypto_backend_sha256_hash_buffer;
use crate::dataset::PcdDataset;
use crate::types::{PcdInstance, PcdModule};

/// The WASM runtime structure.
pub struct PcdWasmRuntime<T> {
    pub(crate) linker: Linker<T>,
    pub(crate) store: Store<T>,
    /// The application registry.
    pub(crate) app_registry: Vec<PcdApp>,
    /// The PCD dataset registry.
    pub(crate) data_registry: HashMap<Uuid, PcdDataset>,
    /// Special: the policy engine.
    pub(crate) policy_engine: Option<PcdApp>,
}

/// The application structure.
pub struct PcdApp {
    /// The application module.
    pub(crate) module: PcdModule,
    /// The application instance.
    pub(crate) instance: PcdInstance,
    /// The application hash.
    pub(crate) hash: Vec<u8>,
}

impl<T> PcdWasmRuntime<T>
where
    T: Send + WasiUnstable + WasiSnapshotPreview1,
{
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut c_void {
        self as *mut Self as *mut c_void
    }

    #[inline]
    pub fn as_ptr(&self) -> *const c_void {
        self as *const Self as *const c_void
    }

    #[inline]
    pub fn read(&self, memory: &Memory, offset: usize, len: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0; len];
        memory.read(&self.store, offset, &mut buffer)?;

        Ok(buffer)
    }

    #[inline]
    pub fn write(&mut self, memory: &mut Memory, offset: usize, data: &[u8]) -> Result<()> {
        memory
            .write(&mut self.store, offset, data)
            .map_err(|e| e.into())
    }

    /// Create a new WASM runtime.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to be stored in the runtime.
    pub fn new(data: T) -> Result<Self> {
        let engine = Engine::default();
        let store = Store::new(&engine, data);
        let mut linker = Linker::new(&engine);

        add_to_linker(&mut linker, |s| s)?;

        Ok(Self {
            linker,
            store,
            app_registry: Vec::new(),
            data_registry: HashMap::new(),
            policy_engine: None,
        })
    }

    #[inline]
    pub fn pcd_app_get_app(&self, idx: usize) -> Option<&PcdApp> {
        self.app_registry.get(idx)
    }

    #[inline]
    pub fn is_init(&self) -> bool {
        self.policy_engine.is_some()
    }

    pub fn register_native_functions<Params, Results>(
        &mut self,
        name: &str,
        func: impl IntoFunc<T, Params, Results>,
    ) -> Result<()> {
        let func = Func::wrap(&mut self.store, func);
        self.linker.define(&mut self.store, "env", name, func)?;

        Ok(())
    }

    pub fn load_policy_engine(&mut self, path: &str) -> Result<()> {
        let policy_app = self.load_wasm_module(path)?;
        self.policy_engine = Some(policy_app);

        Ok(())
    }

    pub fn load_new_application(&mut self, path: &str) -> Result<usize> {
        let app = self.load_wasm_module(path)?;
        let idx = self.app_registry.len();
        self.app_registry.push(app);

        Ok(idx)
    }

    pub fn execute_function(
        &mut self,
        idx: usize,
        func_name: &str,
        params: &[Val],
        results: &mut [Val],
    ) -> Result<()> {
        let app = self.app_registry.get(idx).ok_or(anyhow!("App not found"))?;
        let func = app
            .instance
            .get_func(&mut self.store, func_name)
            .ok_or(anyhow!("Function not found"))?;

        println!("Executing function: {func_name} with params: {params:?}");
        func.call(&mut self.store, params, results)
    }

    fn load_wasm_module(&mut self, path: &str) -> Result<PcdApp> {
        let buffer = fs::read_to_string(path)?;
        let hash = pcd_crypto_backend_sha256_hash_buffer(buffer.as_bytes())?;
        let app_module = PcdModule::from_binary(&self.store.engine(), buffer.as_bytes())?;
        let app_instance = self.linker.instantiate(&mut self.store, &app_module)?;
        let app = PcdApp::new(app_module, app_instance, hash);

        Ok(app)
    }
}

impl PcdApp {
    /// Create a new application.
    #[inline]
    pub fn new(module: PcdModule, instance: PcdInstance, hash: Vec<u8>) -> Self {
        Self {
            module,
            instance,
            hash,
        }
    }
}
