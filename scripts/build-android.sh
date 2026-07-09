#!/bin/bash
# Build jianyi's ffi feature as Android shared libraries (per-ABI .so).
# Requires the Android NDK + cargo-ndk:
#   cargo install cargo-ndk
#   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
# Output: target/android/jniLibs/{arm64-v8a,armeabi-v7a,x86_64}/libjianyi.so
set -euo pipefail
cd "$(dirname "$0")/.."

if ! command -v cargo-ndk >/dev/null; then
    echo "❌ cargo-ndk not found — install it and the Android NDK first:" >&2
    echo "   cargo install cargo-ndk && rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android" >&2
    exit 1
fi

OUT=target/android/jniLibs
rm -rf "$OUT" && mkdir -p "$OUT"

cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o "$OUT" \
    build --release --no-default-features --features ffi

echo "✅ $OUT"
