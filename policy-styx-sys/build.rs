fn main() {
    let output = cmake::Config::new("csrc")
        .define("PCD_CONFIG_RUNTIME_WAMR", "1")
        .define("PCD_CONFIG_CRYPTO_AES_GCM", "1")
        .build();

    println!("cargo:rustc-link-search=native={}", output.display());
    println!("cargo:rustc-link-lib=dylib=pcd");
}
