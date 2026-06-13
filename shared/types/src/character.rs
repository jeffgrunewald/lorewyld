//! Server-side 5e character sheet.
//!
//! Mirrors the mobile app's local-first sheet model: the sheet
//! documents and computes (modifiers, proficiency from level) but never
//! enforces — any value the player supplies is accepted. Species,
//! class, and background are stored as plain display strings rather
//! than content references, so a sheet survives module changes intact.

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{AbilityScores, EntityId, Timestamp};

/// One carried item line on the sheet.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterEquipmentItem {
    pub name: String,
    #[serde(default = "default_quantity")]
    pub quantity: i32,
    #[serde(default)]
    pub notes: String,
}

/// One known/prepared spell line on the sheet. `level` 0 = cantrip.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterSheet {
    /// Server-assigned; clients may omit on create (defaults to nil).
    #[serde(default = "nil_uuid")]
    pub uuid: EntityId,
    pub name: String,
    /// Populated by the server on read; ignored on write (ownership
    /// comes from the authenticated session / the existing row).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_user_uuid: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_username: Option<String>,
    #[serde(default)]
    pub race: String,
    #[serde(default)]
    pub class_name: String,
    #[serde(default = "default_level")]
    pub level: i32,
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
    pub created_at: Timestamp,
    #[serde(default = "now")]
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
