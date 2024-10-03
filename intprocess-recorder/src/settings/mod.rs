pub use adapter::{Adapter, AdapterId, AdapterType};
pub use audio::AudioSource;
pub use encoders::Encoder;
pub use framerate::Framerate;
pub use rate_control::RateControl;
pub use resolution::{Resolution, StdResolution};
pub use window::Window;

mod adapter;
mod audio;
mod encoders;
mod framerate;
mod rate_control;
mod resolution;
mod window;

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct RecorderSettings {
    /// ID of GPU
    pub(crate) window: Window,
    pub(crate) input_resolution: Resolution,
    pub(crate) output_resolution: Resolution,
    pub(crate) output_path: String,
    pub(crate) framerate: Option<Framerate>,
    pub(crate) rate_control: Option<RateControl>,
    pub(crate) audio_source: Option<AudioSource>,
    pub(crate) encoder: Option<Encoder>,
}

impl RecorderSettings {
    pub fn new(
        window: Window,
        input_resolution: impl Into<Resolution>,
        output_resolution: impl Into<Resolution>,
        output_path: impl AsRef<std::path::Path>,
    ) -> Self {
        let input_resolution = input_resolution.into();
        let output_resolution = output_resolution.into();
        let output_path = output_path
            .as_ref()
            .to_str()
            .expect("expected unicode path")
            .to_string();

        Self {
            window,
            input_resolution,
            output_resolution,
            output_path,
            framerate: None,
            rate_control: None,
            audio_source: None,
            encoder: None,
        }
    }

    pub fn set_window(&mut self, window: Window) {
        self.window = window;
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn set_input_resolution(&mut self, size: impl Into<Resolution>) {
        self.input_resolution = size.into();
    }

    pub fn get_input_resolution(&self) -> &Resolution {
        &self.input_resolution
    }

    pub fn set_output_resolution(&mut self, resolution: impl Into<Resolution>) {
        self.output_resolution = resolution.into();
    }

    pub fn get_output_resolution(&self) -> &Resolution {
        &self.output_resolution
    }

    pub fn set_output_path(&mut self, output_path: impl Into<String>) {
        self.output_path = output_path.into();
    }

    pub fn get_output_path(&self) -> &str {
        self.output_path.as_str()
    }

    pub fn set_framerate(&mut self, framerate: Framerate) {
        self.framerate = Some(framerate);
    }

    pub fn get_framerate(&self) -> Option<&Framerate> {
        self.framerate.as_ref()
    }

    pub fn set_rate_control(&mut self, rate_control: RateControl) {
        self.rate_control = Some(rate_control);
    }

    pub fn get_rate_control(&self) -> Option<&RateControl> {
        self.rate_control.as_ref()
    }

    pub fn set_audio_source(&mut self, record_audio: AudioSource) {
        self.audio_source = Some(record_audio);
    }

    pub fn get_audio_source(&self) -> Option<&AudioSource> {
        self.audio_source.as_ref()
    }

    pub fn set_encoder(&mut self, encoder: Encoder) {
        self.encoder = Some(encoder);
    }

    pub fn get_encoder(&self) -> Option<&Encoder> {
        self.encoder.as_ref()
    }
}
