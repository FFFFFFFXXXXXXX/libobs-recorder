fn main() {
    // always link libobs
    println!("cargo:rustc-link-search=native={}", env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rustc-link-lib=obs_{}", build_helper::VERSION);

    #[cfg(feature = "bindgen")]
    gen_bindings();
}

#[cfg(feature = "bindgen")]
fn gen_bindings() {
    let bindings_path = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bindings.rs"));

    let bindings = bindgen::builder()
        .header(format!("libobs_headers_{}/obs.h", build_helper::VERSION))
        .blocklist_function("_+.*")
        .derive_copy(true)
        .derive_debug(true)
        .derive_default(true)
        .derive_partialeq(true)
        .derive_eq(true)
        .derive_partialord(true)
        .derive_ord(true)
        .layout_tests(false)
        .merge_extern_blocks(true)
        .generate()
        .expect("Error generating bindings");

    bindings
        .write_to_file(bindings_path)
        .expect("Error outputting bindings");
}
