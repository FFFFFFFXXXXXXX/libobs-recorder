extern crate libobs_sys;

use libobs_sys::{
    base_set_log_handler, bnum_allocs, obs_add_data_path, obs_add_module_path,
    obs_audio_encoder_create, obs_audio_info, obs_encoder, obs_encoder_release,
    obs_encoder_set_audio, obs_encoder_set_video, obs_enum_encoder_types, obs_get_audio,
    obs_get_video, obs_get_video_info, obs_initialized, obs_load_all_modules,
    obs_log_loaded_modules, obs_output, obs_output_create, obs_output_release,
    obs_output_set_audio_encoder, obs_output_set_video_encoder, obs_output_start, obs_output_stop,
    obs_post_load_modules, obs_reset_audio, obs_reset_video, obs_scale_type_OBS_SCALE_BILINEAR,
    obs_set_output_source, obs_shutdown, obs_source, obs_source_create, obs_source_release,
    obs_source_remove, obs_startup, obs_video_encoder_create, obs_video_info,
    speaker_layout_SPEAKERS_STEREO, va_list, video_colorspace_VIDEO_CS_DEFAULT,
    video_format_VIDEO_FORMAT_NV12, video_range_type_VIDEO_RANGE_DEFAULT, OBS_VIDEO_SUCCESS,
};
use window::Window;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::RECT,
        UI::{
            HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE},
            WindowsAndMessaging::{FindWindowA, GetClientRect},
        },
    },
};

use std::{ffi::CStr, fmt::Debug, mem::MaybeUninit, os::raw::c_char, ptr::null_mut};

use framerate::Framerate;
use get::Get;
use obs_data::ObsData;
use rate_control::{Cbr, Cqp, Icq, RateControl};
use resolution::{Resolution, Size};

pub mod framerate;
mod get;
mod obs_data;
pub mod rate_control;
pub mod resolution;
pub mod window;

#[cfg(feature = "debug")]
const DEBUG: bool = true;
#[cfg(not(feature = "debug"))]
const DEBUG: bool = false;

#[cfg(target_os = "windows")]
const GRAPHICS_MODULE: &str = "libobs-d3d11.dll";
#[cfg(not(target_os = "windows"))]
const GRAPHICS_MODULE: &str = "libobs-opengl.dll";

const LIBOBS_DATA_PATH: &str = "./data/libobs/";
const PLUGIN_BIN_PATH: &str = "./obs-plugins/64bit/";
const PLUGIN_DATA_PATH: &str = "./data/obs-plugins/%module%/";

const DEFAULT_RESOLUTION: Resolution = Resolution::_1080p;

const DEFAULT_FRAMERATE_NUM: u32 = 30;
const DEFAULT_FRAMERATE_DEN: u32 = 1;

const DEFAULT_CQP: u32 = 20;

const AMF_CONSTANT_QP: u32 = 0;

const VIDEO_CHANNEL: u32 = 0;
const AUDIO_CHANNEL: u32 = 1;
const AUDIO_ENCODER_INDEX: usize = 0;

static mut ENCODER_TYPE: &str = "";

unsafe extern "C" fn log_handler(
    _lvl: ::std::os::raw::c_int,
    msg: *const ::std::os::raw::c_char,
    _args: va_list,
    _p: *mut ::std::os::raw::c_void,
) {
    if DEBUG {
        dbg!(CStr::from_ptr(msg));
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecorderSettings {
    window: Option<Window>,
    output_resolution: Option<Resolution>,
    framerate: Framerate,
    rate_control: RateControl,
    record_audio: bool,
    output_path: Option<String>,
}

impl RecorderSettings {
    pub fn new() -> Self {
        RecorderSettings {
            window: None,
            output_resolution: None,
            framerate: Framerate::new(0, 0),
            rate_control: RateControl::default(),
            record_audio: true,
            output_path: None,
        }
    }
    pub fn set_window(&mut self, window: Window) {
        self.window = Some(window);
    }
    pub fn set_output_resolution(&mut self, resolution: Resolution) {
        self.output_resolution = Some(resolution);
    }
    pub fn set_framerate(&mut self, framerate: Framerate) {
        self.framerate = framerate;
    }
    pub fn set_cbr(&mut self, bitrate: Cbr) {
        self.rate_control.cbr = bitrate;
    }
    pub fn set_cqp(&mut self, cqp: Cqp) {
        self.rate_control.cqp = cqp;
    }
    pub fn set_icq(&mut self, icq: Icq) {
        self.rate_control.icq = icq;
    }
    pub fn record_audio(&mut self, record_audio: bool) {
        self.record_audio = record_audio;
    }
    pub fn set_output_path<S: Into<String>>(&mut self, output_path: S) {
        self.output_path = Some(output_path.into());
    }
}

pub struct Recorder {
    video_source: *mut obs_source,
    video_encoder: *mut obs_encoder,
    audio_source: *mut obs_source,
    audio_encoder: *mut obs_encoder,
    output: *mut obs_output,
    recording: bool,
}

impl Recorder {
    pub fn init(
        libobs_data_path: Option<String>,
        plugin_bin_path: Option<String>,
        plugin_data_path: Option<String>,
    ) -> Result<String, String> {
        if unsafe { obs_initialized() } {
            return Err("error: obs already initialized".into());
        }

        #[cfg(target_os = "windows")]
        unsafe {
            // Get correct window size from GetClientRect
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE)
        };

        // set defaults in case no arguments were provided
        let libobs_data_path: String = if let Some(path) = libobs_data_path {
            path.into()
        } else {
            LIBOBS_DATA_PATH.into()
        };
        let plugin_bin_path: String = if let Some(path) = plugin_bin_path {
            path.into()
        } else {
            PLUGIN_BIN_PATH.into()
        };
        let plugin_data_path: String = if let Some(path) = plugin_data_path {
            path.into()
        } else {
            PLUGIN_DATA_PATH.into()
        };

        let mut get = Get::new();
        unsafe {
            // STARTUP
            base_set_log_handler(Some(log_handler), null_mut());

            if !obs_startup(get.c_str("en-US"), null_mut(), null_mut()) {
                return Err(String::from("error on libobs startup"));
            }

            obs_add_data_path(get.c_str(libobs_data_path));
            obs_add_module_path(get.c_str(plugin_bin_path), get.c_str(plugin_data_path));
            obs_load_all_modules();

            if DEBUG {
                obs_log_loaded_modules();
            }

            let framerate = Framerate::new(DEFAULT_FRAMERATE_NUM, DEFAULT_FRAMERATE_DEN);
            Self::reset_video(
                DEFAULT_RESOLUTION.get_size(),
                DEFAULT_RESOLUTION.get_size(),
                framerate,
            )?;
            Self::reset_audio()?;

            obs_post_load_modules();

            let mut amf = false;
            let mut jim_nvenc = false;
            let mut ffmpeg_nvenc = false;
            let mut qsv = false;

            let mut n = 0;
            loop {
                let mut ptr = MaybeUninit::<*const c_char>::uninit();
                if !obs_enum_encoder_types(n, ptr.as_mut_ptr()) {
                    break;
                }
                let encoder = ptr.assume_init();
                if let Ok(enc) = CStr::from_ptr(encoder).to_str() {
                    match enc {
                        "amd_amf_h264" => amf = true,
                        "jim_nvenc" => jim_nvenc = true,
                        "ffmpeg_nvenc" => ffmpeg_nvenc = true,
                        "obs_qsv11" => qsv = true,
                        _ => {}
                    }
                }
                n += 1;
            }

            ENCODER_TYPE = if jim_nvenc {
                "jim_nvenc"
            } else if ffmpeg_nvenc {
                "ffmpeg_nvenc"
            } else if amf {
                "amd_amf_h264"
            } else if qsv {
                "obs_qsv11"
            } else {
                "obs_x264"
            };

            Ok(ENCODER_TYPE.into())
        }
    }

    pub fn shutdown() {
        unsafe {
            obs_shutdown();
            if DEBUG {
                println!("{}", bnum_allocs());
            }
        }
    }

    pub fn get(settings: RecorderSettings) -> Result<Self, String> {
        if DEBUG {
            println!("before get: {}", unsafe { bnum_allocs() });
        }

        let window = if let Some(w) = settings.window {
            w
        } else {
            return Err("No window options set".into());
        };

        // RESET VIDEO
        let mut reset_necessary = false;
        let ovi = Self::get_video_info()?;
        let input_size = if let Ok(size) = Self::get_window_size(window.name(), window.class()) {
            if size.width() != ovi.base_width || size.height() != ovi.base_height {
                reset_necessary = true;
            }
            size
        } else {
            Size::new(ovi.base_width, ovi.base_height)
        };
        let output_size = if let Some(resolution) = settings.output_resolution {
            let size = resolution.get_size();
            if size.width() != ovi.output_width || size.height() != ovi.output_height {
                reset_necessary = true;
            }
            size
        } else {
            Size::new(ovi.output_width, ovi.output_height)
        };
        let framerate = if settings.framerate.is_set() {
            if settings.framerate.num() != ovi.fps_num || settings.framerate.den() != ovi.fps_den {
                reset_necessary = true;
            }
            settings.framerate
        } else {
            Framerate::new(ovi.fps_num, ovi.fps_den)
        };
        if reset_necessary {
            Self::reset_video(input_size, output_size, framerate)?;
        }

        let mut get = Get::new();
        unsafe {
            // SETUP NEW VIDEO SOURCE
            // let mut data = ObsData::new();
            // data.set_string("capture_mode", "any_fullscreen");
            // data.set_bool("capture_cursor", true);
            let video_source = {
                let mut data = ObsData::new();
                data.set_string("capture_mode", "window");
                data.set_string("window", window.get_libobs_window_id());

                obs_source_create(
                    get.c_str("game_capture"),
                    get.c_str(""),
                    data.get_ptr(),
                    null_mut(),
                )
            };
            // SETUP NEW VIDEO ENCODER
            let video_encoder = {
                let data = match ENCODER_TYPE {
                    "amd_amf_h264" => Self::amd_amf_h264_settings(&settings.rate_control),
                    "jim_nvenc" | "ffmpeg_nvenc" => Self::nvenc_settings(&settings.rate_control),
                    "obs_qsv11" => Self::quicksync_settings(&settings.rate_control),
                    "obs_x264" => Self::obs_x264_settings(&settings.rate_control),
                    _ => panic!("This shouldnt happen!"),
                };
                let mut get = Get::new();
                obs_video_encoder_create(
                    get.c_str(ENCODER_TYPE),
                    get.c_str(""),
                    data.get_ptr(),
                    null_mut(),
                )
            };

            // SETUP NEW AUDIO SOURCE
            let audio_source = obs_source_create(
                get.c_str("wasapi_output_capture"),
                get.c_str(""),
                null_mut(),
                null_mut(),
            );
            // SETUP NEW AUDIO ENCODER
            let audio_encoder = obs_audio_encoder_create(
                get.c_str("ffmpeg_aac"),
                get.c_str(""),
                null_mut(),
                0,
                null_mut(),
            );

            // SETUP NEW OUTPUT
            let mut data = ObsData::new();
            if let Some(output_path) = settings.output_path {
                data.set_string("path", output_path);
            } else {
                data.set_string("path", "./recording.mp4");
            }
            let output = obs_output_create(
                get.c_str("ffmpeg_muxer"),
                get.c_str(""),
                data.get_ptr(),
                null_mut(),
            );

            obs_encoder_set_video(video_encoder, obs_get_video());
            obs_set_output_source(VIDEO_CHANNEL, video_source);
            obs_output_set_video_encoder(output, video_encoder);

            obs_encoder_set_audio(audio_encoder, obs_get_audio());
            if settings.record_audio {
                obs_set_output_source(AUDIO_CHANNEL, audio_source);
            } else {
                obs_set_output_source(AUDIO_CHANNEL, null_mut());
            }
            obs_output_set_audio_encoder(output, audio_encoder, AUDIO_ENCODER_INDEX);

            if DEBUG {
                println!("after get: {}", bnum_allocs());
            }

            Ok(Recorder {
                video_source,
                video_encoder,
                audio_source,
                audio_encoder,
                output,
                recording: false,
            })
        }
    }

    pub fn start_recording(&mut self) -> bool {
        if DEBUG {
            println!("Recording Start: {}", unsafe { bnum_allocs() });
        }
        if !self.recording && unsafe { obs_output_start(self.output) } {
            self.recording = true;
        }
        self.recording
    }

    pub fn stop_recording(&mut self) -> bool {
        if !self.recording {
            return false;
        }
        unsafe { obs_output_stop(self.output) }
        if DEBUG {
            println!("Recording Stop: {}", unsafe { bnum_allocs() });
        }
        self.recording = false;
        self.recording
    }

    fn amd_amf_h264_settings(settings: &RateControl) -> ObsData {
        let mut data = ObsData::new();
        // Static Properties
        data.set_int("Usage", 0);
        data.set_int("Profile", 100);
        // Common Properties
        data.set_int("VBVBuffer", 1);
        // Picture Control Properties
        data.set_double("KeyframeInterval", 2.0);
        data.set_int("BFrame.Pattern", 0);
        if settings.cbr.is_set() {
            data.set_string("rate_control", "CBR");
            data.set_int("bitrate", settings.cbr);
            data.set_int("FillerData", 1);
            data.set_int("VBVBuffer.Size", settings.cbr);
        } else {
            let cqp = if settings.cqp.is_set() {
                settings.cqp
            } else {
                Cqp::new(DEFAULT_CQP)
            };
            data.set_int("RateControlMethod", AMF_CONSTANT_QP);
            data.set_int("QP.IFrame", cqp);
            data.set_int("QP.PFrame", cqp);
            data.set_int("QP.BFrame", cqp);
            data.set_int("VBVBuffer.Size", 100000);
        }
        return data;
    }

    fn nvenc_settings(settings: &RateControl) -> ObsData {
        let mut data = ObsData::new();
        data.set_string("profile", "high");
        data.set_string("preset", "hq");
        if settings.cbr.is_set() {
            data.set_string("rate_control", "CBR");
            data.set_int("bitrate", settings.cbr);
        } else {
            let cqp = if settings.cqp.is_set() {
                settings.cqp
            } else {
                Cqp::new(DEFAULT_CQP)
            };
            data.set_string("rate_control", "CQP");
            data.set_int("cqp", cqp);
        }
        return data;
    }

    fn quicksync_settings(settings: &RateControl) -> ObsData {
        let mut data = ObsData::new();
        data.set_string("profile", "high");
        if settings.icq.is_set() {
            data.set_string("rate_control", "ICQ");
            data.set_int("icq_quality", settings.icq);
        } else if settings.cbr.is_set() {
            data.set_string("rate_control", "CBR");
            data.set_int("bitrate", settings.cbr);
        } else {
            let cqp = if settings.cqp.is_set() {
                settings.cqp
            } else {
                Cqp::new(DEFAULT_CQP)
            };
            data.set_string("rate_control", "CQP");
            data.set_int("qpi", cqp);
            data.set_int("qpp", cqp);
            data.set_int("qpb", cqp);
        }
        return data;
    }

    fn obs_x264_settings(settings: &RateControl) -> ObsData {
        let mut data = ObsData::new();
        data.set_bool("use_bufsize", true);
        data.set_string("profile", "high");
        data.set_string("preset", "veryfast");
        if settings.cbr.is_set() {
            data.set_string("rate_control", "CBR");
            data.set_int("bitrate", settings.cbr);
        } else {
            let cqp = if settings.cqp.is_set() {
                settings.cqp
            } else {
                Cqp::new(DEFAULT_CQP)
            };
            data.set_string("rate_control", "CRF");
            data.set_int("crf", cqp);
        }
        return data;
    }

    fn get_video_info() -> Result<obs_video_info, String> {
        let mut ovi = obs_video_info {
            adapter: 0,
            graphics_module: null_mut(),
            fps_num: 0,
            fps_den: 0,
            base_width: 0,
            base_height: 0,
            output_width: 0,
            output_height: 0,
            output_format: -1,
            gpu_conversion: false,
            colorspace: -1,
            range: -1,
            scale_type: -1,
        };
        if unsafe { obs_get_video_info(&mut ovi as *mut _) } {
            Ok(ovi)
        } else {
            Err("Error video was not set! Maybe Recorder was not initialized?".into())
        }
    }

    #[cfg(target_os = "windows")]
    fn get_window_size<S: Into<String>>(
        window_title: S,
        window_class: Option<&String>,
    ) -> Result<Size, ()> {
        let mut window_title = window_title.into().clone();
        window_title.push('\0'); // null terminate

        let title = PCSTR(window_title.as_ptr());
        let class = if let Some(cn) = window_class {
            let mut class_name = cn.to_owned();
            class_name.push('\0'); // null terminate
            PCSTR(class_name.as_ptr())
        } else {
            let class_name: PCSTR = PCSTR::default(); // null
            class_name
        };

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

    fn reset_video(
        input_size: Size,
        output_size: Size,
        framerate: Framerate,
    ) -> Result<(), String> {
        unsafe {
            let mut get = Get::new();
            let mut ovi = obs_video_info {
                adapter: 0,
                graphics_module: get.c_str(GRAPHICS_MODULE),
                fps_num: framerate.num(),
                fps_den: framerate.den(),
                base_width: input_size.width(),
                base_height: input_size.height(),
                output_width: output_size.width(),
                output_height: output_size.height(),
                output_format: video_format_VIDEO_FORMAT_NV12,
                gpu_conversion: true,
                colorspace: video_colorspace_VIDEO_CS_DEFAULT,
                range: video_range_type_VIDEO_RANGE_DEFAULT,
                scale_type: obs_scale_type_OBS_SCALE_BILINEAR,
            };

            let ret = obs_reset_video(&mut ovi as *mut _);
            if ret != OBS_VIDEO_SUCCESS as i32 {
                return Err(String::from("error on libobs reset video"));
            }
        }
        Ok(())
    }

    fn reset_audio() -> Result<(), String> {
        let ai = obs_audio_info {
            samples_per_sec: 44100,
            speakers: speaker_layout_SPEAKERS_STEREO,
        };
        let ok = unsafe { obs_reset_audio(&ai) };
        if !ok {
            return Err(String::from("error on libobs reset audio"));
        }
        Ok(())
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        unsafe {
            // video
            obs_source_remove(self.video_source);
            obs_source_release(self.video_source);
            obs_encoder_release(self.video_encoder);
            // audio
            obs_source_remove(self.audio_source);
            obs_source_release(self.audio_source);
            obs_encoder_release(self.audio_encoder);
            // output
            obs_output_release(self.output);
        }
    }
}
