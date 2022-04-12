extern crate bindgen;

fn main() {
    println!("cargo:rustc-link-search=native=./");
    println!("cargo:rustc-link-lib=obs");

    let bindings = bindgen::builder()
        .header("wrapper.h")
        .blacklist_type("_bindgen_ty_2")
        .generate()
        .expect("Unable to generate libOBS bindings. Do you have OBS >= 23.0.0 installed?");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Unable to write bindings in the directory");
}
