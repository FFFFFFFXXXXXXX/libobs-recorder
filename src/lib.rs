extern crate libobs_sys;
use libobs_sys::{
    base_set_log_handler, bnum_allocs, obs_add_data_path, obs_add_module_path,
    obs_audio_encoder_create, obs_audio_info, obs_encoder, obs_encoder_release,
    obs_encoder_set_audio, obs_encoder_set_video, obs_encoder_update, obs_get_audio, obs_get_video,
    obs_initialized, obs_load_all_modules, obs_log_loaded_modules, obs_output, obs_output_create,
    obs_output_release, obs_output_set_audio_encoder, obs_output_set_video_encoder,
    obs_output_start, obs_output_stop, obs_output_update, obs_post_load_modules, obs_reset_audio,
    obs_reset_video, obs_scale_type_OBS_SCALE_BILINEAR, obs_set_output_source, obs_shutdown,
    obs_source, obs_source_create, obs_source_release, obs_source_remove, obs_source_update,
    obs_startup, obs_video_encoder_create, obs_video_info, speaker_layout_SPEAKERS_STEREO, va_list,
    video_colorspace_VIDEO_CS_DEFAULT, video_format_VIDEO_FORMAT_NV12,
    video_range_type_VIDEO_RANGE_DEFAULT, OBS_VIDEO_SUCCESS,
};

use std::{ffi::CStr, ptr::null_mut};

use framerate::Framerate;
use get::Get;
use obs_data::ObsData;
use rate_control::{Cbr, Cqp};
use resolution::{Resolution, Size};

pub mod framerate;
mod get;
mod obs_data;
pub mod rate_control;
pub mod resolution;

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

const DEFAULT_FRAMERATE: u32 = 30;

const VIDEO_CHANNEL: u32 = 0;
const AUDIO_CHANNEL: u32 = 1;
const AUDIO_ENCODER_INDEX: usize = 0;

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
    window_title: Option<String>,
    input_resolution: Option<Resolution>,
    output_resolution: Option<Resolution>,
    framerate: Framerate,
    bitrate: Cbr,
    cqp: Cqp,
    record_audio: bool,
    output_path: Option<String>,
}

impl RecorderSettings {
    pub fn new() -> Self {
        RecorderSettings {
            window_title: None,
            input_resolution: None,
            output_resolution: None,
            framerate: Framerate::new(0),
            bitrate: Cbr::kbit(0),
            cqp: Cqp::new(0),
            record_audio: true,
            output_path: None,
        }
    }
    pub fn set_window_title<S: Into<String>>(&mut self, window_title: S) {
        self.window_title = Some(window_title.into());
    }
    pub fn set_output_resolution(&mut self, resolution: Resolution) {
        self.output_resolution = Some(resolution);
    }
    pub fn set_input_resolution(&mut self, resolution: Resolution) {
        self.input_resolution = Some(resolution);
    }
    pub fn set_framerate(&mut self, framerate: Framerate) {
        self.framerate = framerate;
    }
    pub fn set_cbr(&mut self, bitrate: Cbr) {
        self.bitrate = bitrate;
    }
    pub fn set_cqp(&mut self, cqp: Cqp) {
        self.cqp = cqp;
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
    input_resolution: Resolution,
    output_resolution: Resolution,
    framerate: Framerate,
}

impl Recorder {
    pub fn init(
        libobs_data_path: Option<String>,
        plugin_bin_path: Option<String>,
        plugin_data_path: Option<String>,
    ) -> Result<(), String> {
        if unsafe { obs_initialized() } {
            return Err("error: obs already initialized".into());
        }

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

            let framerate = Framerate::new(DEFAULT_FRAMERATE);
            Self::reset_video(
                DEFAULT_RESOLUTION.get_size(),
                DEFAULT_RESOLUTION.get_size(),
                framerate,
            )?;
            Self::reset_audio()?;

            obs_post_load_modules();
        }

        Ok(())
    }

    pub fn shutdown() {
        unsafe {
            obs_shutdown();
            if DEBUG {
                println!("{}", bnum_allocs());
            }
        }
    }

    pub fn get(settings: RecorderSettings) -> Self {
        let mut get = Get::new();
        unsafe {
            // SETUP NEW VIDEO SOURCE/ENCODER
            let video_source = {
                let mut data = ObsData::new();
                data.set_string("capture_mode", "any_fullscreen");
                data.set_bool("capture_cursor", true);

                obs_source_create(
                    get.c_str("game_capture"),
                    get.c_str(""),
                    data.get_ptr(),
                    null_mut(),
                )
            };

            let video_encoder = obs_video_encoder_create(
                get.c_str("amd_amf_h264"),
                get.c_str(""),
                null_mut(),
                null_mut(),
            );

            // SETUP NEW AUDIO SOURCE/ENCODER
            let audio_source = obs_source_create(
                get.c_str("wasapi_output_capture"),
                get.c_str(""),
                null_mut(),
                null_mut(),
            );
            let audio_encoder = obs_audio_encoder_create(
                get.c_str("ffmpeg_aac"),
                get.c_str(""),
                null_mut(),
                0,
                null_mut(),
            );

            // SETUP NEW OUTPUT
            let output = obs_output_create(
                get.c_str("ffmpeg_muxer"),
                get.c_str(""),
                null_mut(),
                null_mut(),
            );

            obs_encoder_set_video(video_encoder, obs_get_video());
            obs_encoder_set_audio(audio_encoder, obs_get_audio());

            obs_set_output_source(VIDEO_CHANNEL, video_source);
            obs_output_set_video_encoder(output, video_encoder);
            obs_set_output_source(AUDIO_CHANNEL, audio_source);
            obs_output_set_audio_encoder(output, audio_encoder, AUDIO_ENCODER_INDEX);

            let mut recorder = Recorder {
                video_source,
                video_encoder,
                audio_source,
                audio_encoder,
                output,
                recording: false,
                input_resolution: DEFAULT_RESOLUTION,
                output_resolution: DEFAULT_RESOLUTION,
                framerate: Framerate::new(DEFAULT_FRAMERATE),
            };
            recorder.configure(settings);

            return recorder;
        }
    }

    pub fn start_recording(&mut self) -> bool {
        if DEBUG {
            println!("Recording Start: {}", unsafe { bnum_allocs() });
        }
        if self.recording {
            return false;
        }
        if unsafe { obs_output_start(self.output) } {
            self.recording = true;
        }
        self.recording
    }

    pub fn stop_recording(&mut self) -> bool {
        if !self.recording {
            return false;
        }
        unsafe { obs_output_stop(self.output) }
        self.recording = false;
        if DEBUG {
            println!("Recording Stop: {}", unsafe { bnum_allocs() });
        }
        true
    }

    fn configure(&mut self, settings: RecorderSettings) {
        if DEBUG {
            println!("Configure before: {}", unsafe { bnum_allocs() });
        }

        // RESET VIDEO
        let mut reset_necessary = false;
        let input_size = if let Some(resolution) = settings.input_resolution {
            if resolution != self.input_resolution {
                reset_necessary = true;
                self.input_resolution = resolution;
                resolution.get_size()
            } else {
                self.input_resolution.get_size()
            }
        } else {
            self.input_resolution.get_size()
        };
        let output_size = if let Some(resolution) = settings.output_resolution {
            if resolution != self.output_resolution {
                reset_necessary = true;
                self.output_resolution = resolution;
                resolution.get_size()
            } else {
                self.output_resolution.get_size()
            }
        } else {
            self.output_resolution.get_size()
        };
        let framerate = if settings.framerate.is_set() {
            if settings.framerate != self.framerate {
                reset_necessary = true;
                self.framerate = settings.framerate;
                settings.framerate
            } else {
                self.framerate
            }
        } else {
            self.framerate
        };

        if reset_necessary {
            Self::reset_video(input_size, output_size, framerate).unwrap();
            unsafe { obs_encoder_set_video(self.video_encoder, obs_get_video()) };
        }

        unsafe {
            // UPDATE VIDEO SOURCE
            if let Some(window_title) = settings.window_title {
                let mut data = ObsData::new();
                data.set_string("capture_mode", "window");
                data.set_string("window", window_title);

                obs_source_update(self.video_source, data.get_ptr());
            }

            // UPDATE VIDEO ENCODER
            {
                let mut data = ObsData::new();
                // Static Properties
                data.set_int("Usage", 0);
                data.set_int("Profile", 100);
                // Picture Control Properties
                data.set_double("KeyframeInterval", 2.0);
                data.set_int("BFrame.Pattern", 0);
                if settings.bitrate.is_set() {
                    data.set_int("RateControlMethod", 3);
                    data.set_int("Bitrate.Target", settings.bitrate);
                    data.set_int("FillerData", 1);
                    data.set_int("VBVBuffer", 1);
                    data.set_int("VBVBuffer.Size", settings.bitrate);
                } else if settings.cqp.is_set() {
                    data.set_int("RateControlMethod", 0);
                    data.set_int("QP.IFrame", settings.cqp);
                    data.set_int("QP.PFrame", settings.cqp);
                    data.set_int("QP.BFrame", settings.cqp);
                    data.set_int("VBVBuffer", 1);
                    data.set_int("VBVBuffer.Size", 100000);
                }
                if settings.bitrate.is_set() || settings.cqp.is_set() {
                    obs_encoder_update(self.video_encoder, data.get_ptr());
                }
            }

            // UPDATE AUDIO
            if settings.record_audio {
                obs_set_output_source(AUDIO_CHANNEL, self.audio_source);
            } else {
                obs_set_output_source(AUDIO_CHANNEL, null_mut());
            }

            // UPDATE OUTPUT
            if let Some(output_path) = settings.output_path {
                let mut data = ObsData::new();
                data.set_string("path", output_path);
                obs_output_update(self.output, data.get_ptr());
            }
        }

        if DEBUG {
            println!("Configure after: {}", unsafe { bnum_allocs() });
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
                fps_num: framerate.into(),
                fps_den: 1,
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
            if ret != OBS_VIDEO_SUCCESS.try_into().unwrap() {
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
