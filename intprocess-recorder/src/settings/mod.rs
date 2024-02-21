pub use self::{
    audio::AudioSource,
    encoders::Encoder,
    framerate::Framerate,
    rate_control::RateControl,
    resolution::{Resolution, StdResolution},
    window::Window,
};

mod audio;
mod encoders;
mod framerate;
mod rate_control;
mod resolution;
mod window;

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Default)]
pub struct RecorderSettings {
    pub(crate) window: Option<Window>,
    pub(crate) input_resolution: Option<Resolution>,
    pub(crate) output_resolution: Option<Resolution>,
    pub(crate) framerate: Option<Framerate>,
    pub(crate) rate_control: Option<RateControl>,
    pub(crate) record_audio: Option<AudioSource>,
    pub(crate) output_path: Option<String>,
    pub(crate) encoder: Option<Encoder>,
}

impl RecorderSettings {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_window(&mut self, window: Window) {
        self.window = Some(window);
    }

    pub fn set_input_resolution(&mut self, size: impl Into<Resolution>) {
        self.input_resolution = Some(size.into());
    }

    pub fn set_output_resolution(&mut self, resolution: impl Into<Resolution>) {
        self.output_resolution = Some(resolution.into());
    }

    pub fn set_framerate(&mut self, framerate: Framerate) {
        self.framerate = Some(framerate);
    }

    pub fn set_rate_control(&mut self, rate_control: RateControl) {
        self.rate_control = Some(rate_control);
    }

    pub fn record_audio(&mut self, record_audio: AudioSource) {
        self.record_audio = Some(record_audio);
    }

    pub fn set_output_path(&mut self, output_path: impl Into<String>) {
        self.output_path = Some(output_path.into());
    }

    pub fn set_encoder(&mut self, encoder: Encoder) {
        self.encoder = Some(encoder);
    }
}
