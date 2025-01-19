# libobs-recorder

**NEWEST VERSION AVAILABLE: 31.0.1**  
**OLD VERSIONS ARE AVAILABLE BY COMPILING WITH THE LIBOBS_RECORDER_VERSION ENVIRONMENT VARIABLE SET TO THE DESIRED VERSION**

This is a rust library for recording using the [libobs library]("https://github.com/obsproject/obs-studio").
The current version supports only a very limited amount of options and only uses the windows game_capture plugin.

The libobs-sys subcrate is up-to-date with the newest Obs release.

~~For building this library it is necessary to install the Visual Studio Build Tools 2019 and LLVM (and adding the LLVM /bin folder to your `PATH`) because bindgen requires both.~~
This is only necessary if you want to generate the bindings yourself. You don't have to though, since they are now included in repo!
Just delete all the content from the `bindings.rs` file, but don't delete it. This causes the bindings to be regenerated on the next build.

Using this library requires including a bunch of .dll dependencies as well as a data/plugins folder with your compiled program.
You can find these files included in the `Releases` of this repo or in the matching Obs `Release`.
Due to these DLLs you can't just run the example. Instead you have to copy the DLLs to the output folder of the .exe and run the .exe from there.

There is a working version of the record example in the releases section. (for windows-x64)
