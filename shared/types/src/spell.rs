use serde::{Deserialize, Serialize};

use crate::common::{AbilityScore, DamageTypeName, EntityId, SpellSchoolName, Timestamp};

/// One row of a spell's upcast/scaling table (Open5e v2
/// `casting_options`). Every field other than `kind` is an override of
/// the spell's default value; `None` means "unchanged".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpellCastingOption {
    /// `"default"`, `"slot_level_N"` (slot upcast), or
    /// `"player_level_N"` (cantrip scaling).
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_roll: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_count: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concentration: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape_size: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
}

/// A magical spell, structured per the Open5e v2 schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spell {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    /// FK -> `documents.uuid` (source attribution).
    pub document_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    /// 0 = cantrip; 1–9 otherwise.
    pub level: u8,
    /// FK -> `spell_schools.uuid`.
    pub school: EntityId,
    #[serde(default)]
    pub ritual: bool,
    pub concentration: bool,
    pub casting_time: String,
    /// Trigger clause for reaction-cast spells.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reaction_condition: Option<String>,
    /// Numeric range in `range_unit` units; 0 for self/touch.
    pub range: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range_unit: Option<String>,
    /// Human-readable rendering, e.g. `"150 feet"`, `"Self"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range_text: Option<String>,
    /// `"point"`, `"creature"`, `"object"`, `"area"`, …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_count: Option<i32>,
    /// AoE shape (`"sphere"`, `"cone"`, …) when the spell has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape_size: Option<f64>,
    pub duration: String,
    pub verbal: bool,
    pub somatic: bool,
    pub material: bool,
    /// The specific material component, when `material` is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_specified: Option<String>,
    /// Gold-piece cost of the material as a decimal string, when priced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_cost: Option<String>,
    #[serde(default)]
    pub material_consumed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saving_throw_ability: Option<AbilityScore>,
    /// Whether the spell uses a spell-attack roll.
    #[serde(default)]
    pub attack_roll: bool,
    /// Base damage dice expression, e.g. `"8d6"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_roll: Option<String>,
    #[serde(default)]
    pub damage_types: Vec<DamageTypeName>,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub higher_level: Option<String>,
    /// Upcast/scaling table; the `"default"` row mirrors the base fields.
    #[serde(default)]
    pub casting_options: Vec<SpellCastingOption>,
    /// Keys of classes that can cast the spell.
    #[serde(default)]
    pub classes: Vec<String>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Slim list-projection of a [`Spell`] — the shape returned by the
/// compendium list endpoint and stored in the `summary` column. Single
/// source of truth for "what a spell list row contains"; mirrors the
/// fields the clients' list views and filters read, so the wire shape is
/// defined here rather than in SQL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpellSummary {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub document_uuid: EntityId,
    pub key: String,
    pub slug: String,
    pub name: String,
    pub level: u8,
    pub school: EntityId,
    pub concentration: bool,
    pub ritual: bool,
    pub verbal: bool,
    pub somatic: bool,
    pub material: bool,
}

impl Spell {
    /// Derives the list-row summary from the full record.
    pub fn summary(&self) -> SpellSummary {
        SpellSummary {
            uuid: self.uuid,
            content_module_uuid: self.content_module_uuid,
            document_uuid: self.document_uuid,
            key: self.key.clone(),
            slug: self.slug.clone(),
            name: self.name.clone(),
            level: self.level,
            school: self.school,
            concentration: self.concentration,
            ritual: self.ritual,
            verbal: self.verbal,
            somatic: self.somatic,
            material: self.material,
        }
    }
}

#[cfg(test)]
mod summary_tests {
    use super::*;

    #[test]
    fn spell_summary_wire_shape_is_stable() {
        let s = SpellSummary {
            uuid: EntityId::nil(),
            content_module_uuid: EntityId::nil(),
            document_uuid: EntityId::nil(),
            key: "k".into(),
            slug: "s".into(),
            name: "n".into(),
            level: 3,
            school: EntityId::nil(),
            concentration: true,
            ritual: false,
            verbal: true,
            somatic: false,
            material: true,
        };
        let v = serde_json::to_value(&s).unwrap();
        let mut keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "concentration",
                "content_module_uuid",
                "document_uuid",
                "key",
                "level",
                "material",
                "name",
                "ritual",
                "school",
                "slug",
                "somatic",
                "uuid",
                "verbal",
            ]
        );
        // Booleans serialize as JSON true/false (not the SQL projection's
        // 0/1), which the clients' list filters rely on.
        assert_eq!(v["concentration"], serde_json::json!(true));
        assert_eq!(v["level"], serde_json::json!(3));
    }
}

/// One of the eight schools of magic. Lookup-table row backing the
/// closed-set `SpellSchoolName` enum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellSchool {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: SpellSchoolName,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    pub description: String,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
