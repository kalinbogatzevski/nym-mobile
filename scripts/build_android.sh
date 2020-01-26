#!/usr/bin/env bash
min_ver=22

cargo ndk --target aarch64-linux-android --android-platform ${min_ver} -- build --release
cargo ndk --target armv7-linux-androideabi --android-platform ${min_ver} -- build --release
cargo ndk --target i686-linux-android --android-platform ${min_ver} -- build --release
cargo ndk --target x86_64-linux-android --android-platform ${min_ver} -- build --release

