pub use intprocess_recorder::{settings::RecorderSettings, InpRecorder as SingletonRecorder};
pub mod settings {
    pub use intprocess_recorder::settings::{
        AudioSource, Encoder, Framerate, RateControl, Resolution, StdResolution, Window,
    };
}

use intprocess_recorder::settings::Encoder;
use ipc_link::{IpcCommand, IpcLinkMaster, IpcResponse};

use std::{error, fmt, io, path};

#[cfg(target_family = "windows")]
const EXECUTABLE: &str = "./extprocess_recorder.exe";
#[cfg(target_family = "unix")]
const EXECUTABLE: &str = "./extprocess_recorder";

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Recorder(String),
    ShutdownFailed(Recorder, String),
    ShouldNeverHappenNotifyMe,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => f.write_fmt(format_args!("{e:?}")),
            Error::Recorder(e) | Error::ShutdownFailed(_, e) => f.write_str(e),
            Error::ShouldNeverHappenNotifyMe => f.write_str("This error should never happen - please notify me"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Recorder {
    recorder: IpcLinkMaster,
}

impl Recorder {
    pub fn new() -> Result<Self> {
        Self::new_with_paths(None::<&str>, None, None, None)
    }

    pub fn new_with_paths(
        executable_path: Option<impl AsRef<path::Path>>,
        libobs_data_path: Option<&str>,
        plugin_bin_path: Option<&str>,
        plugin_data_path: Option<&str>,
    ) -> Result<Self> {
        let executable_path = match &executable_path {
            Some(p) => p.as_ref(),
            None => path::Path::new(EXECUTABLE),
        };
        let mut rec = IpcLinkMaster::new(executable_path).map_err(Error::Io)?;

        let cmd = IpcCommand::Init {
            libobs_data_path: libobs_data_path.map(ToString::to_string),
            plugin_bin_path: plugin_bin_path.map(ToString::to_string),
            plugin_data_path: plugin_data_path.map(ToString::to_string),
        };

        match rec.send(cmd) {
            IpcResponse::Ok => Ok(Self { recorder: rec }),
            IpcResponse::Err(e) => Err(Error::Recorder(e)),
            _ => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn configure(&mut self, settings: &RecorderSettings) -> Result<()> {
        match self.recorder.send(IpcCommand::Configure(settings.clone())) {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Err(e) => Err(Error::Recorder(e)),
            _ => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn available_encoders(&mut self) -> Result<Vec<Encoder>> {
        match self.recorder.send(IpcCommand::Encoders) {
            IpcResponse::Encoders { available, .. } => Ok(available),
            IpcResponse::Err(e) => Err(Error::Recorder(e)),
            _ => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn selected_encoder(&mut self) -> Result<Encoder> {
        match self.recorder.send(IpcCommand::Encoders) {
            IpcResponse::Encoders { selected, .. } => Ok(selected),
            IpcResponse::Err(e) => Err(Error::Recorder(e)),
            _ => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn start_recording(&mut self) -> Result<()> {
        match self.recorder.send(IpcCommand::StartRecording) {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Err(e) => Err(Error::Recorder(e)),
            _ => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn stop_recording(&mut self) -> Result<()> {
        match self.recorder.send(IpcCommand::StopRecording) {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Err(e) => Err(Error::Recorder(e)),
            _ => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn is_recording(&mut self) -> Result<bool> {
        match self.recorder.send(IpcCommand::StopRecording) {
            IpcResponse::Recording(recording) => Ok(recording),
            IpcResponse::Err(e) => Err(Error::Recorder(e)),
            _ => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn shutdown(mut self) -> Result<()> {
        match self.recorder.send(IpcCommand::Shutdown) {
            IpcResponse::Ok => { /* OK continue */ }
            IpcResponse::Err(e) => return Err(Error::ShutdownFailed(self, e)),
            _ => return Err(Error::ShouldNeverHappenNotifyMe),
        }
        match self.recorder.send(IpcCommand::Exit) {
            IpcResponse::Ok => {
                self.recorder.drain_logs();
                Ok(())
            }
            IpcResponse::Err(e) => Err(Error::ShutdownFailed(self, e)),
            _ => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }
}
