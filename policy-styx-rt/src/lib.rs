use std::ffi::{c_char, c_void, CStr, CString};
use std::fs;
use std::path::PathBuf;

use wamr_rust_sdk::sys::*;
use wamr_rust_sdk::{ExecError, RuntimeError};

pub type WasrResult<T> = Result<T, RuntimeError>;

const ERR_BUF_LEN: usize = 128;

pub struct PcdNativeSymbol<'sym> {
    pub symbol: &'sym str,
    pub func_ptr: *mut c_void,
    pub signature: &'sym str,
}

/// Setup the environment for the policy-styx runtime and also register native symbols.
pub fn pcd_env_init(native_symbol_list: &[&PcdNativeSymbol]) {
    unsafe {
        wasm_runtime_init();
    }

    let mut native_symbols = native_symbol_list
        .into_iter()
        .map(|sym| NativeSymbol {
            symbol: CStr::from_bytes_until_nul(sym.symbol.as_bytes())
                .unwrap()
                .as_ptr(),
            func_ptr: sym.func_ptr,
            signature: CStr::from_bytes_until_nul(sym.signature.as_bytes())
                .unwrap()
                .as_ptr(),
            attachment: std::ptr::null_mut(),
        })
        .collect::<Vec<_>>();

    // Register native symbols
    let env = CString::new("env").unwrap();
    unsafe {
        wasm_runtime_register_natives(
            env.as_ptr(),
            native_symbols.as_mut_ptr(),
            native_symbols.len() as _,
        );
    }
}

pub struct PcdApp {
    module: wasm_module_t,
    buf: Vec<u8>,
    stack_size: u32,
    heap_size: u32,
}

pub struct PcdAppRuntime {
    app: PcdApp,
    instance: wasm_module_inst_t,
}

impl PcdApp {
    pub fn pcd_app_load(stack_size: u32, heap_size: u32, path: &PathBuf) -> WasrResult<Self> {
        let mut module_content = fs::read(path)?;
        let mut error_buf = [0i8; ERR_BUF_LEN];

        let module = unsafe {
            wasm_runtime_load(
                module_content.as_mut_ptr(),
                module_content.len() as _,
                error_buf.as_mut_ptr(),
                error_buf.len() as _,
            )
        };

        if module.is_null() {
            return Err(RuntimeError::InitializationFailure);
        }

        Ok(Self {
            module,
            buf: module_content,
            stack_size,
            heap_size,
        })
    }
}

impl Drop for PcdApp {
    fn drop(&mut self) {
        unsafe {
            wasm_runtime_unload(self.module);
        }
    }
}

impl PcdAppRuntime {
    pub fn from_pcd_app(app: PcdApp, dir_allow_list: &[&str]) -> WasrResult<Self> {
        if app.module.is_null() {
            return Err(RuntimeError::InitializationFailure);
        }

        unsafe {
            wasm_runtime_set_wasi_args(
                app.module,
                dir_allow_list.as_ptr() as _,
                dir_allow_list.len() as _,
                std::ptr::null_mut(),
                0, // map_dir
                std::ptr::null_mut(),
                0, // env
                std::ptr::null_mut(),
                0, // argv, argc. Not set here
            );
        }

        let mut error_buf = [0 as c_char; ERR_BUF_LEN];
        let instance = unsafe {
            wasm_runtime_instantiate(
                app.module,
                app.stack_size,
                app.heap_size,
                error_buf.as_mut_ptr() as _,
                error_buf.len() as _,
            )
        };

        if instance.is_null() {
            return Err(RuntimeError::InitializationFailure);
        }

        Ok(Self { app, instance })
    }

    /// Executes a specified function within the WASM runtime.
    ///
    /// # Arguments
    ///
    /// * `func` - The name of the function to execute.
    /// * `args` - The arguments to pass to the function.
    ///
    /// # Returns
    ///
    /// Returns a `WasrResult` indicating the success or failure of the function execution.
    ///
    /// # Errors
    ///
    /// This function will return an error if the function cannot be executed within the WASM runtime.
    pub fn pcd_runtime_execute_function(
        &self,
        func_name: &str,
        args: &mut [u32],
    ) -> WasrResult<()> {
        let c_func_name = CString::new(func_name).map_err(|e| {
            RuntimeError::InstantiationFailure(format!("Failed to convert func_name: {}", e))
        })?;

        // We first look up the function by name.
        let func = unsafe { wasm_runtime_lookup_function(self.instance, c_func_name.as_ptr()) };
        if func.is_null() {
            return Err(RuntimeError::FunctionNotFound);
        }

        let exec_env = unsafe { wasm_runtime_create_exec_env(self.instance, self.app.stack_size) };
        if exec_env.is_null() {
            return Err(RuntimeError::InstantiationFailure(
                "Failed to create exec env".to_string(),
            ));
        }

        unsafe {
            if !wasm_runtime_call_wasm(exec_env, func, args.len() as _, args.as_mut_ptr()) {
                return Err(RuntimeError::ExecutionError(ExecError {
                    message: CString::from_raw(wasm_runtime_get_exception(self.instance) as _)
                        .to_str()
                        .unwrap()
                        .to_string(),
                    exit_code: 1,
                }));
            }
            wasm_runtime_destroy_exec_env(exec_env);
        }

        Ok(())
    }
}

impl Drop for PcdAppRuntime {
    fn drop(&mut self) {
        unsafe {
            wasm_runtime_deinstantiate(self.instance);
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_pcd_wasm_runtime() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("../test/gcd_wasm32_wasi.wasm");

        // Initialize environment
        pcd_env_init(&[]);

        // Load app
        let app = PcdApp::pcd_app_load(1024 * 6, 1024 * 6, &d).unwrap();
        // Create runtime
        let runtime = PcdAppRuntime::from_pcd_app(app, &[]).unwrap();

        // Execute function
        let mut args = [10, 20];
        runtime
            .pcd_runtime_execute_function("gcd", &mut args)
            .unwrap();
    }
}
