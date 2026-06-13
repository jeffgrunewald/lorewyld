//! Shared schema types for Lorewyld.
//!
//! All types follow the D&D 5e SRD data model documented in
//! `dnd5e_data_model/README.md`, aligned with the Open5e v2 API
//! structure. Every type carries a `#[typeshare]` annotation so the same
//! shape can be generated for the Flutter mobile client and any other
//! consumer (TypeScript, Swift, Kotlin, etc.).
//!
//! Run `typeshare . --lang=dart --output-folder=mobile/lib/types` from
//! the workspace root to (re)generate Dart bindings.

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
