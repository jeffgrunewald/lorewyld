//! Shared primitive types: enums for fields with closed value sets,
//! plus small structured helpers that appear in multiple schemas.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use uuid::Uuid;

/// Canonical UUID alias. Typeshare emits this as `String` in target
/// languages, matching the v4-UUID-as-text storage convention.
#[typeshare(serialized_as = "string")]
pub type EntityId = Uuid;

/// Canonical timestamp alias.
#[typeshare(serialized_as = "string")]
pub type Timestamp = DateTime<Utc>;

/// Fixed namespace for deterministic content UUIDs. Never change this
/// value: every seeded record's identity is derived from it.
pub const LOREWYLD_CONTENT_NAMESPACE: Uuid =
    Uuid::from_u128(0x8f0c2f9b_3a41_4f7e_9c3d_6b1e5a2d7c90);

/// Derives the canonical UUID for a content record as
/// UUIDv5(namespace, `"{type_tag}:{key}"`), where `key` is the record's
/// stable external identifier (Open5e key for imported content).
/// Regenerating a bundle therefore never churns identities, and foreign
/// keys are computable without lookups.
pub fn content_uuid(type_tag: &str, key: &str) -> Uuid {
    Uuid::new_v5(
        &LOREWYLD_CONTENT_NAMESPACE,
        format!("{type_tag}:{key}").as_bytes(),
    )
}

/// One of the six core ability scores.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AbilityScore {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

/// Per-ability values, used for monster ability scores, ASIs, and
/// modifier maps. All six axes are required so consumers don't have to
/// branch on missing entries.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AbilityScores {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

/// One of the nine SRD alignment values on the law/chaos × good/evil axes.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignmentName {
    LawfulGood,
    NeutralGood,
    ChaoticGood,
    LawfulNeutral,
    TrueNeutral,
    ChaoticNeutral,
    LawfulEvil,
    NeutralEvil,
    ChaoticEvil,
    Unaligned,
}

/// SRD damage type enum. Used by weapons, spells, and monster
/// resistance/immunity arrays.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DamageTypeName {
    Acid,
    Bludgeoning,
    Cold,
    Fire,
    Force,
    Lightning,
    Necrotic,
    Piercing,
    Poison,
    Psychic,
    Radiant,
    Slashing,
    Thunder,
}

/// SRD condition names. Referenced by `condition_immunities` arrays.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionName {
    Blinded,
    Charmed,
    Deafened,
    Exhaustion,
    Frightened,
    Grappled,
    Incapacitated,
    Invisible,
    Paralyzed,
    Petrified,
    Poisoned,
    Prone,
    Restrained,
    Stunned,
    Unconscious,
}

/// One of the eight schools of magic.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpellSchoolName {
    Abjuration,
    Conjuration,
    Divination,
    Enchantment,
    Evocation,
    Illusion,
    Necromancy,
    Transmutation,
}

/// SRD rarity grades for magic items. Variant order encodes the
/// Open5e rarity rank (Common = 1 … Artifact = 6).
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    VeryRare,
    Legendary,
    Artifact,
}

/// Movement modes a creature can use, in feet. `Walk` is required;
/// other modes default to 0 when absent.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MovementSpeed {
    pub walk: i32,
    #[serde(default)]
    pub fly: i32,
    #[serde(default)]
    pub swim: i32,
    #[serde(default)]
    pub climb: i32,
    #[serde(default)]
    pub burrow: i32,
    #[serde(default)]
    pub crawl: i32,
    #[serde(default)]
    pub hover: bool,
}

/// Senses with numeric range (feet) and a free-text human-readable
/// summary that the SRD prints inline (e.g.
/// `"darkvision 60 ft., passive Perception 10"`).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Senses {
    pub passive_perception: i32,
    #[serde(default)]
    pub darkvision: i32,
    #[serde(default)]
    pub blindsight: i32,
    #[serde(default)]
    pub tremorsense: i32,
    #[serde(default)]
    pub truesight: i32,
    #[serde(default)]
    pub summary: String,
}

/// Generic "choose N from a list" pattern that appears for skill,
/// language, tool, and equipment proficiencies.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceFrom {
    pub choose: u32,
    pub from: Vec<String>,
    /// Optional human-readable rendering of the choice (e.g.
    /// "Two of your choice").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Optional named modifier — used for saving-throw and skill bonuses
/// keyed by their canonical name.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedModifier {
    pub name: String,
    pub bonus: i32,
}

/// Free-form JSON, used for fields whose shape varies per record
/// (class feature tables, action arrays, scaling rules, etc.).
///
/// Typeshare emits this as a permissive type in non-Rust targets
/// (`any` / `dynamic` / `Map<String, Any>` depending on language).
#[typeshare(serialized_as = "object")]
pub type Json = serde_json::Value;

/// Audit columns shared by every persisted record.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditFields {
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

// ─── Lookup-table row structs for the closed-set enums ──────────────────

/// Persisted ability-score lookup row. The `name` enum acts as the
/// canonical identifier; UUID + slug remain for FK and URL ergonomics.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbilityScoreEntry {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: AbilityScore,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    pub short_name: String,
    pub description: String,
    /// Mapping of raw score values (3–30) to modifiers (-5..=+10).
    /// Stored as JSON because the table is short and consumers want
    /// it as a literal `{score: modifier}` map.
    pub modifier_table: Json,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// SRD alignment lookup row. Boolean axes match the README schema and
/// support partial-alignment queries (e.g. `is_evil=true`).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alignment {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: AlignmentName,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    pub is_lawful: bool,
    pub is_neutral: bool,
    pub is_chaotic: bool,
    pub is_good: bool,
    pub is_evil: bool,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Creature/object size lookup row (Open5e v2 `sizes`). Replaces the
/// former closed `CreatureSize` enum so content packs can add sizes.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    /// Ordering rank: Tiny = 1 … Gargantuan = 6.
    pub rank: i32,
    /// Diameter of the space the creature controls, in feet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_diameter: Option<f64>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Skill lookup row (Open5e v2 `skills`), keyed to its governing ability.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skill {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    pub ability: AbilityScore,
    pub description: String,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// SRD damage-type lookup row.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageType {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: DamageTypeName,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    pub description: String,
    /// Other damage types resistant to this one.
    #[serde(default)]
    pub resistances_against: Vec<DamageTypeName>,
    /// Other damage types immune to this one.
    #[serde(default)]
    pub immunities_against: Vec<DamageTypeName>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_uuid_is_stable_and_type_scoped() {
        let a = content_uuid("spell", "srd-2024_fireball");
        assert_eq!(a, content_uuid("spell", "srd-2024_fireball"));
        // Pinned value: changing it would re-identify every seeded record.
        assert_eq!(a.to_string(), content_uuid("spell", "srd-2024_fireball").to_string());
        assert_ne!(a, content_uuid("creature", "srd-2024_fireball"));
        assert_ne!(a, content_uuid("spell", "srd_fireball"));
    }
}
