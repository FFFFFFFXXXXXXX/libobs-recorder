use std::{env, path};

use fs_extra::dir;

const NEWEST_VERSION: &str = "30.0.2";
pub const VERSION: &str = {
    if let Some(version) = option_env!("LIBOBS_RECORDER_VERSION") {
        version
    } else {
        NEWEST_VERSION
    }
};

pub type Error = Box<dyn std::error::Error>;

pub fn build() -> Result<(), Error> {
    build_internal(None::<&path::Path>)
}

pub fn build_to_path(path: impl AsRef<path::Path>) -> Result<(), Error> {
    build_internal(Some(path))
}

fn build_internal(path: Option<impl AsRef<path::Path>>) -> Result<(), Error> {
    // compile time
    let this_crate_dir = path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // run time;
    let consumer_crate_output_dir = match path {
        Some(path) => path::PathBuf::from(path.as_ref()),
        None => get_cargo_target_dir()?,
    };

    let bin_res_dir = this_crate_dir.join(format!("libobs_{}", VERSION));
    dir::copy(
        bin_res_dir,
        consumer_crate_output_dir.join("libobs"),
        &dir::CopyOptions::new().overwrite(true).content_only(true),
    )?;

    Ok(())
}

// Credit: https://github.com/rust-lang/cargo/issues/9661#issuecomment-1722358176
fn get_cargo_target_dir() -> Result<path::PathBuf, Error> {
    let out_dir = path::PathBuf::from(env::var("OUT_DIR")?);
    let profile = env::var("PROFILE")?;
    let mut target_dir = None;
    let mut sub_path = out_dir.as_path();
    while let Some(parent) = sub_path.parent() {
        if parent.ends_with(&profile) {
            target_dir = Some(parent);
            break;
        }
        sub_path = parent;
    }
    let target_dir = target_dir.ok_or("not found")?;
    Ok(target_dir.to_path_buf())
}
