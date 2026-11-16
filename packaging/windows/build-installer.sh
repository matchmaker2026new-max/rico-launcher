#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Cargo is required. Install the stable Rust toolchain first." >&2
  exit 1
fi
if ! cargo xwin --version >/dev/null 2>&1; then
  echo "cargo-xwin is required. Install it with: cargo install cargo-xwin --locked" >&2
  exit 1
fi
if ! command -v ninja >/dev/null 2>&1; then
  echo "Ninja is required for the Windows Opus build. Install it with your Linux package manager (package: ninja-build)." >&2
  exit 1
fi
if ! command -v makensis >/dev/null 2>&1; then
  echo "NSIS is required. Install it with your Linux package manager (package: nsis)." >&2
  exit 1
fi

VERSION="$(sed -n '/^\[workspace\.package\]/,/^\[/ s/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
if [[ -z "$VERSION" ]]; then
  echo "Could not read the workspace version from Cargo.toml" >&2
  exit 1
fi

TARGET="x86_64-pc-windows-msvc"
LOG="$ROOT/target/rico-xwin-build.log"
mkdir -p "$ROOT/dist/windows" "$ROOT/target"
echo "Building RICO for Windows x64, version $VERSION..."
if ! cargo xwin build --release --locked --target "$TARGET" -p arctic-app 2>&1 | tee "$LOG"; then
  echo "Windows cross-build failed; see $LOG" >&2
  exit 1
fi
if grep -q "could not embed exe icon" "$LOG"; then
  echo "Windows executable was built without its RICO icon; refusing to package it." >&2
  exit 1
fi

SOURCE_EXE="$ROOT/target/$TARGET/release/rico-launcher.exe"
if [[ ! -f "$SOURCE_EXE" ]]; then
  echo "Expected Windows executable was not created: $SOURCE_EXE" >&2
  exit 1
fi
OUTPUT="$ROOT/dist/windows/RICO-Setup-$VERSION.exe"
echo "Packaging installer: $OUTPUT"
makensis \
  "-DAPP_VERSION=$VERSION" \
  "-DSOURCE_EXE=$SOURCE_EXE" \
  "-DLOGO_ICO=$ROOT/assets/rico-logo.ico" \
  "-DOUTPUT_FILE=$OUTPUT" \
  "$ROOT/packaging/windows/RICO-Setup.nsi"

test -s "$OUTPUT"
echo "Installer ready: $OUTPUT"
