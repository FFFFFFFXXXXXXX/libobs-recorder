#[cfg(feature = "full")]
mod recorder;
mod settings;

#[cfg(feature = "full")]
pub use recorder::InpRecorder;

pub use settings::{
    audio::AudioSource,
    encoders::Encoder,
    framerate::Framerate,
    rate_control::RateControl,
    window::{Resolution, Size, Window},
    RecorderSettings,
};
