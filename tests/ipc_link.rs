use intprocess_recorder::settings::{AudioSource, Framerate, RateControl, RecorderSettings, StdResolution, Window};
use ipc_link::{IpcCommand, IpcLinkMaster};

#[cfg(target_family = "windows")]
const EXECUTABLE: &str = "./target/debug/libobs/extprocess_recorder.exe";
#[cfg(target_family = "unix")]
const EXECUTABLE: &str = "./target/debug/libobs/extprocess_recorder";

const WINDOW_TITLE: &str = "League of Legends (TM) Client";
const WINDOW_CLASS: &str = "RiotWindowClass";
const WINDOW_PROCESS: &str = "League of Legends.exe";

// copy binaries and extprocess_recorder to target/debug/libobs first
#[test]
fn main() {
    let mut link = IpcLinkMaster::new(EXECUTABLE).unwrap();

    link.send(IpcCommand::Init {
        libobs_data_path: None,
        plugin_bin_path: None,
        plugin_data_path: None,
    });
    println!("Configure: {:?}", link.send(IpcCommand::Configure(settings())));

    println!("Start: {:?}", link.send(IpcCommand::StartRecording));
    std::thread::sleep(std::time::Duration::from_secs(3));
    println!("Stop: {:?}", link.send(IpcCommand::StopRecording));

    println!("Shutdown: {:?}", link.send(IpcCommand::Shutdown));
    println!("Exit: {:?}", link.send(IpcCommand::Exit));
}

fn settings() -> RecorderSettings {
    let mut settings = RecorderSettings::new(
        Window::new(WINDOW_TITLE, Some(WINDOW_CLASS.into()), Some(WINDOW_PROCESS.into())),
        StdResolution::_2560x1440p,
        StdResolution::_2560x1440p,
        "./output.mp4",
    );

    settings.set_framerate(Framerate::new(45, 1));
    settings.set_rate_control(RateControl::CQP(25));
    settings.set_audio_source(AudioSource::ALL);

    settings
}
