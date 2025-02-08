use std::process::Command;

const WASI_OPENSSL_PATH: &str = "/opt/openssl-wasm";

fn main() {
    // Change the current working directory to the `csrc` directory
    std::env::set_current_dir("csrc").unwrap();
    if !std::path::Path::new("build").exists() {
        std::fs::create_dir("build").unwrap();
    }

    // Run the `cmake` command
    let output = Command::new("cmake")
        .arg("-S")
        .arg(".")
        .arg("-B")
        .arg("build")
        .arg("-DCMAKE_CXX_COMPILER=emcc")
        .arg("-DCMAKE_C_COMPILER=emcc")
        .arg("-DCMAKE_CXX_FLAGS=-I/opt/openssl-wasm/include -s STANDALONE_WASM")
        .arg("-DCMAKE_C_FLAGS=-I/opt/openssl-wasm/include -s STANDALONE_WASM")
        .output()
        .expect("Failed to run cmake");

    // Print the output of the `cmake` command
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));

    // Print the error of the `cmake` command
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    println!("cargo:rustc-link-search=native={}/csrc/build", env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rustc-link-lib=static=pcd");
}
