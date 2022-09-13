extern crate bindgen;

fn main() {
    println!(
        "cargo:rustc-link-search=native={}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    );
    println!("cargo:rustc-link-lib=obs");

    let bindings = bindgen::builder()
        .header("libobs_headers/obs.h")
        .blacklist_type("_bindgen_ty_1")
        .blacklist_type("_bindgen_ty_2")
        .generate()
        .expect("Error generating bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Error outputting bindings");
}
