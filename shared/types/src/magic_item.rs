use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{EntityId, Json, Rarity, Timestamp};

/// A category grouping for magic items (Weapon, Armor, Potion, etc.).
/// Forms a tree via `parent_uuid`.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MagicItemCategory {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_uuid: Option<EntityId>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A magical item with powers, rarity, attunement, and category.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MagicItem {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Free-form item-type label (Armor, Ring, Wand, Cloak…). Kept as
    /// `String` rather than enum because content packs invent new types.
    #[serde(rename = "type")]
    pub kind: String,
    pub rarity: Rarity,
    /// Optional rarity-roll table entry, e.g. `{ min: 1, max: 21, result: 11 }`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rarity_roll: Option<Json>,
    pub requires_attunement: bool,
    /// Free-text attunement clause (e.g. "by a sorcerer, warlock, or wizard").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attunement_requirement: Option<String>,
    #[serde(default)]
    pub attunement_by_class: Vec<String>,
    #[serde(default)]
    pub attunement_by_race: Vec<String>,
    #[serde(default)]
    pub attunement_by_alignment: Vec<String>,
    pub desc: String,
    /// Structured power entries; shape varies per item family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<Json>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
