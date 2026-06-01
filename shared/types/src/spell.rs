use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{EntityId, Json, SpellComponent, SpellSchoolName, Timestamp};

/// A magical spell from the SRD.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spell {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// 0 = cantrip; 1–9 otherwise.
    pub level: u8,
    /// FK -> `spell_schools.uuid`.
    pub school: EntityId,
    #[serde(default)]
    pub ritual: bool,
    pub duration: String,
    pub casting_time: String,
    pub range: String,
    #[serde(default)]
    pub components: Vec<SpellComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_components: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub higher_level: Option<String>,
    /// Structured scaling data, shape varies per spell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scaling: Option<Json>,
    pub concentration: bool,
    pub requires_verbal: bool,
    pub requires_somatic: bool,
    pub requires_material: bool,
    /// Classes that can cast the spell (slug refs).
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub subclasses: Vec<String>,
    /// Spell-list slugs the spell appears on.
    #[serde(default)]
    pub spell_lists: Vec<String>,
    /// Per-class first-available level overrides, e.g.
    /// `{"bard": 2, "wizard": 3}`. Free-form JSON because expansions
    /// add new classes routinely.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub levels_by_class: Option<Json>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// One of the eight schools of magic. Lookup-table row backing the
/// closed-set `SpellSchoolName` enum.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellSchool {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: SpellSchoolName,
    pub slug: String,
    pub description: String,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A named collection of spells tied to one or more classes (Bard
/// spell list, Cleric spell list, etc.).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellList {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Primary class slug this list belongs to.
    pub class: String,
    /// All class slugs that use this list (subclasses included).
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
