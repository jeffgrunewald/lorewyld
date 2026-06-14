#!/usr/bin/env bash
# Regenerate the flutter_rust_bridge bindings between the shared Rust core
# (`mobile-ffi`, which re-exports `lorewyld-domain`) and the Flutter app.
#
# Run after editing `mobile-ffi/src/api/`. Commits the generated Dart
# (`mobile/lib/ffi/`) and `mobile-ffi/src/frb_generated.rs`.
#
# Prereq: the installed flutter_rust_bridge_codegen version MUST match the
# `flutter_rust_bridge` package version pinned in `mobile/pubspec.yaml`.
#   cargo install flutter_rust_bridge_codegen --version 2.12.0
set -euo pipefail

cd "$(dirname "$0")/../mobile"
flutter_rust_bridge_codegen generate
echo "Regenerated FFI bindings (mobile/lib/ffi/ + mobile-ffi/src/frb_generated.rs)"
