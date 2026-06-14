//! flutter_rust_bridge entry crate: the FFI boundary between the shared
//! Rust core (`lorewyld-types` / `lorewyld-domain`) and the Flutter app.
//!
//! Only marshaling lives here — all rules logic stays in
//! `lorewyld-domain` so it is shared verbatim with the server and web.
//! `mod frb_generated;` is added by `flutter_rust_bridge_codegen generate`.

pub mod api;
mod frb_generated;
