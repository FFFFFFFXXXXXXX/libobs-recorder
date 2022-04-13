extern crate libobs_sys;
use std::{ffi::CStr, ptr::null_mut};

use cqp::Cqp;
use get::Get;
use libobs_sys::{
    base_set_log_handler, bnum_allocs, obs_add_data_path, obs_add_module_path,
    obs_audio_encoder_create, obs_audio_info, obs_encoder, obs_encoder_release,
    obs_encoder_set_audio, obs_encoder_set_video, obs_get_audio, obs_get_video, obs_initialized,
    obs_load_all_modules, obs_log_loaded_modules, obs_output, obs_output_create,
    obs_output_release, obs_output_set_audio_encoder, obs_output_set_video_encoder,
    obs_output_start, obs_output_stop, obs_post_load_modules, obs_reset_audio, obs_reset_video,
    obs_set_output_source, obs_shutdown, obs_source, obs_source_create, obs_source_release,
    obs_source_remove, obs_startup, obs_video_encoder_create, obs_video_info,
    speaker_layout_SPEAKERS_STEREO, va_list, video_colorspace_VIDEO_CS_DEFAULT,
    video_format_VIDEO_FORMAT_NV12, video_range_type_VIDEO_RANGE_DEFAULT,
    video_scale_type_VIDEO_SCALE_BILINEAR, OBS_VIDEO_SUCCESS,
};

use bitrate::Bitrate;
use framerate::Framerate;
use obs_data::ObsData;
use resolution::{Resolution, Size};

pub mod bitrate;
pub mod cqp;
pub mod framerate;
mod get;
mod obs_data;
pub mod resolution;

const DEBUG: bool = true;

#[cfg(target_os = "windows")]
const GRAPHICS_MODULE: &str = "libobs-d3d11.dll";
#[cfg(not(target_os = "windows"))]
const GRAPHICS_MODULE: &str = "libobs-opengl.dll";

const LIBOBS_DATA_PATH: &str = "./data/libobs/";
const PLUGIN_BIN_PATH: &str = "./obs-plugins/64bit/";
const PLUGIN_DATA_PATH: &str = "./data/obs-plugins/%module%/";

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;

const DEFAULT_FRAMERATE: u32 = 30;

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
    input_resolution: Resolution,
    output_resolution: Resolution,
    framerate: Framerate,
    bitrate: Bitrate,
    cqp: Cqp,
    record_audio: bool,
    output_path: Option<String>,
}

impl RecorderSettings {
    pub fn new() -> Self {
        RecorderSettings {
            window_title: None,
            input_resolution: Resolution::_1080p,
            output_resolution: Resolution::_1080p,
            framerate: Framerate::default(),
            bitrate: Bitrate::kbit(0),
            cqp: Cqp::new(13),
            record_audio: true,
            output_path: None,
        }
    }
    pub fn set_window_title(&mut self, window_title: String) {
        self.window_title = Some(window_title);
    }
    pub fn set_output_resolution(&mut self, resolution: Resolution) {
        self.output_resolution = resolution;
    }
    pub fn set_input_resolution(&mut self, resolution: Resolution) {
        self.input_resolution = resolution;
    }
    pub fn set_framerate(&mut self, framerate: Framerate) {
        self.framerate = framerate;
    }
    pub fn set_bitrate(&mut self, bitrate: Bitrate) {
        self.bitrate = bitrate;
    }
    pub fn set_cqp(&mut self, cqp: Cqp) {
        self.cqp = cqp;
    }
    pub fn record_audio(&mut self, record_audio: bool) {
        self.record_audio = record_audio;
    }
    pub fn set_output_path(&mut self, output_path: String) {
        self.output_path = Some(output_path);
    }
}

pub struct Recorder {
    video_source: Option<*mut obs_source>,
    audio_source: Option<*mut obs_source>,
    video_encoder: Option<*mut obs_encoder>,
    audio_encoder: Option<*mut obs_encoder>,
    output: Option<*mut obs_output>,
    configured: bool,
    recording: bool,
}

impl Recorder {
    pub fn create() -> Result<Self, String> {
        if unsafe { obs_initialized() } {
            return Err(String::from("error: can only create one recorder object"));
        }

        let mut get = Get::new();
        unsafe {
            // STARTUP
            base_set_log_handler(Some(log_handler), null_mut());

            if !obs_startup(get.c_str("en-US"), null_mut(), null_mut()) {
                return Err(String::from("error on libobs startup"));
            }

            obs_add_data_path(get.c_str(LIBOBS_DATA_PATH));
            obs_add_module_path(get.c_str(PLUGIN_BIN_PATH), get.c_str(PLUGIN_DATA_PATH));
            obs_load_all_modules();

            if DEBUG {
                obs_log_loaded_modules();
            }

            let size = Size::new(DEFAULT_WIDTH, DEFAULT_HEIGHT);
            let framerate = Framerate::new(DEFAULT_FRAMERATE);
            Self::reset_video(size, size, framerate)?;
            Self::reset_audio()?;

            obs_post_load_modules();
        }

        Ok(Recorder {
            video_source: None,
            audio_source: None,
            video_encoder: None,
            audio_encoder: None,
            output: None,
            configured: false,
            recording: false,
        })
    }

    pub fn configure(&mut self, settings: RecorderSettings) -> Result<(), String> {
        if self.recording {
            return Err(String::from(
                "error: cannot change configuration while recording",
            ));
        }
        if settings.window_title.is_none() || settings.output_path.is_none() {
            return Err(String::from(
                "error: it is required to set a window title and an output path",
            ));
        }

        // REMOVE OLD SOURCES/ENCODERS
        self.release_all();

        // RESET VIDEO
        let input_size = settings.input_resolution.get_size();
        let output_size = settings.output_resolution.get_size();
        Self::reset_video(input_size, output_size, settings.framerate)?;

        let mut get = Get::new();
        unsafe {
            // SETUP NEW VIDEO SOURCE/ENCODER
            let video_source = {
                let mut data = ObsData::new();
                data.set_string("capture_mode", "window");
                data.set_string("window", settings.window_title.unwrap());

                obs_source_create(
                    get.c_str("game_capture"),
                    get.c_str(""),
                    data.get_obs_data(),
                    null_mut(),
                )
            };
            obs_set_output_source(0, video_source);
            self.video_source = Some(video_source);

            let video_encoder = {
                let mut data = ObsData::new();
                data.set_string("rate_control", "CBR");
                data.set_int("bitrate", settings.bitrate);

                let encoder_settings = if settings.bitrate.is_valid() {
                    data.get_obs_data()
                } else {
                    null_mut()
                };

                obs_video_encoder_create(
                    get.c_str("amd_amf_h264"),
                    get.c_str(""),
                    encoder_settings,
                    null_mut(),
                )
            };
            obs_encoder_set_video(video_encoder, obs_get_video());
            self.video_encoder = Some(video_encoder);

            // SETUP NEW AUDIO SOURCE/ENCODER
            if settings.record_audio {
                let audio_source = obs_source_create(
                    get.c_str("wasapi_output_capture"),
                    get.c_str(""),
                    null_mut(),
                    null_mut(),
                );
                obs_set_output_source(1, audio_source);
                self.audio_source = Some(audio_source);

                let audio_encoder = obs_audio_encoder_create(
                    get.c_str("ffmpeg_aac"),
                    get.c_str(""),
                    null_mut(),
                    0,
                    null_mut(),
                );
                obs_encoder_set_audio(audio_encoder, obs_get_audio());
                self.audio_encoder = Some(audio_encoder);
            }

            // SETUP NEW OUTPUT
            let output = {
                let mut data = ObsData::new();
                data.set_string("path", settings.output_path.unwrap());
                obs_output_create(
                    get.c_str("ffmpeg_muxer"),
                    get.c_str(""),
                    data.get_obs_data(),
                    null_mut(),
                )
            };

            obs_output_set_video_encoder(output, self.video_encoder.unwrap());
            if let Some(ae) = self.audio_encoder {
                obs_output_set_audio_encoder(output, ae, 0);
            }
            self.output = Some(output);
        }

        self.configured = true;
        Ok(())
    }

    pub fn start_recording(&mut self) -> bool {
        if self.recording || !self.configured || self.output.is_none() {
            return false;
        }

        if unsafe { obs_output_start(self.output.unwrap()) } {
            self.recording = true;
        }
        self.recording
    }

    pub fn stop_recording(&mut self) -> bool {
        if !self.recording {
            return false;
        }

        unsafe { obs_output_stop(self.output.unwrap()) }
        self.recording = false;
        self.recording
    }

    fn release_all(&mut self) {
        unsafe {
            // video
            if let Some(source) = self.video_source {
                obs_source_remove(source);
                obs_source_release(source);
                self.video_source = None;
            }
            if let Some(encoder) = self.video_encoder {
                obs_encoder_release(encoder);
                self.video_encoder = None;
            }
            // audio
            if let Some(source) = self.audio_source {
                obs_source_remove(source);
                obs_source_release(source);
                self.audio_source = None;
            }
            if let Some(encoder) = self.audio_encoder {
                obs_encoder_release(encoder);
                self.audio_encoder = None;
            }
            // output
            if let Some(out) = self.output {
                obs_output_release(out);
                self.output = None;
            }
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
                scale_type: video_scale_type_VIDEO_SCALE_BILINEAR,
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
        self.release_all();
        unsafe {
            obs_shutdown();
            if DEBUG {
                println!("{}", bnum_allocs());
            }
        }
    }
}
