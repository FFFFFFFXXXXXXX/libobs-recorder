#[cfg(feature = "full")]
mod recorder;
pub mod settings;

#[cfg(feature = "full")]
pub use recorder::InpRecorder;
