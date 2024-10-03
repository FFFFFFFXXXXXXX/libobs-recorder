use libobs_recorder::settings::{AudioSource, Framerate, RateControl, RecorderSettings, StdResolution, Window};
use libobs_recorder::Recorder;

fn main() {
    let mut rec = Recorder::new().unwrap();
    println!("created recorder");
    println!("configured recorder: {:?}\n", rec.configure(&settings()));

    println!("selected adapter: {:?}\n", rec.adapter_info());
    println!("available encoders for adapter: {:?}", rec.available_encoders());
    println!("selected encoder: {:?}\n", rec.selected_encoder());

    println!("started recording: {:?}", rec.start_recording());
    std::thread::sleep(std::time::Duration::from_secs(15));
    println!("stopped recording: {:?}", rec.stop_recording());
}

fn settings() -> RecorderSettings {
    const WINDOW_TITLE: &str = "League of Legends (TM) Client";
    const WINDOW_CLASS: &str = "RiotWindowClass";
    const WINDOW_PROCESS: &str = "League of Legends.exe";

    let mut settings = RecorderSettings::new(
        Window::new(WINDOW_TITLE, Some(WINDOW_CLASS.into()), Some(WINDOW_PROCESS.into())),
        StdResolution::_2560x1440p,
        StdResolution::_2560x1440p,
        "./output.mp4",
    );

    settings.set_framerate(Framerate::new(60, 1));
    settings.set_rate_control(RateControl::CBR(10000));
    settings.set_audio_source(AudioSource::ALL);

    settings
}
