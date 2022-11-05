# libobs-recorder

**CURRENTLY TIED TO OBS VERSION: 28.1.1**

This is a rust library for recording using the [libobs library]("https://github.com/obsproject/obs-studio").
The current version supports only a very limited amount of options and only uses the windows game_capture plugin.

The libobs-sys subcrate is up to date with the newest Obs release.

For building this library it is necessary to install the Visual Studio Build Tools 2019 and LLVM (and adding the LLVM /bin folder to your `PATH`) because bindgen requires both.

Using this library requires including a bunch of .dll dependencies as well as a data/plugins folder with your compiled program.
You can find these files in the newest Obs release. Alternatively you could build a headless Obs version yourself.

There is a working version of the record example in the releases section. (for windows-x64)

List of required files to copy from Obs release (or from a build you made yourself):

- avcodec-59.dll
- avdevice-59.dll
- avfilter-8.dll
- avformat-59.dll
- avutil-57.dll
- libcurl.dll
- libmbedcrypto.dll
- libobs-d3d11.dll (for windows) / libobs-opengl.dll (for linux or mac)
- libobs-winrt.dll
- librist.dll
- libsrt.dll
- libx264-164.dll
- obs-amf-test.exe
- obs-ffmpeg-mux.exe
- obs.dll
- swresample-3.dll
- swscale-5.dll
- w32-pthreads.dll
- zlib.dll
- data
  - libobs
    - area.effect
    - bicubic_scale.effect
    - bilinear_lowres_scale.effect
    - color.effect
    - default_rect.effect
    - default.effect
    - deinterlace_base.effect
    - deinterlace_blend_2x.effect
    - deinterlace_blend.effect
    - deinterlace_discard_2x.effect
    - deinterlace_discard.effect
    - deinterlace_linear_2x.effect
    - deinterlace_linear.effect
    - deinterlace_yadif_2x.effect
    - deinterlace_yadif.effect
    - format_conversion.effect
    - lanczos_scale.effect
    - opaque.effect
    - premultiplied_alpha.effect
    - repeat.effect
    - solid.effect
  - obs-plugins
    - enc-amf
      - enc-amf-test32.exe
      - enc-amf-test64.exe
    - win-capture
      - get-graphics-offsets32.exe
      - get-graphics-offsets64.exe
      - graphics-hook32.dll
      - graphics-hook64.dll
      - inject-helper32.exe
      - inject-helper64.exe
      - obs-vulkan32.json
      - obs-vulkan64.json
- obs-plugins
  - 64bit
    - coreaudio-encoder.dll
    - enc-amf.dll
    - obs-ffmpeg.dll
    - obs-outputs.dll
    - obs-qsv11.dll
    - obs-x264.dll
    - win-capture.dll
    - win-wasapi.dll
    - libEGL.dll
    - libGLESv2.dll
