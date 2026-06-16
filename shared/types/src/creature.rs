use serde::{Deserialize, Serialize};

use crate::common::{
    AbilityScores, ConditionName, DamageTypeName, EntityId, MovementSpeed, NamedModifier, Senses,
    Timestamp,
};

/// Closed-set of SRD creature classifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CreatureTypeName {
    Aberration,
    Beast,
    Celestial,
    Construct,
    Dragon,
    Elemental,
    Fey,
    Fiend,
    Giant,
    Humanoid,
    Monstrosity,
    Ooze,
    Plant,
    Undead,
}

/// Classification of creature type (Aberration, Beast, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatureType {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: CreatureTypeName,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A terrain/habitat tag creatures can be filtered by (Open5e v2
/// `environments`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Kind discriminator for entries in a creature's `actions` array.
/// Wire form matches Open5e v2's `action_type` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreatureActionKind {
    Action,
    BonusAction,
    Reaction,
    LegendaryAction,
}

/// How often a limited-use action recharges (Open5e v2 `usage_limits`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageLimit {
    /// `"PER_DAY"`, `"RECHARGE_ON_ROLL"`, `"RECHARGE_AFTER_REST"`, …
    #[serde(rename = "type")]
    pub kind: String,
    /// Uses per day, or minimum d6 result to recharge, per `kind`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<i32>,
}

/// A structured attack roll inside a creature action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatureAttack {
    pub name: String,
    /// `"WEAPON"` or `"SPELL"`.
    pub attack_type: String,
    pub to_hit_mod: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reach: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub long_range: Option<f64>,
    #[serde(default)]
    pub target_creature_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_die_count: Option<i32>,
    /// Die face label, e.g. `"D6"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_die_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_bonus: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_type: Option<DamageTypeName>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_damage_die_count: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_damage_die_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_damage_bonus: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_damage_type: Option<DamageTypeName>,
}

/// One entry in a creature's stat-block action list. A single ordered
/// array holds actions, bonus actions, reactions, and legendary actions;
/// `kind` discriminates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatureAction {
    pub name: String,
    pub desc: String,
    #[serde(rename = "type")]
    pub kind: CreatureActionKind,
    /// Position within the stat block, for faithful rendering.
    #[serde(default)]
    pub order: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legendary_action_cost: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_limit: Option<UsageLimit>,
    #[serde(default)]
    pub attacks: Vec<CreatureAttack>,
}

/// A non-action stat-block trait (Amphibious, Magic Resistance,
/// Spellcasting, …). Spellcasting remains prose, matching the SRD.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatureTrait {
    pub name: String,
    pub desc: String,
}

/// A creature stat block, structured per the Open5e v2 schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Creature {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    /// FK -> `documents.uuid` (source attribution).
    pub document_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    /// FK -> `creature_types.uuid`.
    #[serde(rename = "type")]
    pub kind: EntityId,
    /// FK -> `sizes.uuid`.
    pub size: EntityId,
    /// Free text as printed in the stat block (`"chaotic evil"`,
    /// `"any alignment"`, `"unaligned"`). Not an FK: the SRD prints
    /// values outside the nine-point grid.
    pub alignment: String,
    /// Numeric CR; fractional values are exact (0.125 = 1/8).
    pub challenge_rating: f32,
    pub proficiency_bonus: i32,
    pub experience_points: i32,
    pub armor_class: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armor_detail: Option<String>,
    pub hit_points: i32,
    /// Dice expression, e.g. `"18d10+72"`.
    pub hit_dice: String,
    pub speed: MovementSpeed,
    pub ability_scores: AbilityScores,
    /// Derived modifiers, supplied explicitly so consumers don't
    /// re-derive them.
    pub modifiers: AbilityScores,
    #[serde(default)]
    pub initiative_bonus: i32,
    /// Proficient saving throws only.
    #[serde(default)]
    pub saving_throws: Vec<NamedModifier>,
    /// Proficient skill bonuses only.
    #[serde(default)]
    pub skill_bonuses: Vec<NamedModifier>,
    #[serde(default)]
    pub damage_resistances: Vec<DamageTypeName>,
    /// Conditional prose the typed array can't carry (e.g.
    /// "bludgeoning from nonmagical attacks").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_resistances_display: Option<String>,
    #[serde(default)]
    pub damage_vulnerabilities: Vec<DamageTypeName>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_vulnerabilities_display: Option<String>,
    #[serde(default)]
    pub damage_immunities: Vec<DamageTypeName>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_immunities_display: Option<String>,
    #[serde(default)]
    pub condition_immunities: Vec<ConditionName>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition_immunities_display: Option<String>,
    pub senses: Senses,
    /// Free-text language list as printed in the SRD.
    #[serde(default)]
    pub languages: String,
    /// FK refs into `languages.uuid`, kept alongside the free text for
    /// structured queries.
    #[serde(default)]
    pub languages_list: Vec<EntityId>,
    /// Single ordered list; `kind` discriminates action class.
    #[serde(default)]
    pub actions: Vec<CreatureAction>,
    #[serde(default)]
    pub traits: Vec<CreatureTrait>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legendary_desc: Option<String>,
    /// Environment keys the creature appears in.
    #[serde(default)]
    pub environments: Vec<String>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Slim list-projection of a [`Creature`]. Single source of truth for the
/// creature list-row shape (see `Creature::summary`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatureSummary {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub document_uuid: EntityId,
    pub key: String,
    pub slug: String,
    pub name: String,
    pub challenge_rating: f32,
    #[serde(rename = "type")]
    pub kind: EntityId,
    pub size: EntityId,
}

#[cfg(test)]
mod summary_tests {
    use super::*;

    #[test]
    fn creature_summary_renames_kind_to_type() {
        let s = CreatureSummary {
            uuid: EntityId::nil(),
            content_module_uuid: EntityId::nil(),
            document_uuid: EntityId::nil(),
            key: "k".into(),
            slug: "s".into(),
            name: "n".into(),
            challenge_rating: 0.25,
            kind: EntityId::nil(),
            size: EntityId::nil(),
        };
        let v = serde_json::to_value(&s).unwrap();
        let obj = v.as_object().unwrap();
        // The FK serializes as "type" (matching the record + clients), not "kind".
        assert!(obj.contains_key("type"));
        assert!(!obj.contains_key("kind"));
        // Fractional CR survives as a JSON number.
        assert_eq!(v["challenge_rating"], serde_json::json!(0.25));
    }
}

impl Creature {
    /// Derives the list-row summary from the full record.
    pub fn summary(&self) -> CreatureSummary {
        CreatureSummary {
            uuid: self.uuid,
            content_module_uuid: self.content_module_uuid,
            document_uuid: self.document_uuid,
            key: self.key.clone(),
            slug: self.slug.clone(),
            name: self.name.clone(),
            challenge_rating: self.challenge_rating,
            kind: self.kind,
            size: self.size,
        }
    }
}
