// 'windows_subsystem = "windows/console"' decides if the executable should launch in a console window or not
// but only add this for release builds (debug_assertions disabled)
// gets ignored on all other targets
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use intprocess_recorder::InpRecorder;
use ipc_link::{IpcCommand, IpcLinkSlave, IpcResponse};

fn main() {
    let mut recorder = None;

    IpcLinkSlave::new().respond(|cmd| match cmd {
        IpcCommand::Init {
            libobs_data_path,
            plugin_bin_path,
            plugin_data_path,
        } => {
            if let Err(e) = InpRecorder::initialize(
                libobs_data_path.as_deref(),
                plugin_bin_path.as_deref(),
                plugin_data_path.as_deref(),
            ) {
                return Some(IpcResponse::Err(e.to_string()));
            }

            match InpRecorder::get_handle() {
                Ok(rec) => {
                    recorder = Some(rec);
                    Some(IpcResponse::Ok)
                }
                Err(e) => Some(IpcResponse::Err(e.to_string())),
            }
        }
        IpcCommand::Configure(settings) => {
            if let Some(recorder) = recorder.as_mut() {
                if let Err(e) = recorder.configure(&settings) {
                    Some(IpcResponse::Err(e.to_string()))
                } else {
                    Some(IpcResponse::Ok)
                }
            } else {
                Some(IpcResponse::Err("recorder not initialized".into()))
            }
        }
        IpcCommand::Encoders => {
            if let Some(recorder) = recorder.as_mut() {
                Some(IpcResponse::Encoders {
                    available: recorder.get_available_encoders(),
                    selected: recorder.selected_encoder(),
                })
            } else {
                Some(IpcResponse::Err("recorder not initialized".into()))
            }
        }
        IpcCommand::Adapter => {
            if let Some(recorder) = recorder.as_mut() {
                Some(IpcResponse::Adapter(recorder.get_adapter_info()))
            } else {
                Some(IpcResponse::Err("recorder not initialized".into()))
            }
        }
        IpcCommand::StartRecording => {
            if let Some(recorder) = recorder.as_mut() {
                if let Err(e) = recorder.start_recording() {
                    Some(IpcResponse::Err(format!("failed to start recording: {e}")))
                } else {
                    Some(IpcResponse::Ok)
                }
            } else {
                Some(IpcResponse::Err("recorder not initialized".into()))
            }
        }
        IpcCommand::StopRecording => {
            if let Some(recorder) = recorder.as_mut() {
                if recorder.is_recording() {
                    recorder.stop_recording();
                }
                Some(IpcResponse::Ok)
            } else {
                Some(IpcResponse::Err("recorder not initialized".into()))
            }
        }
        IpcCommand::IsRecording => {
            if let Some(recorder) = recorder.as_mut() {
                Some(IpcResponse::Recording(recorder.is_recording()))
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
                Some(IpcResponse::Err(e.to_string()))
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
