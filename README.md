Pocket ReadLex
==============

Pocket ReadLex is an Android app containing a portable version of the
[Read Lexicon](https://readlex.pythonanywhere.com) which is a spelling
dictionary for the Shavian alphabet. It contains all of the dictionary
data in the package so the app does not need to access the internet.

Building
--------

This git repo does not include the assets containing the dictionary
data. Instead it is built from the ReadLex data and the included
compiledb program. The ReadLex data is in a git submodule. In order to
get it, be sure to run the following git command:

```bash
git submodule update --init
```

The compiledb program will be built as part of the app build in order
to generate the dictionary data. It is written in Rust, so you need to
make sure you have a [Rust compiler](https://rustup.rs/) for the host
machine installed. There is also a native library written in Rust used
by the app so you will also need the 4 compiler targets that Android
uses. You can install them with:

```bash
rustup target install \
        aarch64-linux-android \
        armv7-linux-androideabi \
        i686-linux-android \
        x86_64-linux-android
```

You will also need the NDK installed.

Assuming you have the Android SDK installed correctly, you can build
the app either with Android Studio or the command line as follows.

Debug mode:

    cd $HOME/prevo
    ./gradlew assembleDebug

Release mode:

    cd $HOME/prevo
    ./gradlew assembleRelease

You should then have the final package in either
`app/build/outputs/apk/debug/` or `app/build/outputs/apk/release/`
depending on the build type.
