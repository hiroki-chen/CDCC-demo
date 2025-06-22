use std::collections::HashMap;
use std::ffi::c_void;
use std::fs;
use std::sync::{Arc, RwLock};

use anyhow::anyhow;
use uuid::Uuid;
use wasi_common::sync::add_to_linker;
use wasi_common::WasiCtx;
pub use wasmtime::{Engine, Func, IntoFunc, Linker, Memory, Result, Store, Val};
use wasmtime::{WasmParams, WasmResults};

use crate::crypto::pcd_crypto_backend_sha256_hash_buffer;
use crate::types::{PcdInstance, PcdModule};

/// A simple session.
#[derive(Debug, Default)]
pub struct Session {
    pub id: Uuid,
    pub app_idx: Option<usize>, // The index of the application in the PCD WASM runtime.
    pub key: Vec<u8>,
}

pub struct PcdRuntimeState {
    pub wasi: WasiCtx,
    pub sessions: HashMap<Uuid, Session>,
    /// Special: the policy engine.
    pub(crate) policy_engine: Arc<RwLock<Option<PcdApp>>>,
}

/// The WASM runtime structure.
pub struct PcdWasmRuntime {
    pub(crate) linker: Linker<PcdRuntimeState>,
    pub store: Store<PcdRuntimeState>,
    pub(crate) app_registry: Vec<PcdApp>,
}

pub struct PcdWasmRuntimeBuilder {
    engine: Engine,
    linker: Linker<PcdRuntimeState>,
}

impl PcdWasmRuntimeBuilder {
    /// Starts building a new runtime.
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);

        // Add WASI functions immediately. This is the base layer.
        add_to_linker(&mut linker, |s: &mut PcdRuntimeState| &mut s.wasi)?;

        Ok(Self { engine, linker })
    }

    /// Register a custom native function with the runtime builder.
    pub fn with_host_function<Params, Results>(
        mut self,
        name: &str,
        func: impl IntoFunc<PcdRuntimeState, Params, Results>,
    ) -> Result<Self> {
        // Note: The original code defines 'func' inside the runtime, which is complex.
        // It's simpler to define it against the linker directly before the store exists.
        self.linker.func_wrap("pcd_host_api", name, func)?;
        Ok(self)
    }

    /// Consumes the builder to produce the final, ready-to-use runtime.
    pub fn build(self, wasi_ctx: WasiCtx) -> PcdWasmRuntime {
        let state = PcdRuntimeState {
            wasi: wasi_ctx,
            sessions: HashMap::new(),
            policy_engine: Arc::new(RwLock::new(None)),
        };

        let store = Store::new(&self.engine, state);

        PcdWasmRuntime {
            linker: self.linker,
            store,
            app_registry: Vec::new(),
        }
    }
}

/// The application structure.
pub struct PcdApp {
    /// The application module.
    #[allow(dead_code)]
    pub(crate) module: PcdModule,
    /// The application instance.
    pub(crate) instance: PcdInstance,
    /// The application hash.
    #[allow(dead_code)]
    pub(crate) hash: Vec<u8>,
}

impl PcdWasmRuntime {
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

    #[inline]
    pub fn pcd_app_get_app(&self, idx: usize) -> Option<&PcdApp> {
        self.app_registry.get(idx)
    }

    #[inline]
    pub fn is_init(&self) -> bool {
        self.store.data().policy_engine.read().unwrap().is_some()
    }

    pub fn load_policy_engine(&mut self, path: &str) -> Result<()> {
        let policy_app = self.load_wasm_module(path)?;
        let mut policy_engine = self.store.data().policy_engine.write().unwrap();
        if policy_engine.is_some() {
            return Err(anyhow!("Policy engine is already loaded"));
        }
        *policy_engine = Some(policy_app);

        Ok(())
    }

    pub fn load_new_application(&mut self, path: &str) -> Result<usize> {
        let app = self.load_wasm_module(path)?;
        let idx = self.app_registry.len();
        self.app_registry.push(app);

        Ok(idx)
    }

    pub fn execute_typed_function<P, R>(
        &mut self,
        idx: usize,
        func_name: &str,
        params: P,
    ) -> Result<R>
    where
        P: WasmParams,
        R: WasmResults,
    {
        let app = self.app_registry.get(idx).ok_or(anyhow!("App not found"))?;
        let func = app
            .instance
            .get_typed_func::<P, R>(&mut self.store, func_name)?;

        func.call(&mut self.store, params)
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
        let buffer = fs::read(path)?;
        let hash = pcd_crypto_backend_sha256_hash_buffer(&buffer)?;
        let app_module = PcdModule::from_binary(&self.store.engine(), &buffer)?;
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
