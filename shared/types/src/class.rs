use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{AbilityScore, ChoiceFrom, EntityId, Json, Timestamp};

/// The character level(s) at which a class feature is gained.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureLevel {
    pub level: u8,
    /// Per-level variation text (e.g. `"2 uses"` at the level where a
    /// feature improves).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// A named class or subclass feature with its level progression.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassFeature {
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    pub name: String,
    pub desc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_type: Option<String>,
    /// Every level the feature is gained or improves at.
    #[serde(default)]
    pub gained_at: Vec<FeatureLevel>,
}

/// A playable class or subclass. Subclasses are sibling rows linked via
/// `subclass_of` (the Open5e v2 model) rather than a separate type.
///
/// The proficiency/equipment/spell-slot fields are *retained sheet-math
/// data* that Open5e v2 dropped to prose; the bundle generator populates
/// them from the v1 API and curated overrides. They are `None`/empty on
/// subclass rows, which inherit from their parent class.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Class {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    /// FK -> `documents.uuid` (source attribution).
    pub document_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    pub desc: String,
    /// FK -> `classes.uuid` of the parent class; `None` for base classes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subclass_of: Option<EntityId>,
    /// Die size: 6, 8, 10, or 12. `None` on subclass rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_dice: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp_at_1st_level: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp_at_higher_levels: Option<String>,
    /// `"FULL"`, `"HALF"`, `"NONE"`, … Informational only — sparsely
    /// populated upstream; never derive mechanics from it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caster_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prof_armor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prof_weapons: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prof_tools: Option<String>,
    #[serde(default)]
    pub prof_saving_throws: Vec<AbilityScore>,
    /// Skill-proficiency choice; e.g. `{ choose: 2, from: [...] }`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prof_skills: Option<ChoiceFrom>,
    /// Starting-equipment options with choices. Free-form because each
    /// class shapes its option lists differently.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipment: Option<Json>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spellcasting_ability: Option<AbilityScore>,
    /// Spell slots by class level: `{ "1": [2], "2": [3], ... }` where
    /// each array is slots per spell level. `None` for non-casters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spell_slot_table: Option<Json>,
    /// Display label for the subclass family (Archetype, Divine Domain,
    /// Pact Boon, etc.). `None` on subclass rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtypes_name: Option<String>,
    #[serde(default)]
    pub features: Vec<ClassFeature>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
