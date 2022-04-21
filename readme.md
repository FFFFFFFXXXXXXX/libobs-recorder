# libobs-recorder

This is a rust library for recording using the [libobs library]("https://github.com/obsproject/obs-studio").
The current version supports only a very limited amount of options and only uses the windows game_capture plugin.

In order to generate the FFI bindings you need to replace the C header files in the libobs-sys/libobs_headers/ folder with the ones your Obs version uses. You can find them in the Obs github project linked above in the *libobs* folder.

Using this library requires including a bunch of .dll files as well as a data/plugins folder with your program.
You can find these files in the prebuilt Obs downloads or you can build a headless Obs version yourself.
You also need a obs.lib file to link against. Either you take the .dll from from a prebuilt Obs version and generate obs.lib from that or if you built a headless Obs version yourself you can find the obs.lib in your build files.

There is a working version of the record example in the releases section. (for windows-x64)
