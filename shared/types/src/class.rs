use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{AbilityScore, ChoiceFrom, EntityId, Json, Timestamp};

/// A playable character class with hit dice, proficiencies, feature
/// progression, archetype support, and optional spellcasting.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Class {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Die size: 6, 8, 10, or 12.
    pub hit_dice: u8,
    pub hp_at_1st_level: String,
    pub hp_at_higher_levels: String,
    pub prof_armor: String,
    pub prof_weapons: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prof_tools: Option<String>,
    pub prof_saving_throws: String,
    /// Skill-proficiency choice; e.g. `{ choose: 2, from: [...] }`.
    pub prof_skills: ChoiceFrom,
    /// Starting-equipment options with choices.
    pub equipment: Json,
    /// Full per-level progression table (proficiency bonus, features,
    /// spells known, slots-by-level, etc.). Free-form because each
    /// class shapes it differently.
    pub feature_table: Json,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spellcasting_ability: Option<AbilityScore>,
    /// For Pact Magic classes only — separate slot table from the
    /// main feature table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spellcasting_table: Option<Json>,
    /// Display label for the subclass family (Archetype, Divine Domain,
    /// Pact Boon, etc.).
    pub subtypes_name: String,
    /// Subclass feature progression keyed by level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archetype_table: Option<Json>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A subclass/archetype/domain/path/pact within a class.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subclass {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    pub parent_class_uuid: EntityId,
    /// Display name of the parent class, kept alongside the FK for
    /// catalog views that don't join.
    pub parent_class: String,
    /// Character level at which this subclass becomes available
    /// (typically 1, 2, or 3 in the SRD).
    pub level: u8,
    /// Subclass features keyed by character level.
    pub feature_table: Json,
    /// `{ level: { name, desc } }` shape — kept Json because the SRD
    /// emits features in irregular nested forms.
    pub features: Json,
    /// Spells gained at each level for casting subclasses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spell_list: Option<Json>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
