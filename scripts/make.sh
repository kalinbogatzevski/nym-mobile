#!/usr/bin/env bash
cbindgen ../src/lib.rs -l c > rustylib.h
cargo lipo --release
