//! FFI surface for 5e character-sheet math.
//!
//! All functions are `#[frb(sync)]` because the mobile sheet UI reads
//! derived values synchronously inside `build()`. The heavy lifting lives
//! in `lorewyld-domain`; this module only marshals across the boundary.

use flutter_rust_bridge::frb;
use lorewyld_types::character::CharacterSheet;

/// A named derived value (an ability, save, or skill) and its bonus.
pub struct NamedBonus {
    pub name: String,
    pub bonus: i32,
}

/// Everything the sheet UI derives from the raw scores, computed in one
/// call per edit.
pub struct DerivedStats {
    pub proficiency_bonus: i32,
    pub ability_modifiers: Vec<NamedBonus>,
    pub saving_throw_bonuses: Vec<NamedBonus>,
    pub skill_bonuses: Vec<NamedBonus>,
    pub initiative: i32,
    pub passive_perception: i32,
}

impl From<lorewyld_domain::NamedBonus> for NamedBonus {
    fn from(b: lorewyld_domain::NamedBonus) -> Self {
        Self {
            name: b.name,
            bonus: b.bonus,
        }
    }
}

fn map_bonuses(v: Vec<lorewyld_domain::NamedBonus>) -> Vec<NamedBonus> {
    v.into_iter().map(Into::into).collect()
}

impl From<lorewyld_domain::DerivedStats> for DerivedStats {
    fn from(d: lorewyld_domain::DerivedStats) -> Self {
        Self {
            proficiency_bonus: d.proficiency_bonus,
            ability_modifiers: map_bonuses(d.ability_modifiers),
            saving_throw_bonuses: map_bonuses(d.saving_throw_bonuses),
            skill_bonuses: map_bonuses(d.skill_bonuses),
            initiative: d.initiative,
            passive_perception: d.passive_perception,
        }
    }
}

/// Derive all sheet stats from a JSON-serialized `CharacterSheet` (the
/// mobile app already produces this via `CharacterSheet.toJson()`). A
/// malformed payload yields a zeroed block rather than crashing the
/// offline UI.
#[frb(sync)]
pub fn derive_stats(sheet_json: String) -> DerivedStats {
    serde_json::from_str::<CharacterSheet>(&sheet_json)
        .map(|sheet| lorewyld_domain::derive_stats(&sheet).into())
        .unwrap_or(DerivedStats {
            proficiency_bonus: 0,
            ability_modifiers: Vec::new(),
            saving_throw_bonuses: Vec::new(),
            skill_bonuses: Vec::new(),
            initiative: 0,
            passive_perception: 0,
        })
}

/// Ability modifier for a raw score — used by the create wizard before a
/// full sheet object exists.
#[frb(sync)]
pub fn ability_modifier(score: i32) -> i32 {
    lorewyld_domain::ability_modifier(score)
}

/// Proficiency bonus for a level.
#[frb(sync)]
pub fn proficiency_bonus(level: i32) -> i32 {
    lorewyld_domain::proficiency_bonus(level)
}
