//! This is the Rust implementation of `transformer_middleware` thing.
//! 
//! So this should go enter a dead loop.

use std::path::PathBuf;

use clap::{command, Parser, ValueEnum};
use policy_styx_rt::{pcd_env_init, PcdApp, PcdAppRuntime, PcdNativeSymbol, WasrResult};
use policy_styx_sys::policy::{pcd_get_output_custodian, pcd_get_program_hash};

#[derive(ValueEnum, Debug, Clone)]
enum PcdAppName {
    PolarsDemo,
    PolicyEngine,
}

#[derive(Parser, Debug)]
#[command(name = "policy-styx-runner", version, about, author)]
struct Args {
    #[clap(
        short,
        long,
        default_value = "policy-styx.wasm",
        help = "Path to where the WASM module lives"
    )]
    path: String,
    #[clap(short, long, default_value = "8192", help = "Size of the stack")]
    stack_size: u32,
    #[clap(long, default_value = "8192", help = "Size of the heap")]
    heap_size: u32,
    #[clap(
        long,
        default_value = "PolicyEngine",
        help = "Name of the policy engine"
    )]
    app_name: PcdAppName,
}

const PCD_POLICY_ENGINE_NATIVE_SYMBOLS: &[&PcdNativeSymbol] = &[
    &PcdNativeSymbol {
        symbol: "pcd_get_program_hash\0",
        func_ptr: pcd_get_program_hash as _,
        signature: "(*)i\0",
    },
    &PcdNativeSymbol {
        symbol: "pcd_get_output_custodian\0",
        func_ptr: pcd_get_output_custodian as _,
        signature: "(*)i\0",
    },
];

const PCD_POLICY_ENGINE_WASM_NAME: &str = "policy-styx.wasm";
const POLARS_WASM_NAME: &str = "polars_demo.wasm";
const POLARS_ENTRY: &str = "polars_demo";

/// Launches the policy engine.
fn launch_policy_engine(stack_size: u32, heap_size: u32, path: &str) -> WasrResult<()> {
    // First let us initialize the policy-styx runtime environment.
    pcd_env_init(PCD_POLICY_ENGINE_NATIVE_SYMBOLS);

    let mut path = PathBuf::from(path);
    path.push(PCD_POLICY_ENGINE_WASM_NAME);

    // Load app
    let app = PcdApp::pcd_app_load(stack_size, heap_size, &path)?;
    // Create runtime
    let runtime = PcdAppRuntime::from_pcd_app(app, &[])?;

    Ok(())
}

fn launch_polars_demo(stack_size: u32, heap_size: u32, path: &str) -> WasrResult<()> {
    pcd_env_init(PCD_POLICY_ENGINE_NATIVE_SYMBOLS);

    let mut path = PathBuf::from(path);
    path.push(POLARS_WASM_NAME);

    let app = PcdApp::pcd_app_load(stack_size, heap_size, &path)?;
    let runtime = PcdAppRuntime::from_pcd_app(app, &[])?;

    runtime.pcd_runtime_execute_function(POLARS_ENTRY, &mut [])?;
    
    // Main service logic: handle requests; maybe we should launch a server here?

    Ok(())
}

fn main() -> WasrResult<()> {
    let args = Args::parse();

    match args.app_name {
        PcdAppName::PolicyEngine => {
            launch_policy_engine(args.stack_size, args.heap_size, &args.path)?
        },
        PcdAppName::PolarsDemo => launch_polars_demo(args.stack_size, args.heap_size, &args.path)?,
    }

    loop {
        // Enter a dead loop and wait for the input.
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match input.trim() {
            "exit" => break,
            _ => println!("Unknown command: {}", input),
        }
    }

    Ok(())
}
