use intprocess_recorder::settings::Encoder;
pub use intprocess_recorder::{settings::RecorderSettings, InpRecorder as SingletonRecorder};
use ipc_link::{IpcCommand, IpcLinkMaster, IpcResponse};

pub mod settings {
    pub use intprocess_recorder::settings::{AudioSource, Encoder, Framerate, RateControl, Resolution, Size, Window};
}

#[cfg(target_family = "windows")]
const EXECUTABLE: &str = "./extprocess_recorder.exe";
#[cfg(target_family = "unix")]
const EXECUTABLE: &str = "./extprocess_recorder";

#[derive(Debug)]
pub struct Recorder {
    recorder: IpcLinkMaster,
}

impl Recorder {
    pub fn new(enable_logging: bool) -> Result<Self> {
        Self::new_with_paths(None, None, None, None, enable_logging)
    }

    pub fn new_with_paths(
        executable_path: Option<&std::path::Path>,
        libobs_data_path: Option<&str>,
        plugin_bin_path: Option<&str>,
        plugin_data_path: Option<&str>,
        enable_logging: bool,
    ) -> Result<Self> {
        let mut rec = IpcLinkMaster::new(
            executable_path.unwrap_or(std::path::Path::new(EXECUTABLE)),
            enable_logging,
        )
        .map_err(Error::IoError)?;

        let cmd = IpcCommand::Init {
            libobs_data_path: libobs_data_path.map(ToString::to_string),
            plugin_bin_path: plugin_bin_path.map(ToString::to_string),
            plugin_data_path: plugin_data_path.map(ToString::to_string),
        };

        match rec.send(cmd) {
            IpcResponse::Ok => Ok(Self { recorder: rec }),
            IpcResponse::Err(e) => Err(Error::RecorderError(e)),
            IpcResponse::Encoders { .. } => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn configure(&mut self, settings: &RecorderSettings) -> Result<()> {
        match self.recorder.send(IpcCommand::Configure(settings.clone())) {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Err(e) => Err(Error::RecorderError(e)),
            IpcResponse::Encoders { .. } => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn available_encoders(&mut self) -> Result<Vec<Encoder>> {
        match self.recorder.send(IpcCommand::Encoders) {
            IpcResponse::Encoders { available, .. } => Ok(available),
            IpcResponse::Err(e) => Err(Error::RecorderError(e)),
            IpcResponse::Ok => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn selected_encoder(&mut self) -> Result<Encoder> {
        match self.recorder.send(IpcCommand::Encoders) {
            IpcResponse::Encoders { selected, .. } => Ok(selected),
            IpcResponse::Err(e) => Err(Error::RecorderError(e)),
            IpcResponse::Ok => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn start_recording(&mut self) -> Result<()> {
        match self.recorder.send(IpcCommand::StartRecording) {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Err(e) => Err(Error::RecorderError(e)),
            IpcResponse::Encoders { .. } => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn stop_recording(&mut self) -> Result<()> {
        match self.recorder.send(IpcCommand::StopRecording) {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Err(e) => Err(Error::RecorderError(e)),
            IpcResponse::Encoders { .. } => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn shutdown(mut self) -> Result<()> {
        match self.recorder.send(IpcCommand::Shutdown) {
            IpcResponse::Ok => { /* OK continue */ }
            IpcResponse::Err(e) => return Err(Error::ShutdownFailed(self, e)),
            IpcResponse::Encoders { .. } => return Err(Error::ShouldNeverHappenNotifyMe),
        }
        match self.recorder.send(IpcCommand::Exit) {
            IpcResponse::Ok => {
                self.recorder.drain_logs();
                Ok(())
            }
            IpcResponse::Err(e) => Err(Error::ShutdownFailed(self, e)),
            IpcResponse::Encoders { .. } => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    RecorderError(String),
    ShutdownFailed(Recorder, String),
    ShouldNeverHappenNotifyMe,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(e) => f.write_fmt(format_args!("{e:?}")),
            Error::RecorderError(e) | Error::ShutdownFailed(_, e) => f.write_str(e),
            Error::ShouldNeverHappenNotifyMe => f.write_str("This error should never happen - please notify me"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IoError(e) => Some(e),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
