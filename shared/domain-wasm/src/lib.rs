//! WASM bindings exposing `lorewyld-domain` 5e math to the web client.
//!
//! The Leptos app is SSR-only with hand-written client JS (no client-side
//! Leptos/WASM runtime), so the shared rules logic reaches the browser as
//! this standalone WASM module. The surface mirrors the mobile FFI
//! (`lorewyld-mobile-ffi`): JSON in, derived stats out — so both clients
//! drive the *same* Rust implementation.

use lorewyld_types::character::CharacterSheet;
use wasm_bindgen::prelude::*;

/// Ability modifier for a raw score.
#[wasm_bindgen]
pub fn ability_modifier(score: i32) -> i32 {
    lorewyld_domain::ability_modifier(score)
}

/// Proficiency bonus for a level (clamped 1..=20).
#[wasm_bindgen]
pub fn proficiency_bonus(level: i32) -> i32 {
    lorewyld_domain::proficiency_bonus(level)
}

/// Derive every sheet stat from a JSON-serialized `CharacterSheet`, returned
/// as a JSON string for the caller to parse. Returns `"null"` on malformed
/// input rather than throwing, matching the mobile FFI's defensive contract.
#[wasm_bindgen]
pub fn derive_stats(sheet_json: &str) -> String {
    match serde_json::from_str::<CharacterSheet>(sheet_json) {
        Ok(sheet) => serde_json::to_string(&lorewyld_domain::derive_stats(&sheet))
            .unwrap_or_else(|_| "null".to_string()),
        Err(_) => "null".to_string(),
    }
}
