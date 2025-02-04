//! This is the Rust implementation of `transformer_middleware` thing.
//!
//! So this should go enter a dead loop.

use std::path::PathBuf;

use policy_styx_rt::{pcd_env_init, PcdApp, PcdAppRuntime, RuntimeError, WasrResult};
use styx_runner::StyxRunner;

mod styx_runner;

// const PCD_POLICY_ENGINE_WASM_NAME: &str = "policy-styx.wasm";
// const POLARS_WASM_NAME: &str = "polars_demo.wasm";
const POLARS_ENTRY: &str = "polars_demo";

fn pcd_load_app(stack_size: u32, heap_size: u32, path: &str) -> WasrResult<PcdAppRuntime> {
    let path = PathBuf::from(path);

    // Load app
    let app = PcdApp::pcd_app_load(stack_size, heap_size, &path)?;
    // Create runtime
    let runtime = PcdAppRuntime::from_pcd_app(app, &[])?;

    Ok(runtime)
}

fn main() -> WasrResult<()> {
    pcd_env_init();

    println!("Please load the policy engine.");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let args = input.trim().split_whitespace().collect::<Vec<_>>();

    if args.len() < 3 {
        println!("Usage: [path] [stack size] [heap size]");
        return Ok(());
    }

    let stack_size = args[1]
        .parse()
        .map_err(|_| RuntimeError::CompilationError("Stack size invalid".into()))?;
    let heap_size = args[2]
        .parse()
        .map_err(|_| RuntimeError::CompilationError("Heap size invalid".into()))?;

    let runtime = pcd_load_app(stack_size, heap_size, args[0])?;
    println!("Policy engine loaded at {}", args[0]);

    let mut styx_runner = StyxRunner::new(runtime);

    println!("Please now load the sandboxed application.");
    std::io::stdin().read_line(&mut input)?;
    let args = input.trim().split_whitespace().collect::<Vec<_>>();

    if args.len() < 3 {
        println!("Usage: [path] [stack size] [heap size]");
        return Ok(());
    }

    let stack_size = args[1]
        .parse()
        .map_err(|_| RuntimeError::CompilationError("Stack size invalid".into()))?;
    let heap_size = args[2]
        .parse()
        .map_err(|_| RuntimeError::CompilationError("Heap size invalid".into()))?;

    let runtime = pcd_load_app(stack_size, heap_size, args[0])?;
    styx_runner.load_app(POLARS_ENTRY, runtime);

    println!("Sandboxed application loaded at {}", args[0]);

    Ok(())
}
