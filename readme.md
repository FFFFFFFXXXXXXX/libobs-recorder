# libobs-recorder

This is a rust library for recording using the [libobs library]("https://github.com/obsproject/obs-studio").
Using this library requires including a bunch of .dll files as well as a data/plugins folder with your program.
You can find these files in the prebuilt Obs downloads or you can build a headless Obs version yourself.
You also need a obs.lib file to link against. Either you take the .dll from from a prebuilt Obs version and generate obs.lib from that or if you built a headless Obs version yourself you can find the obs.lib in your build files.

There is an example .zip file in the releases.
