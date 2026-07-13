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
    #[serde(default)]
    pub class_name: String,
    #[serde(default = "default_level")]
    pub level: i32,
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
