use crate::{
    encoders::Encoder, framerate::Framerate, resolution::Resolution, Size, Window,
};

#[derive(Clone, Debug, PartialEq)]
pub struct RecorderSettings {
    pub(crate) window: Option<Window>,
    pub(crate) input_size: Option<Size>,
    pub(crate) output_resolution: Option<Resolution>,
    pub(crate) framerate: Framerate,
    pub(crate) rate_control: RateControl,
    pub(crate) record_audio: RecordAudio,
    pub(crate) output_path: Option<String>,
    pub(crate) encoder: Option<Encoder>,
}

impl RecorderSettings {
    pub fn new() -> Self {
        RecorderSettings {
            window: None,
            input_size: None,
            output_resolution: None,
            framerate: Framerate::new(0, 0),
            rate_control: RateControl::default(),
            record_audio: RecordAudio::APPLICATION,
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
        self.framerate = framerate;
    }
    pub fn set_rate_control(&mut self, rate_control: RateControl) {
        self.rate_control = rate_control;
    }
    pub fn record_audio(&mut self, record_audio: RecordAudio) {
        self.record_audio = record_audio;
    }
    pub fn set_output_path(&mut self, output_path: impl Into<String>) {
        self.output_path = Some(output_path.into());
    }
    pub fn set_encoder(&mut self, encoder: Encoder) {
        self.encoder = Some(encoder);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RecordAudio {
    NONE,
    APPLICATION,
    SYSTEM
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RateControl {
    CBR(u32),
    VBR(u32),
    CQP(u32),
    CRF(u32),
    ICQ(u32)
}

impl Default for RateControl {
    fn default() -> Self {
        Self::CQP(20)
    }
}
