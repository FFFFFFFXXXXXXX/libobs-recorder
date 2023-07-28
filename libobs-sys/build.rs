fn main() {
    // always link libobs
    println!("cargo:rustc-link-search=native={}", env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rustc-link-lib=obs");

    #[cfg(feature = "bindgen")]
    gen_bindings();
}

#[cfg(feature = "bindgen")]
fn gen_bindings() {
    let bindings_path = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bindings.rs"));

    let bindings = bindgen::builder()
        .header("libobs_headers/obs.h")
        .blocklist_type("_bindgen_ty_1")
        .blocklist_type("_bindgen_ty_2")
        .generate()
        .expect("Error generating bindings");

    bindings
        .write_to_file(bindings_path)
        .expect("Error outputting bindings");
}
