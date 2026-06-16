//! Pure 5e rules logic shared across every Lorewyld client.
//!
//! This crate is the single source of truth for computations that were
//! previously re-implemented per platform (Dart on mobile, hand-written
//! JS on the web). It is pure Rust over [`lorewyld_types`] with no
//! async/runtime dependencies, so it compiles natively (server, Leptos
//! web), to WASM (web), and over FFI (Flutter mobile via
//! `lorewyld-mobile-ffi`).

pub mod authoring;
pub mod sheet;

pub use authoring::{
    AUTHORABLE_CATEGORIES, EnumOption, FieldDef, FieldError, FieldKind, FieldSchema,
    default_record, field_schema, validate_record, validate_value,
};
pub use sheet::{
    DerivedStats, NamedBonus, ability_modifier, derive_stats, initiative, passive_perception,
    proficiency_bonus, saving_throw_bonus, skill_bonus,
};
