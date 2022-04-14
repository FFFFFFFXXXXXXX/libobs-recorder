extern crate libobs_recorder;
use libobs_recorder::{
    framerate::Framerate,
    rate_control::{Cbr, Cqp},
    resolution::Resolution,
    Recorder, RecorderSettings,
};

fn main() {
    // RECORDER
    let mut recorder = Recorder::create().unwrap();

    println!("RECORDING 1");

    // SETTINGS 1
    let mut settings = RecorderSettings::new();
    settings.set_window_title(
        "League of Legends (TM) Client:RiotWindowClass:League of Legends.exe".to_string(),
    );
    settings.set_input_resolution(Resolution::_1440p);
    settings.set_output_path("C:/Users/Felix/Videos/Test1.mp4".to_string());
    settings.set_cqp(Cqp::new(16));
    recorder.configure(settings).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(5));

    // RECORD 1
    if recorder.start_recording() {
        std::thread::sleep(std::time::Duration::from_secs(5));
        dbg!(recorder.stop_recording());
    } else {
        println!("error starting recorder");
    }

    std::thread::sleep(std::time::Duration::from_secs(5));
    println!("RECORDING 2");

    // SETTINGS 2
    let mut settings = RecorderSettings::new();
    settings.set_output_path("C:/Users/Felix/Videos/Test2.mp4".to_string());
    settings.set_input_resolution(libobs_recorder::resolution::Resolution::_1080p);
    settings.set_output_resolution(libobs_recorder::resolution::Resolution::_1440p);
    settings.set_framerate(Framerate::new(45));
    recorder.configure(settings).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(5));

    // RECORD 2
    if recorder.start_recording() {
        std::thread::sleep(std::time::Duration::from_secs(5));
        dbg!(recorder.stop_recording());
    } else {
        println!("error starting recorder");
    }

    std::thread::sleep(std::time::Duration::from_secs(5));
    println!("RECORDING 3");

    // SETTINGS 3
    let mut settings = RecorderSettings::new();
    settings.set_output_path("C:/Users/Felix/Videos/Test3.mp4".to_string());
    settings.set_framerate(Framerate::new(60));
    settings.set_cbr(Cbr::mbit(20));
    settings.record_audio(false);
    recorder.configure(settings).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(5));

    // RECORD 3
    if recorder.start_recording() {
        std::thread::sleep(std::time::Duration::from_secs(5));
        dbg!(recorder.stop_recording());
    } else {
        println!("error starting recorder");
    }
}
