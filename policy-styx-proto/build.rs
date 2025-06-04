fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=proto/types.proto");
    println!("cargo:rerun-if-changed=proto/handlers.proto");

    tonic_build::configure()
        .out_dir("src")
        .compile_protos(&["proto/types.proto", "proto/handlers.proto"], &["proto"])
        .expect("Failed to compile proto files");
}
