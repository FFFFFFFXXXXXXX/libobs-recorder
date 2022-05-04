extern crate libobs_recorder;

use libobs_recorder::{
    rate_control::{Cbr, Cqp},
    Framerate, Recorder, RecorderSettings, Resolution, Size, Window,
};

#[cfg(target_os = "windows")]
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::RECT,
        UI::WindowsAndMessaging::{FindWindowA, GetClientRect},
    },
};

#[cfg(target_os = "windows")]
fn get_window_size(window_title: &str, window_class: &str) -> Result<Size, ()> {
    let mut window_title = window_title.to_owned();
    window_title.push('\0'); // null terminate
    let mut window_class = window_class.to_owned();
    window_class.push('\0'); // null terminate

    let title = PCSTR(window_title.as_ptr());
    let class = PCSTR(window_class.as_ptr());

    let hwnd = unsafe { FindWindowA(class, title) };
    if hwnd.is_invalid() {
        return Err(());
    }

    let mut rect = RECT::default();
    let ok = unsafe { GetClientRect(hwnd, &mut rect as _).as_bool() };
    if ok && rect.right > 0 && rect.bottom > 0 {
        Ok(Size::new(rect.right as u32, rect.bottom as u32))
    } else {
        Err(())
    }
}

fn main() {
    // SETTINGS 1
    let mut settings1 = RecorderSettings::new();
    settings1.set_window(Window::new(
        "League of Legends (TM) Client",
        Some("RiotWindowClass".into()),
        Some("League of Legends.exe".into()),
    ));
    if let Ok(size) = get_window_size("League of Legends (TM) Client", "RiotWindowClass") {
        settings1.set_input_size(size);
    }
    settings1.set_output_path("./Test1.mp4");
    settings1.set_cqp(Cqp::new(25));

    // SETTINGS 2
    let mut settings2 = RecorderSettings::new();
    settings2.set_window(Window::new(
        "League of Legends (TM) Client",
        Some("RiotWindowClass".into()),
        Some("League of Legends.exe".into()),
    ));
    settings2.set_output_path("./Test2.mp4");
    if let Ok(size) = get_window_size("League of Legends (TM) Client", "RiotWindowClass") {
        settings2.set_input_size(size);
    }
    settings2.set_output_resolution(Resolution::_1440p);
    settings2.set_framerate(Framerate::new(45, 1));
    settings2.set_cqp(Cqp::new(10));
    settings2.record_audio(false);

    // SETTINGS 3
    let mut settings3 = RecorderSettings::new();
    settings3.set_window(Window::new(
        "League of Legends (TM) Client",
        Some("RiotWindowClass".into()),
        Some("League of Legends.exe".into()),
    ));
    settings3.set_output_path("./Test3.mp4");
    if let Ok(size) = get_window_size("League of Legends (TM) Client", "RiotWindowClass") {
        settings3.set_input_size(size);
    }
    settings3.set_framerate(Framerate::new(60, 1));
    settings3.set_cbr(Cbr::mbit(20));
    settings3.record_audio(true);

    Recorder::init(None, None, None).unwrap();
    {
        // RECORD 1
        println!("RECORDING 1");

        let mut recorder = Recorder::get(settings1).unwrap();
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

        let mut recorder = Recorder::get(settings2).unwrap();
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

        let mut recorder = Recorder::get(settings3).unwrap();
        if recorder.start_recording() {
            std::thread::sleep(std::time::Duration::from_secs(5));
            dbg!(recorder.stop_recording());
        } else {
            println!("error starting recorder");
        }
    }
    Recorder::shutdown();
}
