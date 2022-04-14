extern crate libobs_recorder;
use libobs_recorder::{
    framerate::Framerate,
    rate_control::{Cbr, Cqp},
    resolution::Resolution,
    Recorder, RecorderSettings,
};

fn main() {
    // SETTINGS 1
    let mut settings1 = RecorderSettings::new();
    settings1
        .set_window_title("League of Legends (TM) Client:RiotWindowClass:League of Legends.exe");
    settings1.set_input_resolution(Resolution::_1440p);
    settings1.set_output_path("C:/Users/Felix/Videos/Test1.mp4");
    settings1.set_cqp(Cqp::new(16));

    // SETTINGS 2
    let mut settings2 = RecorderSettings::new();
    settings2
        .set_window_title("League of Legends (TM) Client:RiotWindowClass:League of Legends.exe");
    settings2.set_output_path("C:/Users/Felix/Videos/Test2.mp4");
    settings2.set_input_resolution(libobs_recorder::resolution::Resolution::_1080p);
    settings2.set_output_resolution(libobs_recorder::resolution::Resolution::_1440p);
    settings2.set_framerate(Framerate::new(45));
    settings2.set_cqp(Cqp::new(16));
    settings2.record_audio(false);

    // SETTINGS 3
    let mut settings3 = RecorderSettings::new();
    settings3
        .set_window_title("League of Legends (TM) Client:RiotWindowClass:League of Legends.exe");
    settings3.set_output_path("C:/Users/Felix/Videos/Test3.mp4");
    settings3.set_framerate(Framerate::new(60));
    settings3.set_cbr(Cbr::mbit(20));
    settings3.record_audio(true);

    Recorder::init().unwrap();
    {
        // RECORD 1
        println!("RECORDING 1");

        let mut recorder = Recorder::get(settings1);
        if recorder.start_recording() {
            std::thread::sleep(std::time::Duration::from_secs(5));
            dbg!(recorder.stop_recording());
        } else {
            println!("error starting recorder");
        }
    }
    {
        // RECORD 2
        println!("RECORDING 2");

        let mut recorder = Recorder::get(settings2);
        if recorder.start_recording() {
            std::thread::sleep(std::time::Duration::from_secs(5));
            dbg!(recorder.stop_recording());
        } else {
            println!("error starting recorder");
        }
    }
    {
        // RECORD 3
        println!("RECORDING 3");

        let mut recorder = Recorder::get(settings3);
        if recorder.start_recording() {
            std::thread::sleep(std::time::Duration::from_secs(5));
            dbg!(recorder.stop_recording());
        } else {
            println!("error starting recorder");
        }
    }
    Recorder::shutdown();
}
