// Global Flutter-test setup: initialize the shared Rust core (FFI) once
// before any test runs, so widget tests that build the character sheet
// (which derives stats via the Rust core) work on the host VM.
//
// The native library is built from the workspace with:
//   cargo build -p lorewyld_mobile_ffi
// which lands at <workspace>/target/debug/, i.e. ../target/debug from the
// Flutter package. (cargokit builds it for real iOS/Android app builds.)

import 'dart:async';
import 'dart:io';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated_io.dart';
import 'package:lorewyld/ffi/frb_generated.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  final libName = Platform.isMacOS
      ? 'liblorewyld_mobile_ffi.dylib'
      : Platform.isWindows
      ? 'lorewyld_mobile_ffi.dll'
      : 'liblorewyld_mobile_ffi.so';
  await RustLib.init(
    externalLibrary: ExternalLibrary.open('../target/debug/$libName'),
  );
  await testMain();
}
