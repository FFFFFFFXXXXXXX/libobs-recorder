fn main() {
    // always link libobs
    println!(
        "cargo:rustc-link-search=native={}",
        env!("CARGO_MANIFEST_DIR")
    );
    println!("cargo:rustc-link-lib=obs");

    // only generate bindings if they don't exist yet
    let bindings_path =
        std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bindings.rs"));
    if let Ok(metadata) = bindings_path.metadata() {
        if metadata.len() > 0 {
            return;
        }
    }

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
