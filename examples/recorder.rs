use intprocess_recorder::settings::{AudioSource, Framerate, RateControl, Resolution, Window};
use libobs_recorder::{Recorder, RecorderSettings};

fn main() {
    let mut rec = Recorder::new().unwrap();

    println!("configure start");
    rec.configure(&settings());
    println!("configure stop");

    rec.start_recording();
    std::thread::sleep(std::time::Duration::from_secs(15));
    rec.stop_recording();
}

fn settings() -> RecorderSettings {
    const WINDOW_TITLE: &str = "League of Legends (TM) Client";
    const WINDOW_CLASS: &str = "RiotWindowClass";
    const WINDOW_PROCESS: &str = "League of Legends.exe";

    let mut settings = RecorderSettings::new();

    settings.set_window(Window::new(
        WINDOW_TITLE,
        Some(WINDOW_CLASS.into()),
        Some(WINDOW_PROCESS.into()),
    ));

    settings.set_input_size(Resolution::_1440p.get_size());
    settings.set_output_resolution(Resolution::_1440p);
    settings.set_framerate(Framerate::new(60, 1));
    settings.set_rate_control(RateControl::CBR(10000));
    settings.record_audio(AudioSource::ALL);
    settings.set_output_path("./output.mp4");

    settings
}
