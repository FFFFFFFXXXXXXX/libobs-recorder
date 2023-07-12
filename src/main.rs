use intprocess_recorder::InpRecorder;
use ipc_link::{IpcCommand, IpcLinkSlave, IpcResponse};

fn main() {
    let mut recorder = None;

    let mut slave = IpcLinkSlave::new();
    slave.respond(|cmd| match cmd {
        IpcCommand::Init {
            libobs_data_path,
            plugin_bin_path,
            plugin_data_path,
        } => {
            recorder = InpRecorder::new(
                libobs_data_path.as_deref(),
                plugin_bin_path.as_deref(),
                plugin_data_path.as_deref(),
            )
            .ok();
            Some(IpcResponse::Ok)
        }
        IpcCommand::Settings(settings) => {
            if let Some(recorder) = recorder.as_mut() {
                if let Err(e) = recorder.set(&settings) {
                    Some(IpcResponse::Err(e))
                } else {
                    Some(IpcResponse::Ok)
                }
            } else {
                Some(IpcResponse::Err("recorder not initialized".into()))
            }
        }
        IpcCommand::StartRecording => {
            if let Some(recorder) = recorder.as_mut() {
                if !recorder.is_recording() {
                    if recorder.start_recording() {
                        Some(IpcResponse::Ok)
                    } else {
                        Some(IpcResponse::Err("failed to start recording".into()))
                    }
                } else {
                    Some(IpcResponse::Err("already recording".into()))
                }
            } else {
                Some(IpcResponse::Err("recorder not initialized".into()))
            }
        }
        IpcCommand::StopRecording => {
            if let Some(recorder) = recorder.as_mut() {
                if recorder.is_recording() {
                    recorder.stop_recording();
                    Some(IpcResponse::Ok)
                } else {
                    Some(IpcResponse::Err("currently not recording".into()))
                }
            } else {
                Some(IpcResponse::Err("recorder not initialized".into()))
            }
        }
        IpcCommand::Shutdown => {
            // stop recording and drop recorder
            if let Some(mut recorder) = recorder.take() {
                recorder.stop_recording();
            }

            if let Err(e) = InpRecorder::shutdown() {
                Some(IpcResponse::Err(e))
            } else {
                Some(IpcResponse::Ok)
            }
        }
        IpcCommand::Exit => {
            if recorder.is_some() {
                Some(IpcResponse::Err("recorder is not shut down".into()))
            } else {
                None
            }
        }
    });
}
