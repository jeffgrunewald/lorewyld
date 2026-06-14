//! Shared schema types for Lorewyld.
//!
//! All types follow the D&D 5e SRD data model documented in
//! `dnd5e_data_model/README.md`, aligned with the Open5e v2 API
//! structure.
//!
//! This crate is the single source of truth for the schema. It is pure
//! Rust (no async/runtime deps) so it can be consumed natively by the
//! server and Leptos web app, compiled to WASM for the web, and bridged
//! to the Flutter mobile client over FFI via `lorewyld-mobile-ffi`.

pub mod api_v1;
pub mod background;
pub mod bundle;
pub mod character;
pub mod class;
pub mod common;
pub mod condition;
pub mod content_module;
pub mod creature;
pub mod document;
pub mod feat;
pub mod item;
pub mod language;
pub mod lore_note;
pub mod setting;
pub mod species;
pub mod spell;
pub mod tag;
pub mod user;
pub mod version;

pub use api_v1::*;
pub use background::*;
pub use bundle::*;
pub use character::*;
pub use class::*;
pub use common::*;
pub use condition::*;
pub use content_module::*;
pub use creature::*;
pub use document::*;
pub use feat::*;
pub use item::*;
pub use language::*;
pub use lore_note::*;
pub use setting::*;
pub use species::*;
pub use spell::*;
pub use tag::*;
pub use user::*;
pub use version::*;
