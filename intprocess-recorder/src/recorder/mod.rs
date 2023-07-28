use std::{
    cell::Cell,
    ffi::CStr,
    mem::MaybeUninit,
    os::raw::c_char,
    ptr::{addr_of_mut, null_mut, NonNull},
    sync::{Once, OnceLock},
    thread::{self, ThreadId},
    time::Duration,
};

use libobs_sys::{
    base_set_log_handler, bnum_allocs, obs_add_data_path, obs_add_module_path, obs_audio_encoder_create,
    obs_audio_info, obs_encoder, obs_encoder_release, obs_encoder_set_audio, obs_encoder_set_video, obs_encoder_update,
    obs_enum_encoder_types, obs_get_audio, obs_get_encoder_by_name, obs_get_output_by_name, obs_get_source_by_name,
    obs_get_video, obs_get_video_info, obs_initialized, obs_load_all_modules, obs_log_loaded_modules, obs_output,
    obs_output_active, obs_output_create, obs_output_force_stop, obs_output_release, obs_output_set_audio_encoder,
    obs_output_set_video_encoder, obs_output_start, obs_output_stop, obs_output_update, obs_post_load_modules,
    obs_reset_audio, obs_reset_video, obs_scale_type_OBS_SCALE_LANCZOS, obs_set_output_source, obs_shutdown,
    obs_source, obs_source_create, obs_source_release, obs_source_update, obs_startup, obs_video_encoder_create,
    obs_video_info, speaker_layout_SPEAKERS_STEREO, va_list, video_colorspace_VIDEO_CS_709,
    video_format_VIDEO_FORMAT_NV12, video_range_type_VIDEO_RANGE_DEFAULT, OBS_VIDEO_SUCCESS,
};

use crate::settings::{AudioSource, Encoder, Framerate, RateControl, RecorderSettings, Size};

use self::{get::Get, obs_data::ObsData};

mod get;
pub(crate) mod obs_data;

#[cfg(feature = "debug")]
const DEBUG: bool = true;
#[cfg(not(feature = "debug"))]
const DEBUG: bool = false;

#[cfg(target_os = "windows")]
const GRAPHICS_MODULE: &str = "libobs-d3d11.dll";
#[cfg(not(target_os = "windows"))]
const GRAPHICS_MODULE: &str = "libobs-opengl.dll";

// default asset paths
const DEFAULT_LIBOBS_DATA_PATH: &str = "./data/libobs/";
const DEFAULT_PLUGIN_BIN_PATH: &str = "./obs-plugins/64bit/";
const DEFAULT_PLUGIN_DATA_PATH: &str = "./data/obs-plugins/%module%/";

// define null terminated libobs object names for ffi
const OUTPUT: *const i8 = b"output\0".as_ptr().cast();
const VIDEO_ENCODER: *const i8 = b"video_encoder\0".as_ptr().cast();
const AUDIO_ENCODER: *const i8 = b"audio_encoder\0".as_ptr().cast();
const VIDEO_SOURCE: *const i8 = b"video_source\0".as_ptr().cast();
const AUDIO_SOURCE1: *const i8 = b"audio_source1\0".as_ptr().cast();
const AUDIO_SOURCE2: *const i8 = b"audio_source2\0".as_ptr().cast();
const AUDIO_SOURCE3: *const i8 = b"audio_source3\0".as_ptr().cast();

// libobs output channel assignments
const VIDEO_CHANNEL: u32 = 0;
const AUDIO_CHANNEL1: u32 = 1;
const AUDIO_CHANNEL2: u32 = 2;
const AUDIO_CHANNEL3: u32 = 3;

static LIBOBS_THREAD: OnceLock<ThreadId> = OnceLock::new();
static LIBOBS_SHUTDOWN: Once = Once::new();

// stores how many instances of Recorder exist in each thread
// it is only possible to create instances of Recorder on one thread due to LIBOBS_THREAD
thread_local! {
    static REF_COUNT: Cell<u32> = Cell::new(0);
    static CURRENT_ENCODER: Cell<Encoder> = Cell::new(Encoder::OBS_X264);
}

type PhantomUnsync = std::marker::PhantomData<std::cell::Cell<()>>;
type PhantomUnsend = std::marker::PhantomData<std::sync::MutexGuard<'static, ()>>;

pub struct InpRecorder {
    output: NonNull<obs_output>,
    video_encoder: Cell<NonNull<obs_encoder>>,
    audio_encoder: NonNull<obs_encoder>,
    video_source: NonNull<obs_source>,
    audio_source1: NonNull<obs_source>,
    audio_source2: NonNull<obs_source>,
    audio_source3: NonNull<obs_source>,

    _phantom: std::marker::PhantomData<(PhantomUnsend, PhantomUnsync)>,
}

// implement associated functions
impl InpRecorder {
    /// # Panics
    /// Panics if the libobs initialization sequence fails.
    ///
    /// This can happen because the necessary DLLs are missing or some other necessary files can not be found.
    /// If the `initialize` function runs once without panicking for a certain environment (DLLs, config files, ...)
    /// it is garuanteed to never panic as long as the environment stays the same. If it does it is a bug.
    pub fn initialize(
        libobs_data_path: Option<&str>,
        plugin_bin_path: Option<&str>,
        plugin_data_path: Option<&str>,
    ) -> Result<(), &'static str> {
        // libobs currently cant be reinitialized after being shutdown
        // I assume this is a limitation of the library
        if LIBOBS_SHUTDOWN.is_completed() {
            return Err("libobs has already been shut down");
        }

        if LIBOBS_THREAD.get().is_some() {
            return Err("libobs has already been initialized");
        }

        LIBOBS_THREAD.get_or_init(|| {
            // set defaults in case no arguments were provided
            let libobs_data_path = libobs_data_path.unwrap_or(DEFAULT_LIBOBS_DATA_PATH);
            let plugin_bin_path = plugin_bin_path.unwrap_or(DEFAULT_PLUGIN_BIN_PATH);
            let plugin_data_path = plugin_data_path.unwrap_or(DEFAULT_PLUGIN_DATA_PATH);
            if let Err(e) = Self::init_internal(libobs_data_path, plugin_bin_path, plugin_data_path) {
                panic!("Error initializing libobs: {e}");
            }

            std::thread::current().id()
        });

        if DEBUG {
            println!("libobs initialized");
        }

        Ok(())
    }

    fn init_internal(
        libobs_data_path: &str,
        plugin_bin_path: &str,
        plugin_data_path: &str,
    ) -> Result<(), &'static str> {
        unsafe {
            // INITIALIZE
            let mut get = Get::new();
            if !DEBUG {
                base_set_log_handler(Some(Self::empty_log_handler), null_mut());
            }

            if !obs_startup(get.c_str("en-US"), null_mut(), null_mut()) {
                return Err("libobs startup failed");
            }

            let default_fps = Framerate::new(30, 1);
            let default_size = Size::new(1920, 1080);
            obs_add_data_path(get.c_str(libobs_data_path));
            Self::reset_video(default_size, default_size, default_fps).expect("unable to initialize video");
            Self::reset_audio().expect("unable to initialize audio");

            obs_add_module_path(get.c_str(plugin_bin_path), get.c_str(plugin_data_path));
            obs_load_all_modules();
            obs_post_load_modules();
            if DEBUG {
                obs_log_loaded_modules();
            }

            // CREATE OUTPUT
            let mut data = ObsData::new();
            data.set_string("path", "./recording.mp4");
            let output = obs_output_create(get.c_str("ffmpeg_muxer"), OUTPUT, data.as_ptr(), null_mut());

            // choose 'best' encoder
            let encoders = Self::get_available_encoders_internal();
            if encoders.is_empty() {
                return Err("no encoder available");
            }
            let current_encoder = *encoders.first().unwrap();
            Self::set_current_encoder(current_encoder);

            // CREATE VIDEO ENCODER
            let mut get = Get::new();
            let data: ObsData = current_encoder.settings(RateControl::default());
            let video_encoder = obs_video_encoder_create(
                get.c_str(current_encoder.id()),
                VIDEO_ENCODER,
                data.as_ptr(),
                null_mut(),
            );
            obs_encoder_set_video(video_encoder, obs_get_video());
            obs_output_set_video_encoder(output, video_encoder);

            // CREATE VIDEO SOURCE
            let mut data = ObsData::new();
            data.set_string("capture_mode", "window");
            data.set_string("window", "");
            data.set_bool("capture_cursor", true);
            let video_source = obs_source_create(
                get.c_str("game_capture"),
                VIDEO_SOURCE,
                data.as_ptr(),
                std::ptr::null_mut(),
            );
            obs_set_output_source(VIDEO_CHANNEL, video_source);

            // CREATE AUDIO ENCODER
            let mut data = ObsData::new();
            data.set_int("bitrate", 160);
            let audio_encoder =
                obs_audio_encoder_create(get.c_str("ffmpeg_aac"), AUDIO_ENCODER, data.as_ptr(), 0, null_mut());
            obs_encoder_set_audio(audio_encoder, obs_get_audio());
            obs_output_set_audio_encoder(
                output,
                audio_encoder,
                0, // ignored since we only have 1 output
            );

            // CREATE AUDIO SOURCE 1
            obs_source_create(
                get.c_str("wasapi_process_output_capture"),
                AUDIO_SOURCE1,
                null_mut(),
                null_mut(),
            );

            // CREATE AUDIO SOURCE 2
            let mut data = ObsData::new();
            data.set_string("device_id", "default");
            let audio_source2 = obs_source_create(
                get.c_str("wasapi_output_capture"),
                AUDIO_SOURCE2,
                data.as_ptr(),
                null_mut(),
            );
            obs_set_output_source(AUDIO_CHANNEL2, audio_source2);

            // CREATE AUDIO SOURCE 3
            let mut data = ObsData::new();
            data.set_string("device_id", "default");
            obs_source_create(
                get.c_str("wasapi_input_capture"),
                AUDIO_SOURCE3,
                data.as_ptr(),
                null_mut(),
            );
        }

        Ok(())
    }

    pub fn get_handle() -> Result<Self, &'static str> {
        if LIBOBS_THREAD.get() != Some(&thread::current().id()) {
            return Err("wrong thread - only able to get handle to recorder in the thread in which ");
        }

        unsafe {
            let output = NonNull::new(obs_get_output_by_name(OUTPUT)).ok_or("got nullpointer instead of output")?;
            let video_encoder = Cell::new(
                NonNull::new(obs_get_encoder_by_name(VIDEO_ENCODER))
                    .ok_or("got nullpointer instead of video encoder")?,
            );
            let audio_encoder = NonNull::new(obs_get_encoder_by_name(AUDIO_ENCODER))
                .ok_or("got nullpointer instead of audio encoder")?;
            let video_source =
                NonNull::new(obs_get_source_by_name(VIDEO_SOURCE)).ok_or("got nullpointer instead of video source")?;
            let audio_source1 = NonNull::new(obs_get_source_by_name(AUDIO_SOURCE1))
                .ok_or("got nullpointer instead of audio source 1")?;
            let audio_source2 = NonNull::new(obs_get_source_by_name(AUDIO_SOURCE2))
                .ok_or("got nullpointer instead of audio source2")?;
            let audio_source3 = NonNull::new(obs_get_source_by_name(AUDIO_SOURCE3))
                .ok_or("got nullpointer instead of audio source3")?;

            Self::increment_refcount();

            Ok(Self {
                output,
                video_encoder,
                audio_encoder,
                video_source,
                audio_source1,
                audio_source2,
                audio_source3,
                _phantom: std::marker::PhantomData,
            })
        }
    }

    pub fn shutdown() -> Result<(), &'static str> {
        if unsafe { !obs_initialized() } {
            return Err("libobs was never initialized");
        }

        if LIBOBS_SHUTDOWN.is_completed() {
            return Ok(());
        }
        if Self::get_refcount() > 0 {
            return Err("libobs can't be shut down due to existing Recorder instances");
        }

        unsafe {
            obs_shutdown();
            LIBOBS_SHUTDOWN.call_once(|| ());
        }
        Ok(())
    }

    fn get_video_info() -> Result<obs_video_info, &'static str> {
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

        if unsafe { obs_get_video_info(addr_of_mut!(ovi)) } {
            Ok(ovi)
        } else {
            Err("Error video was not set! Maybe Recorder was not initialized?")
        }
    }

    fn reset_video(input_size: Size, output_size: Size, framerate: Framerate) -> Result<(), &'static str> {
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
                colorspace: video_colorspace_VIDEO_CS_709,
                range: video_range_type_VIDEO_RANGE_DEFAULT,
                scale_type: obs_scale_type_OBS_SCALE_LANCZOS,
            };

            if obs_reset_video(addr_of_mut!(ovi)) != OBS_VIDEO_SUCCESS.try_into().unwrap() {
                return Err("error on libobs reset video");
            }
        }
        Ok(())
    }

    /// only call this function once on startup
    /// resetting audio after initialisation crashes libobs
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

    fn get_available_encoders_internal() -> Vec<Encoder> {
        // GET AVAILABLE ENCODERS
        let mut n = 0;
        let mut encoders = Vec::new();
        let mut ptr = MaybeUninit::<*const c_char>::uninit();
        unsafe {
            while obs_enum_encoder_types(n, ptr.as_mut_ptr()) {
                n += 1;
                let encoder = ptr.assume_init();
                if let Ok(enc) = CStr::from_ptr(encoder).to_str() {
                    let Ok(enc) = Encoder::try_from(enc) else { continue };
                    encoders.push(enc);
                }
            }
        }
        encoders.sort();
        encoders
    }

    unsafe extern "C" fn empty_log_handler(
        _lvl: ::std::os::raw::c_int,
        _msg: *const ::std::os::raw::c_char,
        _args: va_list,
        _p: *mut ::std::os::raw::c_void,
    ) {
        // empty function to block logs
    }

    fn set_current_encoder(encoder: Encoder) {
        CURRENT_ENCODER.with(|cell| cell.set(encoder));
    }

    fn get_current_encoder() -> Encoder {
        CURRENT_ENCODER.with(Cell::get)
    }

    fn increment_refcount() {
        REF_COUNT.with(|cell| cell.set(cell.get() + 1));
    }

    fn decrement_refcount() {
        REF_COUNT.with(|cell| cell.set(cell.get() - 1));
    }

    fn get_refcount() -> u32 {
        REF_COUNT.with(Cell::get)
    }
}

impl InpRecorder {
    pub fn start_recording(&mut self) -> bool {
        if DEBUG {
            println!("Recording Start: {}", unsafe { bnum_allocs() });
        }
        if self.is_recording() {
            true // return true if already recording
        } else {
            unsafe { obs_output_start(self.output.as_ptr()) }
        }
    }

    pub fn stop_recording(&mut self) {
        if self.is_recording() {
            unsafe { obs_output_stop(self.output.as_ptr()) }
            if DEBUG {
                println!("Recording Stop: {}", unsafe { bnum_allocs() });
            }
        }

        let now = std::time::Instant::now();
        loop {
            thread::sleep(Duration::from_millis(100));
            if !self.is_recording() {
                return;
            } else if now.elapsed().as_millis() > 3000 {
                unsafe { obs_output_force_stop(self.output.as_ptr()) };
                return;
            }
        }
    }

    pub fn configure(&self, settings: &RecorderSettings) -> Result<(), &'static str> {
        if self.is_recording() {
            return Err("can't change settings while recording");
        }

        if DEBUG {
            println!("before get: {}", unsafe { bnum_allocs() });
        }

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
        let framerate = match settings.framerate {
            Some(framerate) => framerate,
            None => Framerate::new(ovi.fps_num, ovi.fps_den),
        };

        let video_reset_necessary = input_size.width() != ovi.base_width
            || input_size.height() != ovi.base_height
            || output_size.width() != ovi.output_width
            || output_size.height() != ovi.output_height
            || framerate.num() != ovi.fps_num
            || framerate.den() != ovi.fps_den;
        if video_reset_necessary {
            Self::reset_video(input_size, output_size, framerate)?;

            unsafe {
                // reconfigure video output pipeline after resetting the video backend
                obs_encoder_set_video(self.video_encoder.get().as_ptr(), obs_get_video());
                obs_output_set_video_encoder(self.output.as_ptr(), self.video_encoder.get().as_ptr());
                obs_set_output_source(VIDEO_CHANNEL, self.video_source.as_ptr());
            }
        }

        let mut get = Get::new();
        unsafe {
            // OUTPUT
            if let Some(output_path) = &settings.output_path {
                let mut data = ObsData::new();
                data.set_string("path", output_path);
                obs_output_update(self.output.as_ptr(), data.as_ptr());
            }

            // VIDEO ENCODER
            // create a new encoder if there is none or if it is different from the previously selected encoder
            // or update the encoder if rate_control is Some()
            if let Some(new_encoder) = settings.encoder {
                if new_encoder != Self::get_current_encoder() {
                    // create new encoder
                    let data = new_encoder.settings(settings.rate_control.unwrap_or_default());
                    let new_video_encoder = NonNull::new(obs_video_encoder_create(
                        get.c_str(new_encoder.id()),
                        get.c_str("video_encoder"),
                        data.as_ptr(),
                        null_mut(),
                    ))
                    .expect("unable to create video encoder");

                    obs_encoder_set_video(new_video_encoder.as_ptr(), obs_get_video());
                    obs_output_set_video_encoder(self.output.as_ptr(), new_video_encoder.as_ptr());

                    // replace and release old encoder
                    obs_encoder_release(self.video_encoder.replace(new_video_encoder).as_ptr());
                }
            } else if let Some(rate_control) = settings.rate_control {
                let data = Self::get_current_encoder().settings(rate_control);
                obs_encoder_update(self.video_encoder.get().as_ptr(), data.as_ptr());
            }

            // VIDEO SOURCE
            if let Some(window) = settings.window.as_ref() {
                let mut data = ObsData::new();
                data.set_string("window", window.get_libobs_window_id());
                obs_source_update(self.video_source.as_ptr(), data.as_ptr());
            }

            if let Some(audio_setting) = settings.record_audio {
                // AUDIO SOURCE 1
                let audio_source1 = match audio_setting {
                    AudioSource::APPLICATION => {
                        if let Some(window) = settings.window.as_ref() {
                            let mut data = ObsData::new();
                            data.set_string("window", window.get_libobs_window_id());
                            obs_source_update(self.audio_source1.as_ptr(), data.as_ptr());
                        };
                        self.audio_source1.as_ptr()
                    }
                    _ => null_mut(),
                };
                obs_set_output_source(AUDIO_CHANNEL1, audio_source1);

                // AUDIO SOURCE 2
                let audio_source2 = match audio_setting {
                    AudioSource::SYSTEM | AudioSource::ALL => self.audio_source2.as_ptr(),
                    _ => null_mut(),
                };
                obs_set_output_source(AUDIO_CHANNEL2, audio_source2);

                // AUDIO SOURCE 3
                let audio_source3 = match audio_setting {
                    AudioSource::ALL => self.audio_source3.as_ptr(),
                    _ => null_mut(),
                };
                obs_set_output_source(AUDIO_CHANNEL3, audio_source3);
            }

            if DEBUG {
                println!("after get: {}", bnum_allocs());
            }
        }

        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        unsafe { obs_output_active(self.output.as_ptr()) }
    }

    pub fn get_available_encoders(&self) -> Vec<Encoder> {
        // public version of internal function that is only available after libobs is initialized
        // due to requiring &self
        Self::get_available_encoders_internal()
    }
}

impl Drop for InpRecorder {
    fn drop(&mut self) {
        unsafe {
            // output
            obs_output_release(self.output.as_ptr());
            // video
            obs_encoder_release(self.video_encoder.get().as_ptr());
            obs_source_release(self.video_source.as_ptr());
            // audio
            obs_encoder_release(self.audio_encoder.as_ptr());
            obs_source_release(self.audio_source1.as_ptr());
            obs_source_release(self.audio_source2.as_ptr());
            obs_source_release(self.audio_source3.as_ptr());

            if DEBUG {
                println!("bnum_allocs: {}", bnum_allocs());
            }
        }

        Self::decrement_refcount();
    }
}
