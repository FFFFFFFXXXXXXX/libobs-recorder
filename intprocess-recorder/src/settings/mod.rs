pub mod audio;
pub mod encoders;
pub mod framerate;
pub mod rate_control;
pub mod window;

use self::{
    encoders::Encoder,
    framerate::Framerate,
    window::{Resolution, Size, Window},
};

use self::{audio::AudioSource, rate_control::RateControl};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct RecorderSettings {
    pub(crate) window: Option<Window>,
    pub(crate) input_size: Option<Size>,
    pub(crate) output_resolution: Option<Resolution>,
    pub(crate) framerate: Option<Framerate>,
    pub(crate) rate_control: Option<RateControl>,
    pub(crate) record_audio: Option<AudioSource>,
    pub(crate) output_path: Option<String>,
    pub(crate) encoder: Option<Encoder>,
}

impl RecorderSettings {
    pub fn new() -> Self {
        RecorderSettings {
            window: None,
            input_size: None,
            output_resolution: None,
            framerate: None,
            rate_control: None,
            record_audio: None,
            output_path: None,
            encoder: None,
        }
    }
    pub fn set_window(&mut self, window: Window) {
        self.window = Some(window);
    }
    pub fn set_input_size(&mut self, size: Size) {
        self.input_size = Some(size);
    }
    pub fn set_output_resolution(&mut self, resolution: Resolution) {
        self.output_resolution = Some(resolution);
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
