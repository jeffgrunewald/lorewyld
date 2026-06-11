//! Shared schema types for Lorewyld.
//!
//! All types follow the D&D 5e SRD data model documented in
//! `dnd5e_data_model/README.md`. Every type carries a `#[typeshare]`
//! annotation so the same shape can be generated for the Flutter mobile
//! client and any other consumer (TypeScript, Swift, Kotlin, etc.).
//!
//! Run `typeshare . --lang=dart --output-folder=mobile/lib/types` from
//! the workspace root to (re)generate Dart bindings.

pub mod api_v1;
pub mod background;
pub mod bundle;
pub mod character_feature;
pub mod class;
pub mod common;
pub mod condition;
pub mod content_module;
pub mod document;
pub mod enchantment;
pub mod equipment;
pub mod feat;
pub mod language;
pub mod lore_note;
pub mod magic_item;
pub mod monster;
pub mod plane;
pub mod race;
pub mod setting;
pub mod spell;
pub mod tag;
pub mod user;
pub mod version;

pub use api_v1::*;
pub use background::*;
pub use bundle::*;
pub use character_feature::*;
pub use class::*;
pub use common::*;
pub use condition::*;
pub use content_module::*;
pub use document::*;
pub use enchantment::*;
pub use equipment::*;
pub use feat::*;
pub use language::*;
pub use lore_note::*;
pub use magic_item::*;
pub use monster::*;
pub use plane::*;
pub use race::*;
pub use setting::*;
pub use spell::*;
pub use tag::*;
pub use user::*;
pub use version::*;
