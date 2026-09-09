#!/usr/bin/env bash
# Cross-compile bgscan-installer for release. CI only.
#
# Usage: ./scripts/build.sh {linux|macos|windows|android|all} [version]
#
# Linux/macOS/Windows targets go through cargo-zigbuild (zig does the C
# compilation). Android needs the NDK since zig dropped bionic headers.
#
# Binaries land in dist/.

set -euo pipefail

TARGET="${1:-linux}"
VERSION="${2:-dev}"

ROOT_DIR="$PWD"
DIST_DIR="$ROOT_DIR/dist"
BIN_NAME="bgscan-installer"

mkdir -p "$DIST_DIR"

log() {
  echo
  echo "======================================"
  echo "$*"
  echo "======================================"
}

# build <rust-triple> <output-name>
build() {
  local rust_target="$1"
  local name="$2"

  log "BUILD => $rust_target -> $name"

  rustup target add "$rust_target"

  APP_VERSION="$VERSION" cargo zigbuild --release --target "$rust_target"

  local out="target/$rust_target/release/$BIN_NAME"
  [ -f "$out.exe" ] && out="$out.exe"

  cp "$out" "$DIST_DIR/$name"
  chmod +x "$DIST_DIR/$name"
}

setup_android_ndk() {
  NDK_VERSION="r27d"
  NDK_DIR="$ROOT_DIR/android-ndk-$NDK_VERSION"

  log "ANDROID: setting up NDK"

  if [ ! -d "$NDK_DIR" ]; then
    wget -q "https://dl.google.com/android/repository/android-ndk-${NDK_VERSION}-linux.zip" \
      -O "$ROOT_DIR/ndk.zip"
    unzip -q "$ROOT_DIR/ndk.zip" -d "$ROOT_DIR"
    rm -f "$ROOT_DIR/ndk.zip"
  fi

  local toolchain="$NDK_DIR/toolchains/llvm/prebuilt/linux-x86_64/bin"

  # ring and other C-backed crates need the NDK clang per target
  export CC_aarch64_linux_android="$toolchain/aarch64-linux-android21-clang"
  export CC_armv7_linux_androideabi="$toolchain/armv7a-linux-androideabi21-clang"
  export CC_i686_linux_android="$toolchain/i686-linux-android21-clang"
  export CC_x86_64_linux_android="$toolchain/x86_64-linux-android21-clang"

  export AR_aarch64_linux_android="$toolchain/llvm-ar"
  export AR_armv7_linux_androideabi="$toolchain/llvm-ar"
  export AR_i686_linux_android="$toolchain/llvm-ar"
  export AR_x86_64_linux_android="$toolchain/llvm-ar"

  export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$CC_aarch64_linux_android"
  export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$CC_armv7_linux_androideabi"
  export CARGO_TARGET_I686_LINUX_ANDROID_LINKER="$CC_i686_linux_android"
  export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$CC_x86_64_linux_android"
}

case "$TARGET" in

linux)
  log "TARGET: LINUX"

  build x86_64-unknown-linux-musl      linux-64
  build i686-unknown-linux-musl        linux-32
  build aarch64-unknown-linux-musl     linux-arm64
  build armv7-unknown-linux-musleabihf linux-arm32-v7a
  ;;

macos)
  log "TARGET: MACOS"

  build x86_64-apple-darwin  macos-64
  build aarch64-apple-darwin macos-arm64
  ;;

windows)
  log "TARGET: WINDOWS"

  build x86_64-pc-windows-gnu      windows-64.exe
  build aarch64-pc-windows-gnullvm windows-arm64.exe
  ;;

android)
  log "TARGET: ANDROID"

  setup_android_ndk

  build aarch64-linux-android   android-arm64-v8a
  build armv7-linux-androideabi android-armeabi-v7a
  build i686-linux-android      android-x86
  build x86_64-linux-android    android-x86_64
  ;;

all)
  log "TARGET: ALL"

  bash "$0" linux "$VERSION"
  bash "$0" macos "$VERSION"
  bash "$0" windows "$VERSION"
  bash "$0" android "$VERSION"
  ;;

*)
  echo "Usage: $0 {linux|macos|windows|android|all} [version]"
  exit 1
  ;;
esac

log "BUILD COMPLETE"
echo "Artifacts:"
ls -lh "$DIST_DIR"
