pub use intprocess_recorder::InpRecorder as SingletonRecorder;
pub use intprocess_recorder::{
    AudioSource, Encoder, Framerate, RateControl, RecorderSettings, Resolution, Size, Window,
};

use ipc_link::{IpcCommand, IpcLinkMaster};

#[cfg(target_family = "windows")]
const EXECUTABLE: &str = "extprocess_recorder.exe";
#[cfg(target_family = "unix")]
const EXECUTABLE: &str = "extprocess_recorder";

pub struct Recorder {
    recorder: Option<IpcLinkMaster>,
}

impl Recorder {
    pub fn new() -> Self {
        Self { recorder: None }
    }

    pub fn init(&mut self) -> bool {
        let mut rec = IpcLinkMaster::new(EXECUTABLE);
        let response = rec.send(IpcCommand::Init {
            libobs_data_path: None,
            plugin_bin_path: None,
            plugin_data_path: None,
        });
        self.recorder = Some(rec);

        response
    }

    pub fn configure(&mut self, settings: &RecorderSettings) -> bool {
        if let Some(recorder) = self.recorder.as_mut() {
            recorder.send(ipc_link::IpcCommand::Settings(settings.clone()))
        } else {
            false
        }
    }

    pub fn start_recording(&mut self) -> bool {
        if let Some(recorder) = self.recorder.as_mut() {
            recorder.send(ipc_link::IpcCommand::StartRecording)
        } else {
            false
        }
    }

    pub fn stop_recording(&mut self) -> bool {
        if let Some(recorder) = self.recorder.as_mut() {
            recorder.send(ipc_link::IpcCommand::StopRecording)
        } else {
            false
        }
    }

    pub fn shutdown(&mut self) -> bool {
        let success = if let Some(recorder) = self.recorder.as_mut() {
            recorder.send(ipc_link::IpcCommand::Shutdown)
        } else {
            false
        };

        if success {
            drop(self.recorder.take());
        }

        success
    }
}
