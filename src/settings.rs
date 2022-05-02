use crate::{
    encoder::Encoder, framerate::Framerate, rate_control::*, resolution::Resolution, window::Window,
};

#[derive(Clone, Debug, PartialEq)]
pub struct RecorderSettings {
    pub(crate) window: Option<Window>,
    pub(crate) output_resolution: Option<Resolution>,
    pub(crate) framerate: Framerate,
    pub(crate) rate_control: RateControl,
    pub(crate) record_audio: bool,
    pub(crate) output_path: Option<String>,
    pub(crate) encoder: Option<Encoder>,
}

impl RecorderSettings {
    pub fn new() -> Self {
        RecorderSettings {
            window: None,
            output_resolution: None,
            framerate: Framerate::new(0, 0),
            rate_control: RateControl::default(),
            record_audio: true,
            output_path: None,
            encoder: None,
        }
    }
    pub fn set_window(&mut self, window: Window) {
        self.window = Some(window);
    }
    pub fn set_output_resolution(&mut self, resolution: Resolution) {
        self.output_resolution = Some(resolution);
    }
    pub fn set_framerate(&mut self, framerate: Framerate) {
        self.framerate = framerate;
    }
    pub fn set_cbr(&mut self, bitrate: Cbr) {
        self.rate_control.cbr = bitrate;
    }
    pub fn set_cqp(&mut self, cqp: Cqp) {
        self.rate_control.cqp = cqp;
    }
    pub fn set_icq(&mut self, icq: Icq) {
        self.rate_control.icq = icq;
    }
    pub fn record_audio(&mut self, record_audio: bool) {
        self.record_audio = record_audio;
    }
    pub fn set_output_path<S: Into<String>>(&mut self, output_path: S) {
        self.output_path = Some(output_path.into());
    }
    pub fn set_encoder(&mut self, encoder: Encoder) {
        self.encoder = Some(encoder);
    }
}
