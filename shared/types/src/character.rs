//! Server-side 5e character sheet.
//!
//! Mirrors the mobile app's local-first sheet model: the sheet
//! documents and computes (modifiers, proficiency from level) but never
//! enforces — any value the player supplies is accepted. Species,
//! class, and background are stored as plain display strings rather
//! than content references, so a sheet survives module changes intact.

use serde::{Deserialize, Serialize};

use crate::common::{AbilityScores, EntityId, Timestamp};

/// One carried item line on the sheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CharacterEquipmentItem {
    pub name: String,
    #[serde(default = "default_quantity")]
    pub quantity: i32,
    #[serde(default)]
    pub notes: String,
    /// Attuned to this character; only meaningful when the item's
    /// content record has `requires_attunement`. Clients cap at 3.
    #[serde(default)]
    pub attuned: bool,
}

/// One class a character has levels in. Multiclass characters carry
/// several entries; the character's level is the sum of entry levels.
/// Class and subclass are display-name strings, like the rest of the
/// sheet's content references. Every sheet has at least one entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CharacterClassEntry {
    pub name: String,
    #[serde(default = "default_level")]
    pub level: i32,
    /// Chosen subclass display name; "" = none yet.
    #[serde(default)]
    pub subclass: String,
    /// The initial class, which dictates first-level stats such as the
    /// hit-die seed. Exactly one entry should be flagged.
    #[serde(default)]
    pub starting: bool,
    /// Multiclass prerequisite snapshotted from the class record at pick
    /// time (OR-of-AND ability groups, each needing 13+), so the rule
    /// travels with the character to servers missing the content module.
    #[serde(default)]
    pub primary_abilities: Vec<Vec<String>>,
}

/// One known/prepared spell line on the sheet. `level` 0 = cantrip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CharacterSpellEntry {
    pub name: String,
    #[serde(default)]
    pub level: i32,
    #[serde(default)]
    pub notes: String,
}

/// A complete character sheet. Proficiencies are permissive name-string
/// lists (lowercase ability/skill names) rather than closed enums —
/// content is data we read, not data we control.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CharacterSheet {
    /// Server-assigned; clients may omit on create (defaults to nil).
    #[serde(default = "nil_uuid")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub uuid: EntityId,
    pub name: String,
    /// Populated by the server on read; ignored on write (ownership
    /// comes from the authenticated session / the existing row).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    pub owner_user_uuid: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_username: Option<String>,
    #[serde(default)]
    pub race: String,
    /// Classes this character has levels in. The server requires at
    /// least one entry; the character's level is the sum of entry levels.
    #[serde(default)]
    pub classes: Vec<CharacterClassEntry>,
    /// Accumulated XP for campaigns that advance by experience points.
    /// `None` = campaign doesn't track XP (e.g. milestone advancement).
    /// Advisory only — never auto-advances `level`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experience_points: Option<i32>,
    #[serde(default)]
    pub background: String,
    #[serde(default)]
    pub alignment: String,
    #[serde(default)]
    pub abilities: AbilityScores,
    #[serde(default)]
    pub saving_throw_proficiencies: Vec<String>,
    #[serde(default)]
    pub skill_proficiencies: Vec<String>,
    /// Known languages by display name — permissive strings, like the
    /// proficiency lists, so a sheet survives content-module changes.
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub armor_class: i32,
    #[serde(default)]
    pub speed: i32,
    #[serde(default)]
    pub max_hp: i32,
    #[serde(default)]
    pub current_hp: i32,
    #[serde(default)]
    pub hit_dice: String,
    #[serde(default)]
    pub equipment: Vec<CharacterEquipmentItem>,
    #[serde(default)]
    pub spells: Vec<CharacterSpellEntry>,
    /// Server-stamped; clients may omit on create/replace.
    #[serde(default = "now")]
    #[cfg_attr(feature = "openapi", schema(value_type = String, format = DateTime))]
    pub created_at: Timestamp,
    #[serde(default = "now")]
    #[cfg_attr(feature = "openapi", schema(value_type = String, format = DateTime))]
    pub updated_at: Timestamp,
}

fn now() -> Timestamp {
    chrono::Utc::now()
}

fn nil_uuid() -> EntityId {
    uuid::Uuid::nil()
}

fn default_quantity() -> i32 {
    1
}

fn default_level() -> i32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn experience_points_is_optional_and_round_trips() {
        // Absent in JSON deserializes to None.
        let sheet: CharacterSheet =
            serde_json::from_value(serde_json::json!({ "name": "Thistle" })).unwrap();
        assert_eq!(sheet.experience_points, None);

        // None is omitted on serialize (skip_serializing_if).
        let json = serde_json::to_value(&sheet).unwrap();
        assert!(json.get("experience_points").is_none());

        // A tracked value round-trips.
        let tracked = CharacterSheet {
            experience_points: Some(1500),
            ..sheet
        };
        let back: CharacterSheet =
            serde_json::from_value(serde_json::to_value(&tracked).unwrap()).unwrap();
        assert_eq!(back.experience_points, Some(1500));
    }

    #[test]
    fn classes_default_to_empty_and_round_trip() {
        // Absent in JSON deserializes to empty (the API layer, not
        // serde, enforces the at-least-one-class rule).
        let sheet: CharacterSheet =
            serde_json::from_value(serde_json::json!({ "name": "Thistle" })).unwrap();
        assert!(sheet.classes.is_empty());

        // Entry defaults: level 1, no subclass, not starting, no prereq.
        let entry: CharacterClassEntry =
            serde_json::from_value(serde_json::json!({ "name": "Fighter" })).unwrap();
        assert_eq!(entry.level, 1);
        assert_eq!(entry.subclass, "");
        assert!(!entry.starting);
        assert!(entry.primary_abilities.is_empty());

        // A populated multiclass list round-trips, snapshot included.
        let multi = CharacterSheet {
            classes: vec![
                CharacterClassEntry {
                    name: "Fighter".to_string(),
                    level: 5,
                    subclass: "Champion".to_string(),
                    starting: true,
                    primary_abilities: vec![
                        vec!["strength".to_string()],
                        vec!["dexterity".to_string()],
                    ],
                },
                CharacterClassEntry {
                    name: "Wizard".to_string(),
                    level: 3,
                    subclass: String::new(),
                    starting: false,
                    primary_abilities: vec![vec!["intelligence".to_string()]],
                },
            ],
            ..sheet
        };
        let back: CharacterSheet =
            serde_json::from_value(serde_json::to_value(&multi).unwrap()).unwrap();
        assert_eq!(back.classes, multi.classes);
    }

    #[test]
    fn languages_default_to_empty_and_round_trip() {
        // Absent in JSON deserializes to an empty list.
        let sheet: CharacterSheet =
            serde_json::from_value(serde_json::json!({ "name": "Thistle" })).unwrap();
        assert!(sheet.languages.is_empty());

        // A populated list round-trips.
        let known = CharacterSheet {
            languages: vec!["Common".to_string(), "Draconic".to_string()],
            ..sheet
        };
        let back: CharacterSheet =
            serde_json::from_value(serde_json::to_value(&known).unwrap()).unwrap();
        assert_eq!(back.languages, vec!["Common", "Draconic"]);
    }
}
