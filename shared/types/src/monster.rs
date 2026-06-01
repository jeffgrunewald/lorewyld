use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{
    AbilityScore, AbilityScores, ConditionName, CreatureSize, DamageRoll, DamageTypeName, EntityId,
    Json, MovementSpeed, NamedModifier, Senses, Timestamp,
};

/// Classification of monster type (Aberration, Beast, etc.).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonsterType {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: MonsterTypeName,
    pub slug: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Closed-set of SRD monster classifications.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MonsterTypeName {
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

/// Kind discriminator for `MonsterAction` rows so a single table can
/// hold actions, bonus actions, reactions, legendary actions, and lair
/// actions.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonsterActionKind {
    Action,
    BonusAction,
    Reaction,
    Legendary,
    Lair,
}

/// A reusable action, reaction, or legendary-action definition that
/// can be shared across multiple monsters (Multiattack, Longsword, etc.).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonsterAction {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    #[serde(rename = "type")]
    pub kind: MonsterActionKind,
    pub desc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_bonus: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reach: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage: Option<DamageRoll>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub save_dc: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub save_type: Option<AbilityScore>,
    /// Free text for save consequence — e.g. "half on success".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub save_detail: Option<String>,
    /// Legendary-action cost or per-round limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses_per_round: Option<i32>,
    /// Minimum d6 result to recharge between uses (e.g. 5 for "5-6").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recharge_on_roll: Option<i32>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Spellcasting stat block embedded on a monster.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Spellcasting {
    /// Caster level for slot-based casting; 0 for innate-only casters.
    #[serde(default)]
    pub level: i32,
    /// Spell slots available per spell level, in `{level: count}` form.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slots: Option<Json>,
    /// Spell entries the monster knows — typeshare-opaque because the
    /// SRD nests level + count + spell list inconsistently.
    #[serde(default)]
    pub spells: Vec<Json>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ability: Option<AbilityScore>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bonus: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub save_dc: Option<i32>,
    /// Always-on/at-will spells separate from slot-based ones.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub innate: Option<Json>,
}

/// A creature stat block from the SRD.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Monster {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    /// FK -> `monster_types.uuid`.
    #[serde(rename = "type")]
    pub kind: EntityId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
    pub name: String,
    pub slug: String,
    pub size: CreatureSize,
    /// FK -> `alignments.uuid`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alignment: Option<EntityId>,
    pub armor_class: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armor_type: Option<String>,
    pub hit_points: i32,
    /// Hit-dice expression as printed in the stat block, e.g. `"33 (6d8)"`.
    pub hit_dice: String,
    pub speed: MovementSpeed,
    pub ability_scores: AbilityScores,
    /// Derived modifiers; supplied explicitly so consumers don't have to
    /// re-derive against the `ability_scores.modifier_table` lookup.
    pub ability_modifiers: AbilityScores,
    #[serde(default)]
    pub saving_throws: Vec<NamedModifier>,
    #[serde(default)]
    pub skills: Vec<NamedModifier>,
    #[serde(default)]
    pub damage_resistances: Vec<DamageTypeName>,
    #[serde(default)]
    pub damage_vulnerabilities: Vec<DamageTypeName>,
    #[serde(default)]
    pub damage_immunities: Vec<DamageTypeName>,
    #[serde(default)]
    pub condition_immunities: Vec<ConditionName>,
    pub senses: Senses,
    /// Free-text language list as printed in the SRD.
    #[serde(default)]
    pub languages: String,
    /// FK refs into `languages.uuid` (kept alongside the free-text
    /// rendering for structured queries).
    #[serde(default)]
    pub languages_list: Vec<EntityId>,
    pub challenge_rating: f32,
    pub xp: i32,
    #[serde(default)]
    pub actions: Vec<Json>,
    #[serde(default)]
    pub bonus_actions: Vec<Json>,
    #[serde(default)]
    pub reactions: Vec<Json>,
    #[serde(default)]
    pub legendary_actions: Vec<Json>,
    #[serde(default)]
    pub lair_actions: Vec<Json>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legendary_desc: Option<String>,
    #[serde(default)]
    pub special_abilities: Vec<Json>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spellcasting: Option<Spellcasting>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
