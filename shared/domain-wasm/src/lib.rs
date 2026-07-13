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

/// The authoring [`FieldSchema`](lorewyld_domain::FieldSchema) for a content
/// category, as a JSON string the form builder renders from. `"null"` for an
/// unknown/unauthorable category.
#[wasm_bindgen]
pub fn field_schema(category: &str) -> String {
    lorewyld_domain::field_schema(category)
        .and_then(|s| serde_json::to_string(&s).ok())
        .unwrap_or_else(|| "null".to_string())
}

/// The skeleton record for a category, as a JSON string used to seed a new
/// authoring form. `"null"` for an unknown category.
#[wasm_bindgen]
pub fn default_record(category: &str) -> String {
    lorewyld_domain::default_record(category)
        .and_then(|v| serde_json::to_string(&v).ok())
        .unwrap_or_else(|| "null".to_string())
}

/// Validate authoring input (a JSON object of field values) against the
/// category's schema. Returns a JSON array of `{field, message}` errors —
/// `"[]"` means valid. The web form runs this pre-submit; the server runs
/// the same `lorewyld_domain::validate_record` authoritatively.
#[wasm_bindgen]
pub fn validate_record(category: &str, input_json: &str) -> String {
    let errors = lorewyld_domain::validate_record(category, input_json)
        .err()
        .unwrap_or_default();
    serde_json::to_string(&errors).unwrap_or_else(|_| "[]".to_string())
}

/// The fixed new-player guidance questionnaire, as a JSON string the quiz
/// UI renders from.
#[wasm_bindgen]
pub fn guidance_questionnaire() -> String {
    serde_json::to_string(&lorewyld_domain::guidance_questionnaire())
        .unwrap_or_else(|_| "null".to_string())
}

/// Rank candidate records against quiz answers. `answers_json` is a JSON
/// array of `{question, value}`; `candidates_json` is a JSON object of
/// `{classes, species, backgrounds}` record arrays (summaries suffice).
/// Returns a JSON `RecommendationSet`, or `"null"` on malformed input.
#[wasm_bindgen]
pub fn guidance_recommend(answers_json: &str, candidates_json: &str) -> String {
    let answers: Vec<lorewyld_domain::Answer> = match serde_json::from_str(answers_json) {
        Ok(a) => a,
        Err(_) => return "null".to_string(),
    };
    let candidates: lorewyld_domain::CandidateSets = match serde_json::from_str(candidates_json) {
        Ok(c) => c,
        Err(_) => return "null".to_string(),
    };
    serde_json::to_string(&lorewyld_domain::recommend(&answers, &candidates))
        .unwrap_or_else(|_| "null".to_string())
}
