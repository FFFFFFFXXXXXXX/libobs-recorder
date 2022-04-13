extern crate libobs_recorder;
use libobs_recorder::{bitrate::Bitrate, framerate::Framerate, Recorder, RecorderSettings};

fn main() {
    // RECORDER
    let mut recorder = Recorder::create().unwrap();

    // SETTINGS 1
    let mut settings = RecorderSettings::new();
    settings.set_window_title(
        "League of Legends (TM) Client:RiotWindowClass:League of Legends.exe".to_string(),
    );
    settings.set_output_path("C:/Users/Felix/Videos/Test1.mp4".to_string());
    settings.set_input_resolution(libobs_recorder::resolution::Resolution::_1440p);
    recorder.configure(settings).unwrap();

    // RECORD 1
    if recorder.start_recording() {
        std::thread::sleep(std::time::Duration::from_secs(10));
        recorder.stop_recording();
    } else {
        println!("error starting recorder");
    }

    std::thread::sleep(std::time::Duration::from_secs(1));

    // SETTINGS 2
    let mut settings = RecorderSettings::new();
    settings.set_window_title(
        "League of Legends (TM) Client:RiotWindowClass:League of Legends.exe".to_string(),
    );
    settings.set_output_path("C:/Users/Felix/Videos/Test2.mp4".to_string());
    settings.set_input_resolution(libobs_recorder::resolution::Resolution::_1440p);
    settings.set_framerate(Framerate::new(30));
    settings.set_bitrate(Bitrate::mbit(10));
    recorder.configure(settings).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(5));

    // RECORD 2
    if recorder.start_recording() {
        std::thread::sleep(std::time::Duration::from_secs(20));
        recorder.stop_recording();
    } else {
        println!("error starting recorder");
    }

    //std::thread::sleep(std::time::Duration::from_secs(1));

    // SETTINGS 3
    let mut settings = RecorderSettings::new();
    settings.set_window_title(
        "League of Legends (TM) Client:RiotWindowClass:League of Legends.exe".to_string(),
    );
    settings.set_output_path("C:/Users/Felix/Videos/Test3.mp4".to_string());
    settings.set_input_resolution(libobs_recorder::resolution::Resolution::_1440p);
    settings.set_framerate(Framerate::new(60));
    settings.set_bitrate(Bitrate::mbit(20));
    recorder.configure(settings).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(5));

    // RECORD 3
    if recorder.start_recording() {
        std::thread::sleep(std::time::Duration::from_secs(10));
        recorder.stop_recording();
    } else {
        println!("error starting recorder");
    }
}
