const CURRENT_VERSION: &str = "30.0.2";

fn main() {
    let version = std::env::var("LIBOBS_RECORDER_VERSION").unwrap_or(CURRENT_VERSION.to_string());

    // always link libobs
    println!("cargo:rustc-link-search=native={}", env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rustc-link-lib=obs_{version}");

    #[cfg(feature = "bindgen")]
    gen_bindings(version);
}

#[cfg(feature = "bindgen")]
fn gen_bindings(version: String) {
    let bindings_path = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bindings.rs"));

    let bindings = bindgen::builder()
        .header(format!("libobs_headers_{version}/obs.h"))
        .blocklist_type("_bindgen_ty_1")
        .blocklist_type("_bindgen_ty_2")
        .generate()
        .expect("Error generating bindings");

    bindings
        .write_to_file(bindings_path)
        .expect("Error outputting bindings");
}
