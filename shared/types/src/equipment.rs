use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{DamageTypeName, EntityId, Timestamp};

/// Top-level grouping for mundane equipment (Simple Weapons, Light
/// Armor, etc.). Forms a tree via `parent_uuid`.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentCategory {
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

/// Finer-grained equipment subcategory tied to one `EquipmentCategory`.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentSubcategory {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    pub category_uuid: EntityId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Discriminator for `EquipmentProficiency.kind`.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentProficiencyKind {
    Armor,
    Weapon,
    Shield,
    ArtisanTool,
    MusicalInstrument,
    Vehicle,
}

/// A grantable equipment proficiency (Light Armor, Simple Weapons,
/// Disguise Kit, etc.).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentProficiency {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    #[serde(rename = "type")]
    pub kind: EquipmentProficiencyKind,
    pub name: String,
    pub slug: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    pub category_uuid: EntityId,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A physical consumable component used in spellcasting, alchemy,
/// or crafting.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquipmentMaterial {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    pub description: String,
    pub is_consumable: bool,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Properties an SRD weapon can have.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponProperty {
    Ammunition,
    Finesse,
    Heavy,
    Light,
    Loading,
    Reach,
    Special,
    Thrown,
    TwoHanded,
    Versatile,
}

/// A weapon entry with damage dice, properties, cost, and category.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Weapon {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    pub category_uuid: EntityId,
    /// Display name of the category, kept beside the FK for catalog
    /// rendering that doesn't join.
    pub category_name: String,
    /// Gold-piece cost.
    pub cost: f64,
    /// Damage dice expression — `"1d8"`, `"2d6"`, or
    /// `"1d8/1d10"` for versatile weapons.
    pub damage_dice: String,
    pub damage_type: DamageTypeName,
    /// Weight in pounds.
    pub weight: f64,
    #[serde(default)]
    pub properties: Vec<WeaponProperty>,
    /// `"normal/long"` range string for thrown / ammunition weapons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_range: Option<String>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Armor with AC calculation, weight, cost, and stealth properties.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Armor {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    pub category_uuid: EntityId,
    pub category_name: String,
    /// Base AC value provided.
    pub base_ac: i32,
    /// Whether the DEX modifier is added to AC.
    pub plus_dex_mod: bool,
    pub plus_con_mod: bool,
    pub plus_wis_mod: bool,
    /// Flat AC bonus (e.g. +1 for +1 armor).
    #[serde(default)]
    pub plus_flat_mod: i32,
    /// Maximum DEX modifier that applies; 0 for "no cap".
    #[serde(default)]
    pub plus_max: i32,
    /// Human-readable rendering, e.g. `"13 + Dex modifier (max 2)"`.
    pub ac_string: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strength_requirement: Option<i32>,
    pub cost: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    pub stealth_disadvantage: bool,
    pub is_heavy: bool,
    pub is_medium: bool,
    pub is_light: bool,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
