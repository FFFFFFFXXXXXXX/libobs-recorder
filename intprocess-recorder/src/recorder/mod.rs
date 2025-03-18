use std::cell::Cell;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr::{null_mut, NonNull};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::thread::{self, ThreadId};
use std::time::Duration;

use crate::settings::{Adapter, AdapterId, AudioSource, Encoder, Framerate, RateControl, RecorderSettings, Resolution};
use get::Get;
use obs_data::ObsData;

mod get;
pub(crate) mod obs_data;

#[cfg(target_os = "windows")]
const GRAPHICS_MODULE: &str = "libobs-d3d11.dll";
#[cfg(not(target_os = "windows"))]
const GRAPHICS_MODULE: &str = "libobs-opengl.dll";

// default asset paths
const DEFAULT_LIBOBS_DATA_PATH: &str = "./data/libobs/";
const DEFAULT_PLUGIN_BIN_PATH: &str = "./obs-plugins/64bit/";
const DEFAULT_PLUGIN_DATA_PATH: &str = "./data/obs-plugins/%module%/";

// define null terminated libobs object names for ffi
const OUTPUT: *const i8 = c"output".as_ptr().cast();
const VIDEO_ENCODER: *const i8 = c"video_encoder".as_ptr().cast();
const AUDIO_ENCODER: *const i8 = c"audio_encoder".as_ptr().cast();
const VIDEO_SOURCE: *const i8 = c"video_source".as_ptr().cast();
const AUDIO_SOURCE1: *const i8 = c"audio_source1".as_ptr().cast();
const AUDIO_SOURCE2: *const i8 = c"audio_source2".as_ptr().cast();
const AUDIO_SOURCE3: *const i8 = c"audio_source3".as_ptr().cast();

// libobs output channel assignments
const VIDEO_CHANNEL: u32 = 0;
const AUDIO_CHANNEL1: u32 = 1;
const AUDIO_CHANNEL2: u32 = 2;
const AUDIO_CHANNEL3: u32 = 3;

static LIBOBS_THREAD: OnceLock<ThreadId> = OnceLock::new();
static LIBOBS_SHUTDOWN: AtomicBool = AtomicBool::new(false);

// stores how many instances of Recorder exist in each thread
// it is only possible to create instances of Recorder on one thread due to LIBOBS_THREAD
//
// these are thread local so I don't have to make them thread-safe
thread_local! {
    static REF_COUNT: Cell<u32> = const { Cell::new(0) };
    static CURRENT_ENCODER: Cell<Encoder> = const { Cell::new(Encoder::OBS_X264) };
}

type PhantomUnsync = std::marker::PhantomData<Cell<()>>;
type PhantomUnsend = std::marker::PhantomData<*mut ()>;

pub struct InpRecorder {
    output: NonNull<libobs_sys::obs_output>,
    video_encoder: Cell<NonNull<libobs_sys::obs_encoder>>,
    audio_encoder: NonNull<libobs_sys::obs_encoder>,
    video_source: NonNull<libobs_sys::obs_source>,
    audio_source1: NonNull<libobs_sys::obs_source>,
    audio_source2: NonNull<libobs_sys::obs_source>,
    audio_source3: NonNull<libobs_sys::obs_source>,

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
        // I assume this is a limitation of libobs
        if LIBOBS_SHUTDOWN.load(Ordering::Acquire) {
            return Err("libobs has already been shut down");
        }

        if LIBOBS_THREAD.get().is_some() {
            return Err("libobs has already been initialized");
        }

        LIBOBS_THREAD.get_or_init(|| {
            if let Err(e) = Self::init_internal(libobs_data_path, plugin_bin_path, plugin_data_path) {
                println!("Error initializing libobs: {e}");
                panic!("Error initializing libobs: {e}");
            }

            thread::current().id()
        });

        println!("libobs {} initialized", libobs_sys::VERSION);

        Ok(())
    }

    fn init_internal(
        libobs_data_path: Option<&str>,
        plugin_bin_path: Option<&str>,
        plugin_data_path: Option<&str>,
    ) -> Result<(), &'static str> {
        // set defaults in case no arguments were provided
        let libobs_data_path = libobs_data_path.unwrap_or(DEFAULT_LIBOBS_DATA_PATH);
        let plugin_bin_path = plugin_bin_path.unwrap_or(DEFAULT_PLUGIN_BIN_PATH);
        let plugin_data_path = plugin_data_path.unwrap_or(DEFAULT_PLUGIN_DATA_PATH);

        // INITIALIZE
        let mut get = Get::new();

        if unsafe { !libobs_sys::obs_startup(get.c_str("en-US"), null_mut(), null_mut()) } {
            return Err("libobs startup failed");
        }

        let default_fps = Framerate::new(30, 1);
        let default_size = Resolution::new(1920, 1080);
        unsafe { libobs_sys::obs_add_data_path(get.c_str(libobs_data_path)) };
        Self::reset_video(default_size, default_size, default_fps).expect("unable to initialize video");
        Self::reset_audio().expect("unable to initialize audio");

        unsafe {
            libobs_sys::obs_add_module_path(get.c_str(plugin_bin_path), get.c_str(plugin_data_path));
            libobs_sys::obs_load_all_modules();
            libobs_sys::obs_post_load_modules();
            libobs_sys::obs_log_loaded_modules();
        }

        // CREATE OUTPUT
        let mut data = ObsData::new();
        data.set_string("path", "./recording.mp4");
        let output =
            unsafe { libobs_sys::obs_output_create(get.c_str("ffmpeg_muxer"), OUTPUT, data.as_ptr(), null_mut()) };

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
        let video_encoder = unsafe {
            libobs_sys::obs_video_encoder_create(
                get.c_str(current_encoder.id()),
                VIDEO_ENCODER,
                data.as_ptr(),
                null_mut(),
            )
        };
        unsafe {
            libobs_sys::obs_encoder_set_video(video_encoder, libobs_sys::obs_get_video());
            libobs_sys::obs_output_set_video_encoder(output, video_encoder);
        }

        // CREATE VIDEO SOURCE
        let mut data = ObsData::new();
        data.set_string("capture_mode", "window");
        data.set_string("window", "");
        data.set_bool("capture_cursor", true);
        let video_source = unsafe {
            libobs_sys::obs_source_create(
                get.c_str("game_capture"),
                VIDEO_SOURCE,
                data.as_ptr(),
                std::ptr::null_mut(),
            )
        };
        unsafe { libobs_sys::obs_set_output_source(VIDEO_CHANNEL, video_source) };

        // CREATE AUDIO ENCODER
        let mut data = ObsData::new();
        data.set_int("bitrate", 160);
        let audio_encoder = unsafe {
            libobs_sys::obs_audio_encoder_create(get.c_str("ffmpeg_aac"), AUDIO_ENCODER, data.as_ptr(), 0, null_mut())
        };
        unsafe {
            libobs_sys::obs_encoder_set_audio(audio_encoder, libobs_sys::obs_get_audio());
            libobs_sys::obs_output_set_audio_encoder(
                output,
                audio_encoder,
                0, // ignored since we only have 1 output
            );
        }

        // CREATE AUDIO SOURCE 1
        unsafe {
            libobs_sys::obs_source_create(
                get.c_str("wasapi_process_output_capture"),
                AUDIO_SOURCE1,
                null_mut(),
                null_mut(),
            )
        };

        // CREATE AUDIO SOURCE 2
        let mut data = ObsData::new();
        data.set_string("device_id", "default");
        let audio_source2 = unsafe {
            libobs_sys::obs_source_create(
                get.c_str("wasapi_output_capture"),
                AUDIO_SOURCE2,
                data.as_ptr(),
                null_mut(),
            )
        };
        unsafe { libobs_sys::obs_set_output_source(AUDIO_CHANNEL2, audio_source2) };

        // CREATE AUDIO SOURCE 3
        let mut data = ObsData::new();
        data.set_string("device_id", "default");
        unsafe {
            libobs_sys::obs_source_create(
                get.c_str("wasapi_input_capture"),
                AUDIO_SOURCE3,
                data.as_ptr(),
                null_mut(),
            )
        };

        Ok(())
    }

    pub fn get_handle() -> Result<Self, &'static str> {
        Self::check_thread_initialized()?;

        unsafe {
            let output =
                NonNull::new(libobs_sys::obs_get_output_by_name(OUTPUT)).ok_or("got nullpointer instead of output")?;
            let video_encoder = Cell::new(
                NonNull::new(libobs_sys::obs_get_encoder_by_name(VIDEO_ENCODER))
                    .ok_or("got nullpointer instead of video encoder")?,
            );
            let audio_encoder = NonNull::new(libobs_sys::obs_get_encoder_by_name(AUDIO_ENCODER))
                .ok_or("got nullpointer instead of audio encoder")?;
            let video_source = NonNull::new(libobs_sys::obs_get_source_by_name(VIDEO_SOURCE))
                .ok_or("got nullpointer instead of video source")?;
            let audio_source1 = NonNull::new(libobs_sys::obs_get_source_by_name(AUDIO_SOURCE1))
                .ok_or("got nullpointer instead of audio source 1")?;
            let audio_source2 = NonNull::new(libobs_sys::obs_get_source_by_name(AUDIO_SOURCE2))
                .ok_or("got nullpointer instead of audio source2")?;
            let audio_source3 = NonNull::new(libobs_sys::obs_get_source_by_name(AUDIO_SOURCE3))
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
        Self::check_thread_initialized()?;

        if LIBOBS_SHUTDOWN.load(Ordering::Acquire) {
            return Ok(());
        }

        if REF_COUNT.get() > 0 {
            return Err("libobs can't be shut down due to existing Recorder instances");
        }

        unsafe { libobs_sys::obs_shutdown() };
        LIBOBS_SHUTDOWN.store(true, Ordering::Release);

        Ok(())
    }

    fn get_video_info() -> Result<libobs_sys::obs_video_info, &'static str> {
        let mut ovi = libobs_sys::obs_video_info::default();
        if unsafe { libobs_sys::obs_get_video_info(&mut ovi) } {
            Ok(ovi)
        } else {
            Err("Error video was not set! Maybe Recorder was not initialized?")
        }
    }

    fn reset_video(input_size: Resolution, output_size: Resolution, framerate: Framerate) -> Result<(), &'static str> {
        unsafe {
            let mut get = Get::new();
            let mut ovi = libobs_sys::obs_video_info {
                adapter: AdapterId::default(),
                graphics_module: get.c_str(GRAPHICS_MODULE),
                fps_num: framerate.num(),
                fps_den: framerate.den(),
                base_width: input_size.width(),
                base_height: input_size.height(),
                output_width: output_size.width(),
                output_height: output_size.height(),
                output_format: libobs_sys::video_format_VIDEO_FORMAT_NV12,
                gpu_conversion: true,
                colorspace: libobs_sys::video_colorspace_VIDEO_CS_709,
                range: libobs_sys::video_range_type_VIDEO_RANGE_DEFAULT,
                scale_type: libobs_sys::obs_scale_type_OBS_SCALE_LANCZOS,
            };

            // OBS_VIDEO_SUCCESS is 0, so casting it to c_int should be fine
            if libobs_sys::obs_reset_video(&mut ovi) != libobs_sys::OBS_VIDEO_SUCCESS as c_int {
                return Err("error on libobs reset video");
            }
        }

        Ok(())
    }

    /// only call this function once on startup
    /// resetting audio after initialisation crashes libobs
    fn reset_audio() -> Result<(), String> {
        let ai = libobs_sys::obs_audio_info {
            samples_per_sec: 44100,
            speakers: libobs_sys::speaker_layout_SPEAKERS_STEREO,
        };
        let ok = unsafe { libobs_sys::obs_reset_audio(&ai) };
        if !ok {
            return Err(String::from("error on libobs reset audio"));
        }
        Ok(())
    }

    fn get_available_encoders_internal() -> Vec<Encoder> {
        let adapter = Self::get_adapters_internal()
            .into_iter()
            .find(|e| e.id() == AdapterId::default())
            .expect("no adapters found?");

        // GET AVAILABLE ENCODERS
        let mut n = 0;
        let mut encoders = Vec::new();
        let mut ptr: *const c_char = unsafe { std::mem::zeroed() };
        while unsafe { libobs_sys::obs_enum_encoder_types(n, &mut ptr) } {
            n += 1;
            let cstring = unsafe { CStr::from_ptr(ptr) };
            if let Ok(enc) = cstring.to_str() {
                let Ok(enc) = Encoder::try_from(enc) else { continue };

                if enc.matches_adapter(&adapter) {
                    encoders.push(enc);
                }
            }
        }
        encoders.sort();
        encoders
    }

    fn get_adapters_internal() -> Vec<Adapter> {
        let mut adapters: Vec<Adapter> = Vec::new();

        unsafe extern "C" fn callback(
            vec: *mut ::std::os::raw::c_void,
            name: *const ::std::os::raw::c_char,
            id: u32,
        ) -> bool {
            let adapters = &mut *(vec as *mut Vec<Adapter>);
            adapters.push(Adapter::new(id, CStr::from_ptr(name).to_string_lossy().to_string()));

            true
        }

        unsafe {
            libobs_sys::obs_enter_graphics();
            libobs_sys::gs_enum_adapters(
                Some(callback),
                &mut adapters as *mut Vec<Adapter> as *mut ::std::os::raw::c_void,
            );
            libobs_sys::obs_leave_graphics();
        }

        adapters
    }

    fn check_thread_initialized() -> Result<(), &'static str> {
        match LIBOBS_THREAD.get() {
            Some(thread_id) if thread_id == &thread::current().id() => Ok(()),
            Some(_) => Err("wrong thread - libobs was initialized in another thread"),
            None => Err("libos has not been initialized yet"),
        }
    }

    fn set_current_encoder(encoder: Encoder) {
        CURRENT_ENCODER.set(encoder);
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
}

impl InpRecorder {
    pub fn start_recording(&mut self) -> Result<(), String> {
        println!("Recording Start: {}", unsafe { libobs_sys::bnum_allocs() });
        if self.is_recording() {
            Ok(()) // already recording
        } else {
            if unsafe { libobs_sys::obs_output_start(self.output.as_ptr()) } {
                return Ok(());
            }

            let error = unsafe {
                let err = libobs_sys::obs_output_get_last_error(self.output.as_ptr());
                if err.is_null() {
                    c"no error message"
                } else {
                    CStr::from_ptr(err)
                }
            };
            Err(error.to_str().unwrap_or("error message is invalid UTF-8").to_string())
        }
    }

    pub fn stop_recording(&mut self) {
        if self.is_recording() {
            unsafe { libobs_sys::obs_output_stop(self.output.as_ptr()) }
            println!("Recording Stop: {}", unsafe { libobs_sys::bnum_allocs() });
        }

        let now = std::time::Instant::now();
        loop {
            thread::sleep(Duration::from_millis(100));
            if !self.is_recording() {
                return;
            } else if now.elapsed().as_millis() > 3000 {
                unsafe { libobs_sys::obs_output_force_stop(self.output.as_ptr()) };
                return;
            }
        }
    }

    pub fn configure(&self, settings: &RecorderSettings) -> Result<(), &'static str> {
        if self.is_recording() {
            return Err("can't change settings while recording");
        }

        // set adapter, input_resolution, output_resolution, framerate
        let ovi = Self::get_video_info()?;

        let framerate = settings.framerate.unwrap_or(Framerate::new(30, 1));

        let video_reset_necessary = settings.input_resolution.width() != ovi.base_width
            || settings.input_resolution.height() != ovi.base_height
            || settings.output_resolution.width() != ovi.output_width
            || settings.output_resolution.height() != ovi.output_height
            || framerate.num() != ovi.fps_num
            || framerate.den() != ovi.fps_den;
        if video_reset_necessary {
            Self::reset_video(settings.input_resolution, settings.output_resolution, framerate)?;

            unsafe {
                // reconfigure video output pipeline after resetting the video backend
                libobs_sys::obs_encoder_set_video(self.video_encoder.get().as_ptr(), libobs_sys::obs_get_video());
                libobs_sys::obs_output_set_video_encoder(self.output.as_ptr(), self.video_encoder.get().as_ptr());
                libobs_sys::obs_set_output_source(VIDEO_CHANNEL, self.video_source.as_ptr());
            }
        }

        let available_encoders = Self::get_available_encoders_internal();
        if let Some(encoder) = settings.encoder {
            // check if the given encoder is available on the current adapter
            if !available_encoders.contains(&encoder) {
                return Err("encoder not available");
            }
        }

        // if no encoder was explicitly set, choose an available encoder
        let encoder = match settings.encoder {
            Some(encoder) => encoder,
            None => *available_encoders.first().ok_or("no encoders available")?,
        };

        let mut get = Get::new();

        // set output_path
        let mut data = ObsData::new();
        data.set_string("path", &settings.output_path);
        unsafe { libobs_sys::obs_output_update(self.output.as_ptr(), data.as_ptr()) };

        // set video encoder
        Self::set_current_encoder(encoder);

        let data = encoder.settings(settings.rate_control.unwrap_or_default());
        let new_video_encoder = NonNull::new(unsafe {
            libobs_sys::obs_video_encoder_create(
                get.c_str(encoder.id()),
                get.c_str("video_encoder"),
                data.as_ptr(),
                null_mut(),
            )
        })
        .ok_or("unable to create video encoder")?;

        unsafe {
            libobs_sys::obs_encoder_set_video(new_video_encoder.as_ptr(), libobs_sys::obs_get_video());
            libobs_sys::obs_output_set_video_encoder(self.output.as_ptr(), new_video_encoder.as_ptr());
        }

        // replace and release old encoder
        let old_encoder = self.video_encoder.replace(new_video_encoder);
        unsafe { libobs_sys::obs_encoder_release(old_encoder.as_ptr()) };

        // set video source (window)
        let mut data = ObsData::new();
        data.set_string("window", settings.window.get_libobs_window_id());
        unsafe { libobs_sys::obs_source_update(self.video_source.as_ptr(), data.as_ptr()) };

        // set audio sources
        let audio_setting = settings.audio_source.unwrap_or(AudioSource::APPLICATION);

        // audio source 1
        let audio_source1 = match audio_setting {
            AudioSource::APPLICATION => {
                let mut data = ObsData::new();
                data.set_string("window", settings.window.get_libobs_window_id());
                unsafe { libobs_sys::obs_source_update(self.audio_source1.as_ptr(), data.as_ptr()) };

                self.audio_source1.as_ptr()
            }
            _ => null_mut(),
        };
        unsafe { libobs_sys::obs_set_output_source(AUDIO_CHANNEL1, audio_source1) };

        // audio source 2
        let audio_source2 = match audio_setting {
            AudioSource::SYSTEM | AudioSource::ALL => self.audio_source2.as_ptr(),
            _ => null_mut(),
        };
        unsafe { libobs_sys::obs_set_output_source(AUDIO_CHANNEL2, audio_source2) };

        // audio source 3
        let audio_source3 = match audio_setting {
            AudioSource::ALL => self.audio_source3.as_ptr(),
            _ => null_mut(),
        };
        unsafe { libobs_sys::obs_set_output_source(AUDIO_CHANNEL3, audio_source3) };

        println!("configured");

        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        unsafe { libobs_sys::obs_output_active(self.output.as_ptr()) }
    }

    pub fn get_adapter_info(&self) -> Adapter {
        // public version of internal function that is only available after libobs is initialized
        // due to requiring &self
        Self::get_adapters_internal()
            .into_iter()
            .find(|e| e.id() == AdapterId::default())
            .expect("no adapters found?")
    }

    pub fn get_available_encoders(&self) -> Vec<Encoder> {
        // public version of internal function that is only available after libobs is initialized
        // due to requiring &self
        Self::get_available_encoders_internal()
    }

    // re-export function as only available through a reference to a Recorder
    pub fn selected_encoder(&self) -> Encoder {
        Self::get_current_encoder()
    }
}

impl Drop for InpRecorder {
    fn drop(&mut self) {
        unsafe {
            // output
            libobs_sys::obs_output_release(self.output.as_ptr());
            // video
            libobs_sys::obs_encoder_release(self.video_encoder.get().as_ptr());
            libobs_sys::obs_source_release(self.video_source.as_ptr());
            // audio
            libobs_sys::obs_encoder_release(self.audio_encoder.as_ptr());
            libobs_sys::obs_source_release(self.audio_source1.as_ptr());
            libobs_sys::obs_source_release(self.audio_source2.as_ptr());
            libobs_sys::obs_source_release(self.audio_source3.as_ptr());

            println!("drop bnum_allocs: {}", libobs_sys::bnum_allocs());
        }

        Self::decrement_refcount();
    }
}
