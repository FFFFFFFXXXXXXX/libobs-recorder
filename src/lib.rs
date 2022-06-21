extern crate libobs_sys;

use libobs_sys::{
    base_set_log_handler, bnum_allocs, obs_add_data_path, obs_add_module_path,
    obs_audio_encoder_create, obs_audio_info, obs_encoder, obs_encoder_release,
    obs_encoder_set_audio, obs_encoder_set_video, obs_enum_encoder_types, obs_get_audio,
    obs_get_video, obs_get_video_info, obs_initialized, obs_load_all_modules,
    obs_log_loaded_modules, obs_output, obs_output_create, obs_output_release,
    obs_output_set_audio_encoder, obs_output_set_video_encoder, obs_output_start, obs_output_stop,
    obs_output_update, obs_post_load_modules, obs_reset_audio, obs_reset_video,
    obs_scale_type_OBS_SCALE_BILINEAR, obs_set_output_source, obs_shutdown, obs_source,
    obs_source_create, obs_source_release, obs_source_remove, obs_startup,
    obs_video_encoder_create, obs_video_info, speaker_layout_SPEAKERS_STEREO, va_list,
    video_colorspace_VIDEO_CS_DEFAULT, video_format_VIDEO_FORMAT_NV12,
    video_range_type_VIDEO_RANGE_DEFAULT, OBS_VIDEO_SUCCESS,
};

use std::{ffi::CStr, mem::MaybeUninit, os::raw::c_char, ptr::null_mut};

pub use encoder::Encoder;
pub use framerate::Framerate;
use get::Get;
use obs_data::ObsData;
pub use resolution::*;
pub use settings::RecorderSettings;
pub use window::Window;

mod encoder;
mod framerate;
mod get;
mod obs_data;
pub mod rate_control;
mod resolution;
mod settings;
mod window;

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

const VIDEO_CHANNEL: u32 = 0;
const AUDIO_CHANNEL: u32 = 1;
const AUDIO_ENCODER_INDEX: usize = 0;

static mut ENCODER_TYPE: Encoder = Encoder::OBS_X264;

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
    ) -> Result<Encoder, String> {
        if unsafe { obs_initialized() } {
            return Err("error: obs already initialized".into());
        }

        // set defaults in case no arguments were provided
        let libobs_data_path = match libobs_data_path {
            Some(path) => path,
            None => LIBOBS_DATA_PATH.into(),
        };
        let plugin_bin_path = match plugin_bin_path {
            Some(path) => path,
            None => PLUGIN_BIN_PATH.into(),
        };
        let plugin_data_path = match plugin_data_path {
            Some(path) => path,
            None => PLUGIN_DATA_PATH.into(),
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
                Encoder::JIM_NVENC
            } else if ffmpeg_nvenc {
                Encoder::FFMPEG_NVENC
            } else if amf {
                Encoder::AMD_AMF_H264
            } else if qsv {
                Encoder::OBS_QSV11
            } else {
                Encoder::OBS_X264
            };

            Ok(ENCODER_TYPE)
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

        let window = match settings.window {
            Some(w) => w,
            None => return Err("No window options set".into()),
        };

        // RESET VIDEO
        let ovi = Self::get_video_info()?;
        let input_size = match settings.input_size {
            Some(size) => size,
            None => Size::new(ovi.base_width, ovi.base_height),
        };
        let output_size = match settings.output_resolution {
            Some(resolution) => resolution.get_size(),
            None => Size::new(ovi.output_width, ovi.output_height),
        };
        let framerate = match settings.framerate.is_set() {
            true => settings.framerate,
            false => Framerate::new(ovi.fps_num, ovi.fps_den),
        };

        let reset_necessary = input_size.width() != ovi.base_width
            || input_size.height() != ovi.base_height
            || output_size.width() != ovi.output_width
            || output_size.height() != ovi.output_height
            || framerate.num() != ovi.fps_num
            || framerate.den() != ovi.fps_den;
        if reset_necessary {
            Self::reset_video(input_size, output_size, framerate)?;
        }

        let mut get = Get::new();
        unsafe {
            // SETUP NEW VIDEO SOURCE
            // let mut data = ObsData::new();
            // data.set_string("capture_mode", "any_fullscreen");
            // data.set_bool("capture_cursor", true);
            #[cfg(target_os = "windows")]
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
            #[cfg(target_os = "linux")]
            let video_source = { todo!() };
            #[cfg(target_os = "macos")]
            let video_source = { todo!() };

            // SETUP NEW VIDEO ENCODER
            let encoder = settings.encoder.unwrap_or(ENCODER_TYPE);
            let video_encoder = {
                let data = encoder.settings(&settings.rate_control);
                let mut get = Get::new();
                obs_video_encoder_create(
                    get.c_str(encoder.id()),
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
            match settings.output_path {
                Some(output_path) => data.set_string("path", output_path),
                None => data.set_string("path", "./recording.mp4"),
            };
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

            obs_set_output_source(
                AUDIO_CHANNEL,
                match settings.record_audio {
                    true => audio_source,
                    false => null_mut(),
                },
            );
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

    pub fn set_output(&self, path: impl Into<String>) {
        let mut data = ObsData::new();
        data.set_string("path", path.into());
        unsafe { obs_output_update(self.output, data.get_ptr()) };
    }

    pub fn start_recording(&mut self) -> bool {
        if DEBUG {
            println!("Recording Start: {}", unsafe { bnum_allocs() });
        }
        self.recording = !self.recording && unsafe { obs_output_start(self.output) };
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
        match unsafe { obs_get_video_info(&mut ovi as *mut _) } {
            true => Ok(ovi),
            false => Err("Error video was not set! Maybe Recorder was not initialized?".into()),
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
