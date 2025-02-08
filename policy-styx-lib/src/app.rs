use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ops::Range;

use anyhow::anyhow;
use uuid::Uuid;
use wasi_common::snapshots::preview_0::wasi_unstable::WasiUnstable;
use wasi_common::snapshots::preview_1::wasi_snapshot_preview1::WasiSnapshotPreview1;
use wasi_common::sync::add_to_linker;
use wasmtime::{Engine, Func, IntoFunc, Linker, Memory, MemoryType, Result, Store, Val};

use crate::dataset::PcdDataset;
use crate::types::{PcdInstance, PcdModule};

/// The WASM runtime structure.
pub struct PcdWasmRuntime<'runtime, T> {
    linker: Linker<T>,
    store: Store<T>,
    /// The application registry.
    app_registry: HashMap<Cow<'runtime, str>, PcdApp>,
    /// The PCD data registry.
    data_registry: HashMap<Uuid, PcdDataset>,
    /// Special: the policy engine.
    policy_engine: Option<PcdApp>,
}

/// The application structure.
pub struct PcdApp {
    /// The application module.
    module: PcdModule,
    /// The application instance.
    instance: PcdInstance,
    /// The application memory.
    memory: Memory,
}

impl<'runtime, T> PcdWasmRuntime<'runtime, T>
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
            app_registry: HashMap::new(),
            data_registry: HashMap::new(),
            policy_engine: None,
        })
    }

    #[inline]
    pub fn pcd_app_get_app(&self, name: &str) -> Option<&PcdApp> {
        self.app_registry.get(name)
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

    pub fn load_policy_engine(&mut self, path: &str, mem_size: Range<u32>) -> Result<()> {
        let policy_app = self.load_wasm_module(path, mem_size)?;
        self.policy_engine = Some(policy_app);

        Ok(())
    }

    pub fn load_new_application(
        &mut self,
        name: &'runtime str,
        path: &'runtime str,
        mem_size: Range<u32>,
    ) -> Result<()> {
        let app = self.load_wasm_module(path, mem_size)?;
        self.app_registry.insert(name.into(), app);

        Ok(())
    }

    pub fn execute_function(
        &mut self,
        app_name: &str,
        func_name: &str,
        params: &[Val],
        results: &mut [Val],
    ) -> Result<()> {
        let app = self
            .app_registry
            .get(app_name)
            .ok_or(anyhow!("App not found"))?;
        let func = app
            .instance
            .get_func(&mut self.store, func_name)
            .ok_or(anyhow!("Function not found"))?;

        println!("Executing function: {func_name} with params: {params:?}");
        func.call(&mut self.store, params, results)
    }

    fn load_wasm_module(&mut self, path: &str, mem_size: Range<u32>) -> Result<PcdApp> {
        let memory = Memory::new(
            &mut self.store,
            MemoryType::new(mem_size.start, Some(mem_size.end)),
        )?;

        let app_module = PcdModule::from_file(&self.store.engine(), path)?;
        let app_instance = self.linker.instantiate(&mut self.store, &app_module)?;
        let app = PcdApp::new(app_module, app_instance, memory);

        Ok(app)
    }
}

impl PcdApp {
    /// Create a new application.
    #[inline]
    pub fn new(module: PcdModule, instance: PcdInstance, memory: Memory) -> Self {
        Self {
            module,
            instance,
            memory,
        }
    }
}
