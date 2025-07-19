use std::{env, error, fmt, io, path};

use ipc_link::{IpcCommand, IpcLinkMaster, IpcResponse};

pub use intprocess_recorder::settings;
pub use intprocess_recorder::InpRecorder as SingletonRecorder;

#[cfg(target_family = "windows")]
const EXECUTABLE: &str = "./libobs/extprocess_recorder.exe";
#[cfg(target_family = "unix")]
const EXECUTABLE: &str = "./libobs/extprocess_recorder";

pub enum Error {
    Io(io::Error),
    Recorder(String),
    ShutdownFailed(Recorder, String),
    ExitFailed(Recorder, String),
    ShouldNeverHappenNotifyMe,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => f.write_fmt(format_args!("{e:?}")),
            Error::Recorder(e) | Error::ShutdownFailed(_, e) | Error::ExitFailed(_, e) => f.write_str(e),
            Error::ShouldNeverHappenNotifyMe => f.write_str("This error should never happen - please notify me"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
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

pub type Result<T> = std::result::Result<T, Box<Error>>;

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
        let mut rec = match &executable_path {
            Some(p) => IpcLinkMaster::new(p),
            None => {
                let exe_path = env::current_exe().map_err(Error::Io)?;
                let pwd = exe_path
                    .parent()
                    .expect("current exe should always have a parent directory");
                IpcLinkMaster::new(pwd.join(EXECUTABLE))
            }
        }
        .map_err(Error::Io)?;

        let cmd = IpcCommand::Init {
            libobs_data_path: libobs_data_path.map(ToString::to_string),
            plugin_bin_path: plugin_bin_path.map(ToString::to_string),
            plugin_data_path: plugin_data_path.map(ToString::to_string),
        };

        match rec.send(cmd) {
            IpcResponse::Ok => Ok(Self { recorder: rec }),
            IpcResponse::Err(e) => Err(Box::new(Error::Recorder(e))),
            _ => Err(Box::new(Error::ShouldNeverHappenNotifyMe)),
        }
    }

    pub fn configure(&mut self, settings: &settings::RecorderSettings) -> Result<()> {
        match self.recorder.send(IpcCommand::Configure(settings.clone())) {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Err(e) => Err(Box::new(Error::Recorder(e))),
            _ => Err(Box::new(Error::ShouldNeverHappenNotifyMe)),
        }
    }

    pub fn available_encoders(&mut self) -> Result<Vec<settings::Encoder>> {
        match self.recorder.send(IpcCommand::Encoders) {
            IpcResponse::Encoders { available, .. } => Ok(available),
            IpcResponse::Err(e) => Err(Box::new(Error::Recorder(e))),
            _ => Err(Box::new(Error::ShouldNeverHappenNotifyMe)),
        }
    }

    pub fn selected_encoder(&mut self) -> Result<settings::Encoder> {
        match self.recorder.send(IpcCommand::Encoders) {
            IpcResponse::Encoders { selected, .. } => Ok(selected),
            IpcResponse::Err(e) => Err(Box::new(Error::Recorder(e))),
            _ => Err(Box::new(Error::ShouldNeverHappenNotifyMe)),
        }
    }

    pub fn adapter_info(&mut self) -> Result<settings::Adapter> {
        match self.recorder.send(IpcCommand::Adapter) {
            IpcResponse::Adapter(adapter) => Ok(adapter),
            IpcResponse::Err(e) => Err(Box::new(Error::Recorder(e))),
            _ => Err(Box::new(Error::ShouldNeverHappenNotifyMe)),
        }
    }

    pub fn start_recording(&mut self) -> Result<()> {
        match self.recorder.send(IpcCommand::StartRecording) {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Err(e) => Err(Box::new(Error::Recorder(e))),
            _ => Err(Box::new(Error::ShouldNeverHappenNotifyMe)),
        }
    }

    pub fn stop_recording(&mut self) -> Result<()> {
        match self.recorder.send(IpcCommand::StopRecording) {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Err(e) => Err(Box::new(Error::Recorder(e))),
            _ => Err(Box::new(Error::ShouldNeverHappenNotifyMe)),
        }
    }

    pub fn is_recording(&mut self) -> Result<bool> {
        match self.recorder.send(IpcCommand::StopRecording) {
            IpcResponse::Recording(recording) => Ok(recording),
            IpcResponse::Err(e) => Err(Box::new(Error::Recorder(e))),
            _ => Err(Box::new(Error::ShouldNeverHappenNotifyMe)),
        }
    }

    pub fn shutdown(mut self) -> Result<()> {
        match self.recorder.send(IpcCommand::Shutdown) {
            IpcResponse::Ok => { /* OK continue */ }
            IpcResponse::Err(e) => return Err(Box::new(Error::ShutdownFailed(self, e))),
            _ => return Err(Box::new(Error::ShouldNeverHappenNotifyMe)),
        }
        match self.recorder.send(IpcCommand::Exit) {
            IpcResponse::Ok => {
                self.recorder.drain_logs();
                Ok(())
            }
            IpcResponse::Err(e) => Err(Box::new(Error::ExitFailed(self, e))),
            _ => Err(Box::new(Error::ShouldNeverHappenNotifyMe)),
        }
    }
}
