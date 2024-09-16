pub use self::{
    adapter::{Adapter, AdapterId},
    audio::AudioSource,
    encoders::Encoder,
    framerate::Framerate,
    rate_control::RateControl,
    resolution::{Resolution, StdResolution},
    window::Window,
};

mod adapter;
mod audio;
mod encoders;
mod framerate;
mod rate_control;
mod resolution;
mod window;

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Default)]
pub struct RecorderSettings {
    /// ID of GPU
    pub(crate) adapter_id: Option<AdapterId>,
    pub(crate) window: Option<Window>,
    pub(crate) input_resolution: Option<Resolution>,
    pub(crate) output_resolution: Option<Resolution>,
    pub(crate) framerate: Option<Framerate>,
    pub(crate) rate_control: Option<RateControl>,
    pub(crate) audio_source: Option<AudioSource>,
    pub(crate) output_path: Option<String>,
    pub(crate) encoder: Option<Encoder>,
}

impl RecorderSettings {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_adapter_id(&mut self, adapter_id: AdapterId) {
        self.adapter_id = Some(adapter_id);
    }

    pub fn get_adapter_id(&self) -> Option<&AdapterId> {
        self.adapter_id.as_ref()
    }

    pub fn set_window(&mut self, window: Window) {
        self.window = Some(window);
    }

    pub fn get_window(&self) -> Option<&Window> {
        self.window.as_ref()
    }

    pub fn set_input_resolution(&mut self, size: impl Into<Resolution>) {
        self.input_resolution = Some(size.into());
    }

    pub fn get_input_resolution(&self) -> Option<&Resolution> {
        self.input_resolution.as_ref()
    }

    pub fn set_output_resolution(&mut self, resolution: impl Into<Resolution>) {
        self.output_resolution = Some(resolution.into());
    }

    pub fn get_output_resolution(&self) -> Option<&Resolution> {
        self.output_resolution.as_ref()
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

    pub fn set_output_path(&mut self, output_path: impl Into<String>) {
        self.output_path = Some(output_path.into());
    }

    pub fn get_output_path(&self) -> Option<&str> {
        self.output_path.as_deref()
    }

    pub fn set_encoder(&mut self, encoder: Encoder) {
        self.encoder = Some(encoder);
    }

    pub fn get_encoder(&self) -> Option<&Encoder> {
        self.encoder.as_ref()
    }
}
