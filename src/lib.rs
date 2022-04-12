extern crate libobs_sys;
use std::{
    ffi::{CStr, CString},
    ptr::null_mut as nullptr,
};

use libobs_sys::{
    base_set_log_handler, bnum_allocs, obs_add_data_path, obs_add_module_path,
    obs_audio_encoder_create, obs_audio_info, obs_data_create, obs_data_release, obs_data_set_int,
    obs_data_set_string, obs_encoder, obs_encoder_release, obs_encoder_set_audio,
    obs_encoder_set_video, obs_get_audio, obs_get_video, obs_initialized, obs_load_all_modules,
    obs_log_loaded_modules, obs_output, obs_output_create, obs_output_release,
    obs_output_set_audio_encoder, obs_output_set_video_encoder, obs_output_start, obs_output_stop,
    obs_post_load_modules, obs_reset_audio, obs_reset_video, obs_set_output_source, obs_shutdown,
    obs_source, obs_source_create, obs_source_release, obs_source_remove, obs_startup,
    obs_video_encoder_create, obs_video_info, speaker_layout_SPEAKERS_STEREO, va_list,
    video_colorspace_VIDEO_CS_DEFAULT, video_format_VIDEO_FORMAT_NV12,
    video_range_type_VIDEO_RANGE_DEFAULT, video_scale_type_VIDEO_SCALE_BILINEAR, OBS_VIDEO_SUCCESS,
};

use bitrate::Bitrate;
use framerate::Framerate;
use resolution::Resolution;

pub mod bitrate;
pub mod framerate;
pub mod resolution;

const DEBUG: bool = true;

#[cfg(target_os = "windows")]
const GRAPHICS_MODULE: &str = "libobs-d3d11.dll";
#[cfg(not(target_os = "windows"))]
const GRAPHICS_MODULE: &str = "libobs-opengl.dll";

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;

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
            bitrate: Bitrate::auto(),
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
    pub fn new() -> Result<Self, String> {
        if unsafe { obs_initialized() } {
            return Err(String::from("error: can only create one recorder object"));
        }

        // C STRINGS ----------------------------------------
        let locale = CString::new("en-US").unwrap();

        let data_path = CString::new("./data/libobs/").unwrap();
        let module_path = CString::new("./obs-plugins/64bit/").unwrap();
        let module_data_path = CString::new("./data/obs-plugins/%module%/").unwrap();

        let graphics_module = CString::new(GRAPHICS_MODULE).unwrap();
        // --------------------------------------------------

        let mut ovi = obs_video_info {
            adapter: 0,
            graphics_module: graphics_module.as_ptr(),
            fps_num: 30,
            fps_den: 1,
            base_width: DEFAULT_WIDTH,
            base_height: DEFAULT_HEIGHT,
            output_width: DEFAULT_WIDTH,
            output_height: DEFAULT_HEIGHT,
            output_format: video_format_VIDEO_FORMAT_NV12,
            gpu_conversion: true,
            colorspace: video_colorspace_VIDEO_CS_DEFAULT,
            range: video_range_type_VIDEO_RANGE_DEFAULT,
            scale_type: video_scale_type_VIDEO_SCALE_BILINEAR,
        };

        let ai = obs_audio_info {
            samples_per_sec: 44100,
            speakers: speaker_layout_SPEAKERS_STEREO,
        };

        unsafe {
            // STARTUP
            base_set_log_handler(Some(log_handler), nullptr());

            if !obs_startup(locale.as_ptr(), nullptr(), nullptr()) {
                return Err(String::from("error on libobs startup"));
            }

            obs_add_data_path(data_path.as_ptr());
            obs_add_module_path(module_path.as_ptr(), module_data_path.as_ptr());
            obs_load_all_modules();
            if DEBUG {
                obs_log_loaded_modules();
            }

            let ret = obs_reset_video(&mut ovi as *mut _);
            if ret != OBS_VIDEO_SUCCESS.try_into().unwrap() {
                return Err(String::from("error on libobs reset video"));
            }

            if !obs_reset_audio(&ai) {
                return Err(String::from("error on libobs reset audio"));
            }

            obs_post_load_modules();

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

        // C STRINGS ----------------------------------------
        let empty = CString::new("").unwrap();

        let graphics_module = CString::new(GRAPHICS_MODULE).unwrap();

        let capture_mode_string = CString::new("capture_mode").unwrap();
        let window_string = CString::new("window").unwrap();
        let window_title = CString::new(settings.window_title.unwrap()).unwrap();
        let video_source_type = CString::new("game_capture").unwrap();

        let video_encoder_type = CString::new("amd_amf_h264").unwrap();
        let bitrate_string = CString::new("window").unwrap();

        let audio_source_type = CString::new("wasapi_output_capture").unwrap();
        let audio_encoder_type = CString::new("ffmpeg_aac").unwrap();

        let path_string = CString::new("path").unwrap();
        let path = CString::new(settings.output_path.unwrap()).unwrap();
        let output_type = CString::new("ffmpeg_muxer").unwrap();
        // --------------------------------------------------

        unsafe {
            // RESET VIDEO
            let input_size = settings.input_resolution.get_size();
            let output_size = settings.output_resolution.get_size();

            let mut ovi = obs_video_info {
                adapter: 0,
                graphics_module: graphics_module.as_ptr(),
                fps_num: settings.framerate.into(),
                fps_den: 1,
                base_width: input_size.get_width(),
                base_height: input_size.get_height(),
                output_width: output_size.get_width(),
                output_height: output_size.get_height(),
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

            // REMOVE OLD SOURCES/ENCODERS
            self.release_all();

            // SETUP NEW VIDEO SOURCE/ENCODER
            let video_source_settings = obs_data_create();
            obs_data_set_string(
                video_source_settings,
                capture_mode_string.as_ptr(),
                window_string.as_ptr(),
            );
            obs_data_set_string(
                video_source_settings,
                window_string.as_ptr(),
                window_title.as_ptr(),
            );
            let video_source = obs_source_create(
                video_source_type.as_ptr(),
                empty.as_ptr(),
                video_source_settings,
                nullptr(),
            );
            obs_data_release(video_source_settings);
            obs_set_output_source(0, video_source);

            let video_encoder_settings = obs_data_create();
            obs_data_set_int(
                video_source_settings,
                bitrate_string.as_ptr(),
                settings.bitrate.into(),
            );
            let video_encoder = obs_video_encoder_create(
                video_encoder_type.as_ptr(),
                empty.as_ptr(),
                if settings.bitrate.is_auto() {
                    nullptr()
                } else {
                    video_encoder_settings
                },
                nullptr(),
            );
            obs_data_release(video_encoder_settings);
            obs_encoder_set_video(video_encoder, obs_get_video());

            self.video_source = Some(video_source);
            self.video_encoder = Some(video_encoder);

            // SETUP NEW AUDIO SOURCE/ENCODER
            let audio_encoder = if settings.record_audio {
                let audio_source = obs_source_create(
                    audio_source_type.as_ptr(),
                    empty.as_ptr(),
                    nullptr(),
                    nullptr(),
                );
                obs_set_output_source(1, audio_source);

                let audio_encoder = obs_audio_encoder_create(
                    audio_encoder_type.as_ptr(),
                    empty.as_ptr(),
                    nullptr(),
                    0,
                    nullptr(),
                );
                obs_encoder_set_audio(audio_encoder, obs_get_audio());

                self.audio_source = Some(audio_source);
                self.audio_encoder = Some(audio_encoder);

                Some(audio_encoder)
            } else {
                None
            };

            // SETUP NEW OUTPUT
            let output_settings = obs_data_create();
            obs_data_set_string(output_settings, path_string.as_ptr(), path.as_ptr());
            let output = obs_output_create(
                output_type.as_ptr(),
                empty.as_ptr(),
                output_settings,
                nullptr(),
            );
            obs_data_release(output_settings);

            obs_output_set_video_encoder(output, video_encoder);
            if let Some(ae) = audio_encoder {
                obs_output_set_audio_encoder(output, ae, 0);
            }

            self.output = Some(output);
        }

        self.configured = true;
        Ok(())
    }

    fn release_all(&mut self) {
        unsafe {
            // video
            if let Some(source) = self.video_source {
                obs_source_remove(source);
                obs_source_release(source);
            }
            if let Some(encoder) = self.video_encoder {
                obs_encoder_release(encoder);
            }
            // audio
            if let Some(source) = self.audio_source {
                obs_source_remove(source);
                obs_source_release(source);
            }
            if let Some(encoder) = self.audio_encoder {
                obs_encoder_release(encoder);
            }
            // output
            if let Some(out) = self.output {
                obs_output_release(out);
            }
        }
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
