fn main() {
    // link obs.dll
    println!("cargo:rustc-link-search=native={}", env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rustc-link-lib=obs_{}", build_helper::VERSION);

    let bindings_file = format!("bindings_{}.rs", build_helper::VERSION);
    println!("cargo:rustc-env=LIBOBS_BINDINGS_FILE={bindings_file}");
    println!("cargo:rustc-env=LIBOBS_BINDINGS_VERSION={}", build_helper::VERSION);

    #[cfg(feature = "bindgen")]
    bindgen::builder()
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
        .expect("Error generating bindings")
        .write_to_file(format!("{}/src/{bindings_file}", env!("CARGO_MANIFEST_DIR")))
        .expect("Error outputting bindings");
}
