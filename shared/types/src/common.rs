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

/// Spell-component identifiers from the SRD's verbal/somatic/material
/// set. The canonical wire form is the full word (`"Verbal"`,
/// `"Somatic"`, `"Material"`); the single-letter SRD shorthand
/// (`"V"`, `"S"`, `"M"`) is accepted on deserialization as an alias.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpellComponent {
    #[serde(alias = "V", alias = "v")]
    Verbal,
    #[serde(alias = "S", alias = "s")]
    Somatic,
    #[serde(alias = "M", alias = "m")]
    Material,
}

/// Creature size categories.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CreatureSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

/// SRD rarity grades for magic items and enchantments.
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

/// Damage roll specification: dice expression + flat modifier + type.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageRoll {
    /// Dice expression, e.g. `"1d8"`, `"2d6"`.
    pub dice: String,
    /// Flat damage modifier added to the roll. May be negative.
    #[serde(default)]
    pub plus_mod: i32,
    pub damage_type: DamageTypeName,
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
    pub is_lawful: bool,
    pub is_neutral: bool,
    pub is_chaotic: bool,
    pub is_good: bool,
    pub is_evil: bool,
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
