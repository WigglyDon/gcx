fn main() {
    println!("cargo:rerun-if-env-changed=GCX_ENGINE_LIB_DIR");
    if std::env::var_os("CARGO_FEATURE_NATIVE_ENGINE").is_some() {
        let path = std::env::var("GCX_ENGINE_LIB_DIR")
            .expect("set GCX_ENGINE_LIB_DIR to the built gcx_engine library directory");
        println!("cargo:rustc-link-search=native={path}");
        println!("cargo:rustc-link-lib=dylib=gcx_engine");
    }
}
