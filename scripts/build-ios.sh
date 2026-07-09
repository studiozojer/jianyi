#!/bin/bash
# Build jianyi's ffi feature as an iOS xcframework (device + simulator).
# Output: target/ios/jianyi.xcframework
set -euo pipefail
cd "$(dirname "$0")/.."

export IPHONEOS_DEPLOYMENT_TARGET=15.0

rustup target add aarch64-apple-ios aarch64-apple-ios-sim >/dev/null

echo "building device (aarch64-apple-ios)…"
CARGO_PROFILE_RELEASE_STRIP=none cargo build --release --target aarch64-apple-ios --no-default-features --features ffi

echo "building simulator (aarch64-apple-ios-sim)…"
CARGO_PROFILE_RELEASE_STRIP=none cargo build --release --target aarch64-apple-ios-sim --no-default-features --features ffi

OUT=target/ios
rm -rf "$OUT/jianyi.xcframework"
mkdir -p "$OUT"

xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/release/libjianyi.a -headers include \
    -library target/aarch64-apple-ios-sim/release/libjianyi.a -headers include \
    -output "$OUT/jianyi.xcframework"

echo "✅ $OUT/jianyi.xcframework"
