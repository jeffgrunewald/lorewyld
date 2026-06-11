//! Unified item model per Open5e v2: a single `Item` wraps every piece
//! of gear (mundane and magical); weapon and armor mechanics live in
//! dedicated records the item references.

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{DamageTypeName, EntityId, Rarity, Timestamp};

/// A grouping for items (Adventuring Gear, Weapon, Potion, …).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemCategory {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A weapon-property definition (Finesse, Heavy, …, plus 2024 Mastery
/// properties). A lookup row rather than a closed enum so content packs
/// can add properties.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponPropertyDef {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    /// Property family: `None` for classic properties, `"Mastery"` for
    /// 2024 weapon-mastery properties.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub description: String,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A weapon's link to one of its properties, with the per-weapon detail
/// some properties carry (e.g. Versatile's `"1d10"`).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponPropertyRef {
    /// FK -> `weapon_properties.uuid`.
    pub property_uuid: EntityId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Weapon mechanics. Cost, weight, and category live on the wrapping
/// `Item`.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Weapon {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    /// FK -> `documents.uuid` (source attribution).
    pub document_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    /// Damage dice expression, e.g. `"1d8"`.
    pub damage_dice: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_type: Option<DamageTypeName>,
    /// Normal range in feet for ranged/thrown weapons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub long_range: Option<f64>,
    /// Simple weapon when true; martial otherwise.
    pub is_simple: bool,
    #[serde(default)]
    pub is_improvised: bool,
    #[serde(default)]
    pub properties: Vec<WeaponPropertyRef>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Armor mechanics. Cost, weight, and category live on the wrapping
/// `Item`.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Armor {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    /// FK -> `documents.uuid` (source attribution).
    pub document_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    /// `"light"`, `"medium"`, `"heavy"`, or `"shield"`.
    pub category: String,
    pub ac_base: i32,
    pub ac_add_dexmod: bool,
    /// Maximum DEX modifier that applies; `None` for no cap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ac_cap_dexmod: Option<i32>,
    /// Human-readable rendering, e.g. `"14 + Dex modifier (max 2)"`.
    pub ac_display: String,
    pub grants_stealth_disadvantage: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strength_score_required: Option<i32>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Any piece of gear, mundane or magical. Weapon/armor mechanics hang
/// off `weapon_uuid`/`armor_uuid`; magic items share the base weapon or
/// armor record they enchant (e.g. Adamantine Breastplate references the
/// plain Breastplate's armor row).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    /// FK -> `documents.uuid` (source attribution).
    pub document_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    pub desc: String,
    /// FK -> `item_categories.uuid`.
    pub category_uuid: EntityId,
    /// Weight in pounds.
    #[serde(default)]
    pub weight: f64,
    /// Gold-piece cost as a decimal string (exact; avoids float-cents
    /// drift and stays sortable after CAST in SQL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<String>,
    /// FK -> `weapons.uuid` when the item is a weapon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_uuid: Option<EntityId>,
    /// FK -> `armors.uuid` when the item is armor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armor_uuid: Option<EntityId>,
    /// `None` for mundane items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rarity: Option<Rarity>,
    #[serde(default)]
    pub requires_attunement: bool,
    /// Attunement clause, e.g. `"by a sorcerer, warlock, or wizard"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attunement_detail: Option<String>,
    #[serde(default)]
    pub is_magic: bool,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
