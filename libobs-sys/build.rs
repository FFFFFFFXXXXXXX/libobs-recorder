extern crate bindgen;

fn main() {
    let bindings = bindgen::builder()
        .header("libobs_headers/obs.h")
        .blacklist_type("_bindgen_ty_2")
        .generate()
        .expect("Error generating bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Error outputting bindings");
}
