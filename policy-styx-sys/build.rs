fn main() {
    let output = cmake::Config::new("csrc").build();

    println!("cargo:rustc-link-search=native={}", output.display());
    println!("cargo:rustc-link-lib=dylib=pcd");
}
