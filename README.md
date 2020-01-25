# Nym mobile client

Work in Progress

Nym client port to mobile devices

## Dependencies

### Clone the following nym repository
    
    https://github.com/nymtech/nym

### Install rust targets

    # Android targets
    rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

    # iOS targets
    rustup target add aarch64-apple-ios armv7-apple-ios armv7s-apple-ios x86_64-apple-ios i386-apple-ios

### Rust dependencies 

    # this cargo subcommand will help you create a universal library for use with iOS.
    cargo install cargo-lipo
    # this tool will let you automatically create the C/C++11 headers of the library.
    cargo install cbindgen
    # to install android ndk support
    cargo install cargo-ndk

## Build    

### Export C headers from Rust code 

    $ cbindgen src/lib.rs -l c > rustylib.h

### iOS

    $ cargo lipo --release

### Android 

    $ cargo ndk --target aarch64-linux-android --android-platform 22 -- build --release
    $ cargo ndk --target armv7-linux-androideabi --android-platform 22 -- build --release
    $ cargo ndk --target i686-linux-android --android-platform 22 -- build --release
    $ cargo ndk --target x86_64-linux-android --android-platform 22  -- build --release

### TODO

[] How to reconnect ?
