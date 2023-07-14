pub mod settings {
    pub use intprocess_recorder::settings::{
        AudioSource, Encoder, Framerate, RateControl, Resolution, Size, Window,
    };
}

pub use intprocess_recorder::{settings::RecorderSettings, InpRecorder as SingletonRecorder};
use ipc_link::{IpcCommand, IpcLinkMaster};

#[cfg(target_family = "windows")]
const EXECUTABLE: &str = "extprocess_recorder.exe";
#[cfg(target_family = "unix")]
const EXECUTABLE: &str = "extprocess_recorder";

pub struct Recorder {
    recorder: IpcLinkMaster,
}

impl Recorder {
    pub fn new(
        libobs_data_path: Option<&str>,
        plugin_bin_path: Option<&str>,
        plugin_data_path: Option<&str>,
    ) -> std::io::Result<Self> {
        let mut rec = IpcLinkMaster::new(EXECUTABLE)?;

        let cmd = IpcCommand::Init {
            libobs_data_path: libobs_data_path.map(|s| s.to_string()),
            plugin_bin_path: plugin_bin_path.map(|s| s.to_string()),
            plugin_data_path: plugin_data_path.map(|s| s.to_string()),
        };

        if rec.send(cmd) {
            Ok(Self { recorder: rec })
        } else {
            Err(std::io::ErrorKind::Other.into())
        }
    }

    pub fn configure(&mut self, settings: &RecorderSettings) -> bool {
        self.recorder
            .send(ipc_link::IpcCommand::Settings(settings.clone()))
    }

    pub fn start_recording(&mut self) -> bool {
        self.recorder.send(ipc_link::IpcCommand::StartRecording)
    }

    pub fn stop_recording(&mut self) -> bool {
        self.recorder.send(ipc_link::IpcCommand::StopRecording)
    }

    pub fn shutdown(mut self) -> Result<(), Self> {
        self.recorder.send(ipc_link::IpcCommand::Shutdown);
        self.recorder.send(ipc_link::IpcCommand::Exit);

        Ok(())
    }
}
