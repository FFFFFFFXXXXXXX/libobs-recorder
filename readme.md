# libobs-recorder

This is a rust library for recording using the [libobs library]("https://github.com/obsproject/obs-studio").
The current version supports only a very limited amount of options and only uses the windows game_capture plugin.

In order to generate the FFI bindings you need to replace the C header files in the libobs-sys/libobs_headers/ folder with the ones your Obs version uses. You can find them in the Obs github project linked above in the *libobs* folder.
It is also necessary to install the Visual Studio Build Tools 2019 and LLVM (and adding the LLVM /bin folder to your `PATH`).

Using this library requires including a bunch of .dll files as well as a data/plugins folder with your program.
You can find these files in the prebuilt Obs downloads or you can build a headless Obs version yourself.
You also need a obs.lib file to link against. Either you take the .dll from from a prebuilt Obs version and generate obs.lib from that or if you built a headless Obs version yourself you can find the obs.lib in your build files.

There is a working version of the record example in the releases section. (for windows-x64)

List of required files to copy from Obs (or from a build you made yourself):

- obs.lib
- DLL
  - avcodec-58.dll
  - avdevice-58.dll
  - avfilter-7.dll
  - avformat-58.dll
  - avutil-56.dll
  - libaom.dll
  - libcurl.dll
  - libmbedcrypto.dll
  - libobs-d3d11.dll (for windows) / libobs-opengl.dll (for linux or mac)
  - libobs-winrt.dll
  - libogg-0.dll
  - libopus-0.dll
  - librist.dll
  - libsrt.dll
  - libSvtAv1Enc.dll
  - libvorbis-0.dll
  - libvorbisenc-2.dll
  - libvorbisfile-3.dll
  - libvpx-1.dll
  - libx264-163.dll
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
    - coreaudio-encoder
    - enc-amf
    - obs-ffmpeg
    - obs-outputs
    - obs-qsv11
    - obs-x264
    - win-capture
    - win-wasapi
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
