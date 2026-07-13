//! FFI surface for the new-player guidance quiz.
//!
//! Mirrors the web's WASM bindings: JSON in, JSON out, so the Flutter
//! quiz and the web one drive the *same* `lorewyld-domain` questionnaire
//! and recommendation engine. Sync because both calls are pure CPU.

use flutter_rust_bridge::frb;

/// The fixed new-player guidance questionnaire, as a JSON string the quiz
/// UI renders from.
#[frb(sync)]
pub fn guidance_questionnaire() -> String {
    serde_json::to_string(&lorewyld_domain::guidance_questionnaire())
        .unwrap_or_else(|_| "null".to_string())
}

/// Rank candidate records against quiz answers. `answers_json` is a JSON
/// array of `{question, value}`; `candidates_json` is a JSON object of
/// `{classes, species, backgrounds}` record arrays (full records or
/// summaries). Returns a JSON `RecommendationSet`, or `"null"` on
/// malformed input.
#[frb(sync)]
pub fn guidance_recommend(answers_json: String, candidates_json: String) -> String {
    let answers: Vec<lorewyld_domain::Answer> = match serde_json::from_str(&answers_json) {
        Ok(a) => a,
        Err(_) => return "null".to_string(),
    };
    let candidates: lorewyld_domain::CandidateSets = match serde_json::from_str(&candidates_json) {
        Ok(c) => c,
        Err(_) => return "null".to_string(),
    };
    serde_json::to_string(&lorewyld_domain::recommend(&answers, &candidates))
        .unwrap_or_else(|_| "null".to_string())
}
