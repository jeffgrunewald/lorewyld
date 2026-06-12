//! Deserialize-only models for Open5e v2 API responses. Shapes are kept
//! permissive (Options + defaults everywhere) so upstream nulls and
//! sparse fields never break a fetch.

use serde::{Deserialize, Deserializer};
use serde_json::Value;

/// Accepts `"25.00"`, `25`, `25.5`, or `null` and yields the decimal as
/// a string (the API serializes DRF decimals as strings, but be lenient).
pub fn de_decimal<'de, D: Deserializer<'de>>(de: D) -> Result<Option<String>, D::Error> {
    let v = Option::<Value>::deserialize(de)?;
    Ok(match v {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) if s.is_empty() => None,
        Some(Value::String(s)) => Some(s),
        Some(Value::Number(n)) => Some(n.to_string()),
        Some(other) => Some(other.to_string()),
    })
}

fn empty_as_none<'de, D: Deserializer<'de>>(de: D) -> Result<Option<String>, D::Error> {
    let v = Option::<String>::deserialize(de)?;
    Ok(v.filter(|s| !s.trim().is_empty()))
}

/// Tolerates explicit `null` for fields whose type has a natural default
/// (serde's `default` attribute only covers *absent* fields).
fn null_default<'de, D, T>(de: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(de)?.unwrap_or_default())
}

/// Upstream numeric fields occasionally arrive as prose (`"30 feet"`);
/// accept numbers, numeric strings, or strings with a leading number.
fn flex_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => {
            let lead: String = s
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                .collect();
            lead.parse().ok()
        }
        _ => None,
    }
}

fn de_flex_f64<'de, D: Deserializer<'de>>(de: D) -> Result<f64, D::Error> {
    let v = Option::<Value>::deserialize(de)?;
    Ok(v.as_ref().and_then(flex_f64).unwrap_or_default())
}

fn de_flex_opt_f64<'de, D: Deserializer<'de>>(de: D) -> Result<Option<f64>, D::Error> {
    let v = Option::<Value>::deserialize(de)?;
    Ok(v.as_ref().and_then(flex_f64))
}

#[derive(Debug, Clone, Deserialize)]
pub struct Stub {
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    pub key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocStub {
    pub key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EditionDesc {
    pub desc: String,
    #[serde(default, deserialize_with = "null_default")]
    pub document: String,
}

#[derive(Debug, Deserialize)]
pub struct DocumentRec {
    pub key: String,
    pub name: String,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub desc: Option<String>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub author: Option<String>,
    #[serde(default)]
    pub publication_date: Option<String>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub permalink: Option<String>,
    #[serde(default)]
    pub licenses: Vec<Stub>,
    pub publisher: Stub,
    pub gamesystem: Stub,
}

#[derive(Debug, Deserialize)]
pub struct LicenseRec {
    pub key: String,
    pub name: String,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub desc: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpellRec {
    pub key: String,
    pub name: String,
    pub desc: String,
    pub document: DocStub,
    pub level: u8,
    pub school: Stub,
    #[serde(default)]
    pub classes: Vec<Stub>,
    #[serde(default, deserialize_with = "null_default")]
    pub ritual: bool,
    #[serde(default, deserialize_with = "null_default")]
    pub concentration: bool,
    #[serde(default, deserialize_with = "null_default")]
    pub casting_time: String,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub reaction_condition: Option<String>,
    #[serde(default, deserialize_with = "de_flex_f64")]
    pub range: f64,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub range_unit: Option<String>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub range_text: Option<String>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub target_type: Option<String>,
    #[serde(default)]
    pub target_count: Option<i32>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub shape_type: Option<String>,
    #[serde(default, deserialize_with = "de_flex_opt_f64")]
    pub shape_size: Option<f64>,
    #[serde(default, deserialize_with = "null_default")]
    pub duration: String,
    #[serde(default, deserialize_with = "null_default")]
    pub verbal: bool,
    #[serde(default, deserialize_with = "null_default")]
    pub somatic: bool,
    #[serde(default, deserialize_with = "null_default")]
    pub material: bool,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub material_specified: Option<String>,
    #[serde(default, deserialize_with = "de_decimal")]
    pub material_cost: Option<String>,
    #[serde(default, deserialize_with = "null_default")]
    pub material_consumed: bool,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub saving_throw_ability: Option<String>,
    #[serde(default, deserialize_with = "null_default")]
    pub attack_roll: bool,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub damage_roll: Option<String>,
    #[serde(default)]
    pub damage_types: Vec<String>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub higher_level: Option<String>,
    #[serde(default)]
    pub casting_options: Vec<CastingOptionRec>,
}

#[derive(Debug, Deserialize)]
pub struct CastingOptionRec {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub damage_roll: Option<String>,
    #[serde(default)]
    pub target_count: Option<i32>,
    #[serde(default, deserialize_with = "de_flex_opt_f64")]
    pub range: Option<f64>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub duration: Option<String>,
    #[serde(default)]
    pub concentration: Option<bool>,
    #[serde(default, deserialize_with = "de_flex_opt_f64")]
    pub shape_size: Option<f64>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub desc: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct SpeedAll {
    #[serde(default, deserialize_with = "de_flex_f64")]
    pub walk: f64,
    #[serde(default, deserialize_with = "de_flex_f64")]
    pub crawl: f64,
    #[serde(default, deserialize_with = "null_default")]
    pub hover: bool,
    #[serde(default, deserialize_with = "de_flex_f64")]
    pub fly: f64,
    #[serde(default, deserialize_with = "de_flex_f64")]
    pub burrow: f64,
    #[serde(default, deserialize_with = "de_flex_f64")]
    pub climb: f64,
    #[serde(default, deserialize_with = "de_flex_f64")]
    pub swim: f64,
}

#[derive(Debug, Default, Deserialize)]
pub struct AbilityMap {
    #[serde(default, deserialize_with = "null_default")]
    pub strength: i32,
    #[serde(default, deserialize_with = "null_default")]
    pub dexterity: i32,
    #[serde(default, deserialize_with = "null_default")]
    pub constitution: i32,
    #[serde(default, deserialize_with = "null_default")]
    pub intelligence: i32,
    #[serde(default, deserialize_with = "null_default")]
    pub wisdom: i32,
    #[serde(default, deserialize_with = "null_default")]
    pub charisma: i32,
}

#[derive(Debug, Default, Deserialize)]
pub struct ResistancesRec {
    #[serde(default, deserialize_with = "empty_as_none")]
    pub damage_immunities_display: Option<String>,
    #[serde(default)]
    pub damage_immunities: Vec<Stub>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub damage_resistances_display: Option<String>,
    #[serde(default)]
    pub damage_resistances: Vec<Stub>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub damage_vulnerabilities_display: Option<String>,
    #[serde(default)]
    pub damage_vulnerabilities: Vec<Stub>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub condition_immunities_display: Option<String>,
    #[serde(default)]
    pub condition_immunities: Vec<Stub>,
}

#[derive(Debug, Default, Deserialize)]
pub struct LanguagesRec {
    #[serde(default, deserialize_with = "null_default")]
    pub as_string: String,
    #[serde(default)]
    pub data: Vec<Stub>,
}

#[derive(Debug, Deserialize)]
pub struct UsageLimitsRec {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub param: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AttackRec {
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    pub attack_type: String,
    #[serde(default, deserialize_with = "null_default")]
    pub to_hit_mod: i32,
    #[serde(default, deserialize_with = "de_flex_opt_f64")]
    pub reach: Option<f64>,
    #[serde(default, deserialize_with = "de_flex_opt_f64")]
    pub range: Option<f64>,
    #[serde(default, deserialize_with = "de_flex_opt_f64")]
    pub long_range: Option<f64>,
    #[serde(default, deserialize_with = "null_default")]
    pub target_creature_only: bool,
    #[serde(default)]
    pub damage_die_count: Option<i32>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub damage_die_type: Option<String>,
    #[serde(default)]
    pub damage_bonus: Option<i32>,
    #[serde(default)]
    pub damage_type: Option<Stub>,
    #[serde(default)]
    pub extra_damage_die_count: Option<i32>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub extra_damage_die_type: Option<String>,
    #[serde(default)]
    pub extra_damage_bonus: Option<i32>,
    #[serde(default)]
    pub extra_damage_type: Option<Stub>,
}

#[derive(Debug, Deserialize)]
pub struct ActionRec {
    pub name: String,
    pub desc: String,
    #[serde(default, deserialize_with = "null_default")]
    pub action_type: String,
    #[serde(default, deserialize_with = "null_default")]
    pub order_in_statblock: i32,
    #[serde(default)]
    pub legendary_action_cost: Option<i32>,
    #[serde(default)]
    pub usage_limits: Option<UsageLimitsRec>,
    #[serde(default)]
    pub attacks: Vec<AttackRec>,
}

#[derive(Debug, Deserialize)]
pub struct TraitRec {
    pub name: String,
    pub desc: String,
    #[serde(default, rename = "type", deserialize_with = "empty_as_none")]
    pub kind: Option<String>,
    #[serde(default)]
    pub order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreatureRec {
    pub key: String,
    pub name: String,
    pub document: DocStub,
    #[serde(rename = "type")]
    pub kind: Stub,
    pub size: Stub,
    pub challenge_rating: f64,
    #[serde(default)]
    pub proficiency_bonus: Option<i32>,
    #[serde(default)]
    pub experience_points: Option<i32>,
    #[serde(default, deserialize_with = "null_default")]
    pub alignment: String,
    #[serde(default)]
    pub speed_all: SpeedAll,
    #[serde(default)]
    pub languages: LanguagesRec,
    pub armor_class: i32,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub armor_detail: Option<String>,
    pub hit_points: i32,
    #[serde(default, deserialize_with = "null_default")]
    pub hit_dice: String,
    #[serde(default)]
    pub ability_scores: AbilityMap,
    #[serde(default)]
    pub modifiers: AbilityMap,
    #[serde(default, deserialize_with = "null_default")]
    pub initiative_bonus: i32,
    #[serde(default)]
    pub saving_throws: serde_json::Map<String, Value>,
    #[serde(default)]
    pub skill_bonuses: serde_json::Map<String, Value>,
    #[serde(default)]
    pub passive_perception: Option<i32>,
    #[serde(default)]
    pub resistances_and_immunities: ResistancesRec,
    #[serde(default)]
    pub darkvision_range: Option<i32>,
    #[serde(default)]
    pub blindsight_range: Option<i32>,
    #[serde(default)]
    pub tremorsense_range: Option<i32>,
    #[serde(default)]
    pub truesight_range: Option<i32>,
    #[serde(default)]
    pub actions: Vec<ActionRec>,
    #[serde(default)]
    pub traits: Vec<TraitRec>,
    #[serde(default)]
    pub environments: Vec<Value>,
}

#[derive(Debug, Deserialize)]
pub struct FeatureLevelRec {
    pub level: u8,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClassFeatureRec {
    #[serde(default, deserialize_with = "null_default")]
    pub key: String,
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    pub desc: String,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub feature_type: Option<String>,
    #[serde(default)]
    pub gained_at: Vec<FeatureLevelRec>,
}

#[derive(Debug, Default, Deserialize)]
pub struct HitPointsRec {
    #[serde(default, deserialize_with = "null_default")]
    pub hit_points_at_1st_level: String,
    #[serde(default, deserialize_with = "null_default")]
    pub hit_points_at_higher_levels: String,
}

#[derive(Debug, Deserialize)]
pub struct ClassRec {
    pub key: String,
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    pub desc: String,
    pub document: DocStub,
    #[serde(default, deserialize_with = "null_default")]
    pub hit_dice: String,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub caster_type: Option<String>,
    #[serde(default)]
    pub saving_throws: Vec<Stub>,
    #[serde(default)]
    pub hit_points: HitPointsRec,
    #[serde(default)]
    pub subclass_of: Option<Stub>,
    #[serde(default)]
    pub features: Vec<ClassFeatureRec>,
}

#[derive(Debug, Deserialize)]
pub struct SpeciesRec {
    pub key: String,
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    pub desc: String,
    pub document: DocStub,
    #[serde(default, deserialize_with = "null_default")]
    pub is_subspecies: bool,
    #[serde(default)]
    pub subspecies_of: Option<String>,
    #[serde(default)]
    pub traits: Vec<TraitRec>,
}

#[derive(Debug, Deserialize)]
pub struct BenefitRec {
    #[serde(default)]
    pub name: Option<String>,
    pub desc: String,
    #[serde(default, rename = "type", deserialize_with = "empty_as_none")]
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BackgroundRec {
    pub key: String,
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    pub desc: String,
    pub document: DocStub,
    #[serde(default)]
    pub benefits: Vec<BenefitRec>,
}

#[derive(Debug, Deserialize)]
pub struct FeatRec {
    pub key: String,
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    pub desc: String,
    pub document: DocStub,
    #[serde(default, rename = "type", deserialize_with = "empty_as_none")]
    pub kind: Option<String>,
    #[serde(default, deserialize_with = "null_default")]
    pub has_prerequisite: bool,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub prerequisite: Option<String>,
    #[serde(default)]
    pub benefits: Vec<BenefitRec>,
}

#[derive(Debug, Deserialize)]
pub struct WeaponPropertyUse {
    pub property: WeaponPropertyEmbed,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WeaponPropertyEmbed {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct WeaponRec {
    pub key: String,
    pub name: String,
    pub document: DocStub,
    #[serde(default, deserialize_with = "null_default")]
    pub damage_dice: String,
    #[serde(default)]
    pub damage_type: Option<Stub>,
    #[serde(default, deserialize_with = "de_flex_f64")]
    pub range: f64,
    #[serde(default, deserialize_with = "de_flex_f64")]
    pub long_range: f64,
    #[serde(default, deserialize_with = "null_default")]
    pub is_simple: bool,
    #[serde(default, deserialize_with = "null_default")]
    pub is_improvised: bool,
    #[serde(default)]
    pub properties: Vec<WeaponPropertyUse>,
}

#[derive(Debug, Deserialize)]
pub struct ArmorRec {
    pub key: String,
    pub name: String,
    pub document: DocStub,
    #[serde(default, deserialize_with = "null_default")]
    pub category: String,
    pub ac_base: i32,
    #[serde(default, deserialize_with = "null_default")]
    pub ac_add_dexmod: bool,
    #[serde(default)]
    pub ac_cap_dexmod: Option<i32>,
    #[serde(default, deserialize_with = "null_default")]
    pub ac_display: String,
    #[serde(default, deserialize_with = "null_default")]
    pub grants_stealth_disadvantage: bool,
    #[serde(default)]
    pub strength_score_required: Option<i32>,
}

/// Weapon/armor object embedded on an item: same shape as the standalone
/// record, plus a `key` linking to it. `name` is kept for fallback
/// resolution when edition dedup culled the exact key.
#[derive(Debug, Deserialize)]
pub struct EmbeddedKeyed {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ItemRec {
    pub key: String,
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    pub desc: String,
    pub document: DocStub,
    #[serde(default)]
    pub category: Option<Stub>,
    #[serde(default, deserialize_with = "de_decimal")]
    pub weight: Option<String>,
    #[serde(default, deserialize_with = "de_decimal")]
    pub cost: Option<String>,
    #[serde(default)]
    pub weapon: Option<EmbeddedKeyed>,
    #[serde(default)]
    pub armor: Option<EmbeddedKeyed>,
    #[serde(default)]
    pub rarity: Option<Stub>,
    #[serde(default, deserialize_with = "null_default")]
    pub requires_attunement: bool,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub attunement_detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConditionRec {
    pub key: String,
    pub name: String,
    #[serde(default)]
    pub document: Value,
    #[serde(default)]
    pub descriptions: Vec<EditionDesc>,
}

#[derive(Debug, Deserialize)]
pub struct LanguageRec {
    pub key: String,
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    pub desc: String,
    #[serde(default, deserialize_with = "null_default")]
    pub is_exotic: bool,
    #[serde(default, deserialize_with = "null_default")]
    pub is_secret: bool,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub script_language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SizeRec {
    pub key: String,
    pub name: String,
    pub rank: i32,
    #[serde(default, deserialize_with = "de_flex_opt_f64")]
    pub space_diameter: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct SkillRec {
    pub key: String,
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    pub ability: String,
    #[serde(default)]
    pub descriptions: Vec<EditionDesc>,
}

#[derive(Debug, Deserialize)]
pub struct AbilityRec {
    pub key: String,
    pub name: String,
    #[serde(default)]
    pub descriptions: Vec<EditionDesc>,
}

#[derive(Debug, Deserialize)]
pub struct AlignmentRec {
    pub key: String,
    #[serde(default, deserialize_with = "null_default")]
    pub morality: String,
    #[serde(default, deserialize_with = "null_default")]
    pub societal_attitude: String,
}

#[derive(Debug, Deserialize)]
pub struct SpellSchoolRec {
    pub key: String,
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    pub desc: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatureTypeRec {
    pub key: String,
    pub name: String,
    #[serde(default)]
    pub descriptions: Vec<EditionDesc>,
}

#[derive(Debug, Deserialize)]
pub struct ItemCategoryRec {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct EnvironmentRec {
    pub key: String,
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    pub desc: String,
}

#[derive(Debug, Deserialize)]
pub struct DamageTypeRec {
    pub key: String,
    pub name: String,
    #[serde(default)]
    pub descriptions: Vec<EditionDesc>,
}

#[derive(Debug, Deserialize)]
pub struct WeaponPropertyRec {
    pub key: String,
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    pub desc: String,
    #[serde(default, rename = "type", deserialize_with = "empty_as_none")]
    pub kind: Option<String>,
}
