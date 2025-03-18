use std::fmt::{Display, Formatter};
use std::{env, error, fs, path};

use fs_extra::dir;

const NEWEST_VERSION: &str = "31.0.2";
pub const VERSION: &str = {
    if let Some(version) = option_env!("LIBOBS_RECORDER_VERSION") {
        version
    } else {
        NEWEST_VERSION
    }
};

pub type Error = Box<dyn std::error::Error>;

#[derive(Debug)]
struct NoExtprocessRecorder;
impl error::Error for NoExtprocessRecorder {}
impl Display for NoExtprocessRecorder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("CARGO_BIN_FILE_LIBOBS_RECORDER_extprocess_recorder environment variable not found! Make sure you correctly added the artifact-dependency to your 'Cargo.toml' file")
    }
}

pub fn build() -> Result<(), Error> {
    build_internal(None)
}

pub fn build_to_path(path: impl AsRef<path::Path>) -> Result<(), Error> {
    build_internal(Some(path.as_ref()))
}

fn build_internal(path: Option<&path::Path>) -> Result<(), Error> {
    // compile time
    let this_crate_dir = path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // run time;
    let consumer_crate_output_dir = match path {
        Some(path) => path::PathBuf::from(path),
        None => get_cargo_target_dir()?,
    };

    let bin_res_dir = this_crate_dir.join(format!("libobs_{}", VERSION));
    let target_path = consumer_crate_output_dir.join("libobs");

    fs::create_dir_all(target_path.parent().unwrap())?;

    let copy_options = dir::CopyOptions::new().overwrite(true).content_only(true);
    dir::copy(bin_res_dir, target_path, &copy_options)?;

    Ok(())
}

pub fn copy_artifact_dependency() -> Result<(), Error> {
    copy_artifact_dependency_internal(None)
}

pub fn copy_artifact_dependency_to_path(path: impl AsRef<path::Path>) -> Result<(), Error> {
    copy_artifact_dependency_internal(Some(path.as_ref()))
}

fn copy_artifact_dependency_internal(path: Option<&path::Path>) -> Result<(), Error> {
    let consumer_crate_output_dir = match path {
        Some(path) => path::PathBuf::from(path),
        None => get_cargo_target_dir()?,
    };

    let artifact_path =
        env::var_os("CARGO_BIN_FILE_LIBOBS_RECORDER_extprocess_recorder").ok_or(NoExtprocessRecorder)?;
    let target_path = consumer_crate_output_dir.join("libobs/extprocess_recorder.exe");

    fs::create_dir_all(target_path.parent().unwrap())?;
    fs::copy(artifact_path, target_path)?;

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
