//! FFI surface for homebrew content authoring metadata.
//!
//! Mirrors the web's WASM bindings: JSON in, JSON out, so the Flutter
//! form builder and the Leptos one drive the *same* `lorewyld-domain`
//! schema + validation. All functions are `#[frb(sync)]` because the
//! authoring UI reads the schema and validates synchronously.

use flutter_rust_bridge::frb;

/// The authoring field schema for a content category, as a JSON string the
/// Flutter form builder renders from. `"null"` for an unknown/unauthorable
/// category.
#[frb(sync)]
pub fn field_schema(category: String) -> String {
    lorewyld_domain::field_schema(&category)
        .and_then(|s| serde_json::to_string(&s).ok())
        .unwrap_or_else(|| "null".to_string())
}

/// The skeleton record for a category, as a JSON string used to seed a new
/// authoring form. `"null"` for an unknown category.
#[frb(sync)]
pub fn default_record(category: String) -> String {
    lorewyld_domain::default_record(&category)
        .and_then(|v| serde_json::to_string(&v).ok())
        .unwrap_or_else(|| "null".to_string())
}

/// Validate authoring input (a JSON object of field values) against the
/// category's schema. Returns a JSON array of `{field, message}` errors —
/// `"[]"` means valid.
#[frb(sync)]
pub fn validate_record(category: String, input_json: String) -> String {
    let errors = lorewyld_domain::validate_record(&category, &input_json)
        .err()
        .unwrap_or_default();
    serde_json::to_string(&errors).unwrap_or_else(|_| "[]".to_string())
}
