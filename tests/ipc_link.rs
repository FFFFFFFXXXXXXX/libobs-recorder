use intprocess_recorder::settings::{AudioSource, Framerate, RateControl, Resolution, Window};
use ipc_link::{IpcCommand, IpcLinkMaster};
use libobs_recorder::RecorderSettings;

const WINDOW_TITLE: &str = "League of Legends (TM) Client";
const WINDOW_CLASS: &str = "RiotWindowClass";
const WINDOW_PROCESS: &str = "League of Legends.exe";

fn main() {
    let mut link = IpcLinkMaster::new("./libobs/extprocess_recorder.exe", true).unwrap();

    link.send(IpcCommand::Init {
        libobs_data_path: None,
        plugin_bin_path: None,
        plugin_data_path: None,
    });

    println!("Configure: {:?}", link.send(IpcCommand::Configure(settings())));

    std::thread::sleep(std::time::Duration::from_secs(3));

    println!("Start: {:?}", link.send(IpcCommand::StartRecording));
    std::thread::sleep(std::time::Duration::from_secs(30));
    println!("Stop: {:?}", link.send(IpcCommand::StopRecording));

    std::thread::sleep(std::time::Duration::from_secs(3));

    println!("Shutdown: {:?}", link.send(IpcCommand::Shutdown));
    std::thread::sleep(std::time::Duration::from_secs(3));
    println!("Exit: {:?}", link.send(IpcCommand::Exit));
}

fn settings() -> RecorderSettings {
    let mut settings = RecorderSettings::new();

    settings.set_window(Window::new(
        WINDOW_TITLE,
        Some(WINDOW_CLASS.into()),
        Some(WINDOW_PROCESS.into()),
    ));

    settings.set_input_resolution(Resolution::_2560x1440p);
    settings.set_output_resolution(Resolution::_2560x1440p);
    settings.set_framerate(Framerate::new(45, 1));
    settings.set_rate_control(RateControl::CQP(25));
    settings.record_audio(AudioSource::ALL);
    settings.set_output_path("./output.mp4");

    settings
}
