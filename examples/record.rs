extern crate libobs_recorder;

use libobs_recorder::{
    Framerate, Recorder, RecorderSettings, Resolution, Size, Window, RecordAudio, Encoder, RateControl,
};

use windows::Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE};
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
    #[cfg(target_os = "windows")]
    unsafe {
        // Get correct window size from GetClientRect
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE)
    };

    let league_window = Window::new(
        "League of Legends (TM) Client",
        Some("RiotWindowClass".into()),
        Some("League of Legends.exe".into()),
    );
    let window_size = get_window_size("League of Legends (TM) Client", "RiotWindowClass").unwrap();

    // SETTINGS 1
    let mut settings1 = RecorderSettings::new();
    settings1.set_window(league_window.clone());
    settings1.set_input_size(window_size);
    settings1.set_output_path("./Test1.mp4");
    settings1.set_rate_control(RateControl::CQP(25));
    settings1.record_audio(RecordAudio::NONE);

    // SETTINGS 2
    let mut settings2 = RecorderSettings::new();
    settings2.set_window(league_window.clone());
    settings2.set_output_path("./Test2.mp4");
    settings2.set_input_size(window_size);
    settings2.set_output_resolution(Resolution::_1440p);
    settings2.set_framerate(Framerate::new(45, 1));
    settings2.set_rate_control(RateControl::CQP(10));
    settings2.record_audio(RecordAudio::APPLICATION);

    // SETTINGS 3
    let mut settings3 = RecorderSettings::new();
    settings3.set_window(league_window.clone());
    settings3.set_output_path("./Test3.mp4");
    settings3.set_input_size(window_size);
    settings3.set_framerate(Framerate::new(60, 1));
    settings3.set_rate_control(RateControl::CBR(20000));
    settings3.record_audio(RecordAudio::SYSTEM);

    // Settings 4
    let mut settings4 = RecorderSettings::new();
    settings4.set_window(league_window.clone());
    settings4.set_output_path("./Test4.mp4");
    settings4.set_input_size(window_size);
    settings4.set_framerate(Framerate::new(60, 1));
    settings4.set_rate_control(RateControl::CQP(20));
    settings4.record_audio(RecordAudio::SYSTEM);

    // Setttings 5
    let mut settings5 = RecorderSettings::new();
    settings5.set_window(league_window.clone());
    settings5.set_output_path("./Test5.mp4");
    settings5.set_input_size(window_size);
    settings5.set_framerate(Framerate::new(60, 1));
    settings5.set_rate_control(RateControl::CQP(20));
    settings5.record_audio(RecordAudio::SYSTEM);
    settings5.set_encoder(Encoder::AMD_NEW_H264);

    let settings = [settings1, settings2, settings3, settings4, settings5];

    let encoders = Recorder::init(None, None, None).unwrap();
    println!("available encoders:\n{:?}", encoders);

    for i in 0..settings.len() {
        println!("RECORDING {}", i);

        let mut recorder = Recorder::get(&settings[i]).unwrap();
        if recorder.start_recording() {
            std::thread::sleep(std::time::Duration::from_secs(10));
            dbg!(recorder.stop_recording());
        } else {
            println!("error starting recorder");
        }
    }
    Recorder::shutdown();
}
