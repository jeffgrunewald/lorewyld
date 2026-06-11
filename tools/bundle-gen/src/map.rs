//! Conversion from Open5e API records to `lorewyld-types` records.
//!
//! All UUIDs are derived with `content_uuid(type_tag, key)` so output is
//! deterministic; all timestamps use the pinned bundle epoch.

use std::collections::BTreeMap;

use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Utc};
use lorewyld_types::*;
use serde_json::Value;

use crate::v1;
use crate::v2;

/// Pinned timestamp written into every record so regeneration is
/// byte-stable.
pub fn bundle_epoch() -> Timestamp {
    DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .expect("valid pinned epoch")
        .with_timezone(&Utc)
}

pub fn slug_from_key(key: &str) -> String {
    key.rsplit('_').next().unwrap_or(key).to_string()
}

// ─── Enum parsing ────────────────────────────────────────────────────────

fn enum_from_str<T: serde::de::DeserializeOwned>(kind: &str, raw: &str) -> Result<T> {
    serde_json::from_value(Value::String(raw.to_string()))
        .map_err(|_| anyhow!("unrecognized {kind}: {raw:?}"))
}

pub fn ability_from_str(raw: &str) -> Result<AbilityScore> {
    let full = match raw.to_lowercase().as_str() {
        "str" => "strength",
        "dex" => "dexterity",
        "con" => "constitution",
        "int" => "intelligence",
        "wis" => "wisdom",
        "cha" => "charisma",
        other => return enum_from_str("ability", other),
    };
    enum_from_str("ability", full)
}

pub fn damage_type_from_str(raw: &str) -> Result<DamageTypeName> {
    enum_from_str("damage type", &raw.to_lowercase())
}

pub fn condition_from_str(raw: &str) -> Result<ConditionName> {
    enum_from_str("condition", &raw.to_lowercase())
}

pub fn school_from_str(raw: &str) -> Result<SpellSchoolName> {
    enum_from_str("spell school", &raw.to_lowercase())
}

pub fn creature_type_from_str(raw: &str) -> Result<CreatureTypeName> {
    enum_from_str("creature type", &raw.to_lowercase())
}

pub fn rarity_from_str(raw: &str) -> Result<Rarity> {
    enum_from_str("rarity", &raw.to_lowercase().replace(' ', "_"))
}

pub fn alignment_from_key(key: &str) -> Result<AlignmentName> {
    let normalized = match key {
        "neutral" => "true_neutral".to_string(),
        other => other.replace('-', "_"),
    };
    enum_from_str("alignment", &normalized)
}

// ─── Prose parsing helpers (v1 sheet-math recovery) ──────────────────────

/// Parses `"Choose two from Animal Handling, Athletics, and Survival"`.
/// Falls back to `choose: 0` with the prose preserved in `description`.
pub fn choice_from_prose(prose: &str) -> ChoiceFrom {
    let counts = [
        ("one", 1u32),
        ("two", 2),
        ("three", 3),
        ("four", 4),
        ("five", 5),
        ("six", 6),
    ];
    let lower = prose.to_lowercase();
    let choose = counts
        .iter()
        .find(|(word, _)| lower.starts_with(&format!("choose {word}")))
        .map(|(_, n)| *n);
    let from = lower
        .split_once(" from ")
        .map(|(_, rest)| {
            rest.trim_end_matches('.')
                .replace(" and ", ", ")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    match choose {
        Some(n) if !from.is_empty() => ChoiceFrom {
            choose: n,
            from,
            description: Some(prose.to_string()),
        },
        _ => ChoiceFrom {
            choose: 0,
            from: Vec::new(),
            description: Some(prose.to_string()),
        },
    }
}

/// Extracts the first unsigned integer in a string (`"30 feet"` -> 30).
pub fn first_int(text: &str) -> Option<i32> {
    let digits: String = text
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

fn none_if_none_str(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Standard proficiency bonus by CR when upstream omits it.
fn proficiency_bonus_for_cr(cr: f64) -> i32 {
    if cr < 5.0 { 2 } else { 2 + ((cr as i32 - 1) / 4) }
}

/// Picks the edition-appropriate description: prefer SRD 5.2, then
/// SRD 5.1, then whatever exists.
pub fn pick_desc(descs: &[v2::EditionDesc]) -> String {
    for doc in ["srd-2024", "srd-2014"] {
        if let Some(d) = descs.iter().find(|d| d.document == doc) {
            return d.desc.clone();
        }
    }
    descs.first().map(|d| d.desc.clone()).unwrap_or_default()
}

// ─── Mapping context ─────────────────────────────────────────────────────

/// Resolution state shared by all record mappers: the module identity,
/// per-document UUIDs, and key→UUID maps for every lookup table.
pub struct Ctx {
    pub module_uuid: EntityId,
    pub epoch: Timestamp,
    pub documents: BTreeMap<String, EntityId>,
    pub schools: BTreeMap<String, EntityId>,
    pub creature_types: BTreeMap<String, EntityId>,
    pub sizes: BTreeMap<String, EntityId>,
    pub size_uuid_by_name: BTreeMap<String, EntityId>,
    pub languages: BTreeMap<String, EntityId>,
    pub language_uuid_by_name: BTreeMap<String, EntityId>,
    pub item_categories: BTreeMap<String, EntityId>,
    pub weapon_property_uuid_by_name: BTreeMap<String, EntityId>,
    /// Final class name (lowercase) → final class key, built after dedup
    /// so spell class refs survive edition gap-fill.
    pub class_key_by_name: BTreeMap<String, String>,
}

impl Ctx {
    pub fn document_uuid(&self, doc_key: &str) -> Result<EntityId> {
        self.documents
            .get(doc_key)
            .copied()
            .ok_or_else(|| anyhow!("record references unknown document {doc_key:?}"))
    }
}

// ─── Record mappers ──────────────────────────────────────────────────────

pub fn map_spell(ctx: &Ctx, rec: &v2::SpellRec) -> Result<Spell> {
    let school_uuid = ctx
        .schools
        .get(&rec.school.key)
        .copied()
        .ok_or_else(|| anyhow!("spell {} references unknown school {}", rec.key, rec.school.key))?;
    let classes = rec
        .classes
        .iter()
        .filter_map(|stub| ctx.class_key_by_name.get(&stub.name.to_lowercase()).cloned())
        .collect();
    Ok(Spell {
        uuid: content_uuid("spell", &rec.key),
        content_module_uuid: ctx.module_uuid,
        document_uuid: ctx.document_uuid(&rec.document.key)?,
        name: rec.name.clone(),
        slug: slug_from_key(&rec.key),
        key: rec.key.clone(),
        level: rec.level,
        school: school_uuid,
        ritual: rec.ritual,
        concentration: rec.concentration,
        casting_time: rec.casting_time.clone(),
        reaction_condition: rec.reaction_condition.clone(),
        range: rec.range,
        range_unit: rec.range_unit.clone(),
        range_text: rec.range_text.clone(),
        target_type: rec.target_type.clone(),
        target_count: rec.target_count,
        shape_type: rec.shape_type.clone(),
        shape_size: rec.shape_size,
        duration: rec.duration.clone(),
        verbal: rec.verbal,
        somatic: rec.somatic,
        material: rec.material,
        material_specified: rec.material_specified.clone(),
        material_cost: rec.material_cost.clone(),
        material_consumed: rec.material_consumed,
        saving_throw_ability: rec
            .saving_throw_ability
            .as_deref()
            .map(ability_from_str)
            .transpose()
            .with_context(|| format!("spell {}", rec.key))?,
        attack_roll: rec.attack_roll,
        damage_roll: rec.damage_roll.clone(),
        damage_types: rec
            .damage_types
            .iter()
            .map(|s| damage_type_from_str(s))
            .collect::<Result<_>>()
            .with_context(|| format!("spell {}", rec.key))?,
        description: rec.desc.clone(),
        higher_level: rec.higher_level.clone(),
        casting_options: rec
            .casting_options
            .iter()
            .map(|o| SpellCastingOption {
                kind: o.kind.clone(),
                damage_roll: o.damage_roll.clone(),
                target_count: o.target_count,
                range: o.range,
                duration: o.duration.clone(),
                concentration: o.concentration,
                shape_size: o.shape_size,
                desc: o.desc.clone(),
            })
            .collect(),
        classes,
        is_restricted: false,
        created_at: ctx.epoch,
        updated_at: ctx.epoch,
    })
}

fn named_modifiers(map: &serde_json::Map<String, Value>) -> Vec<NamedModifier> {
    map.iter()
        .filter_map(|(name, v)| {
            v.as_i64().map(|bonus| NamedModifier {
                name: name.clone(),
                bonus: bonus as i32,
            })
        })
        .collect()
}

fn senses_summary(senses: &Senses) -> String {
    let mut parts = Vec::new();
    for (label, range) in [
        ("blindsight", senses.blindsight),
        ("darkvision", senses.darkvision),
        ("tremorsense", senses.tremorsense),
        ("truesight", senses.truesight),
    ] {
        if range > 0 {
            parts.push(format!("{label} {range} ft."));
        }
    }
    parts.push(format!("passive Perception {}", senses.passive_perception));
    parts.join(", ")
}

pub fn map_creature(ctx: &Ctx, rec: &v2::CreatureRec) -> Result<Creature> {
    let kind = ctx
        .creature_types
        .get(&rec.kind.key)
        .copied()
        .ok_or_else(|| anyhow!("creature {} has unknown type {}", rec.key, rec.kind.key))?;
    let size = ctx
        .sizes
        .get(&rec.size.key)
        .copied()
        .ok_or_else(|| anyhow!("creature {} has unknown size {}", rec.key, rec.size.key))?;
    let res = &rec.resistances_and_immunities;
    let parse_damage_list = |list: &[v2::Stub]| -> Vec<DamageTypeName> {
        list.iter()
            .filter_map(|s| damage_type_from_str(&s.key).ok())
            .collect()
    };
    // Upstream's srd-2014 structured attacks carry a corrupt
    // damage_type (nearly all "thunder"); the prose desc is
    // authoritative there, so drop the typed field for that edition.
    let trust_attack_damage_types = rec.document.key != "srd-2014";
    let mut senses = Senses {
        passive_perception: rec.passive_perception.unwrap_or_default(),
        darkvision: rec.darkvision_range.unwrap_or_default(),
        blindsight: rec.blindsight_range.unwrap_or_default(),
        tremorsense: rec.tremorsense_range.unwrap_or_default(),
        truesight: rec.truesight_range.unwrap_or_default(),
        summary: String::new(),
    };
    senses.summary = senses_summary(&senses);

    let actions = rec
        .actions
        .iter()
        .map(|a| {
            let kind = match a.action_type.as_str() {
                "ACTION" | "" => CreatureActionKind::Action,
                "BONUS_ACTION" => CreatureActionKind::BonusAction,
                "REACTION" => CreatureActionKind::Reaction,
                "LEGENDARY_ACTION" => CreatureActionKind::LegendaryAction,
                other => bail!("creature {} action {:?} has kind {other:?}", rec.key, a.name),
            };
            Ok(CreatureAction {
                name: a.name.clone(),
                desc: a.desc.clone(),
                kind,
                order: a.order_in_statblock,
                legendary_action_cost: a.legendary_action_cost,
                usage_limit: a.usage_limits.as_ref().map(|u| UsageLimit {
                    kind: u.kind.clone(),
                    param: u.param,
                }),
                attacks: a
                    .attacks
                    .iter()
                    .map(|atk| CreatureAttack {
                        name: atk.name.clone(),
                        attack_type: atk.attack_type.clone(),
                        to_hit_mod: atk.to_hit_mod,
                        reach: atk.reach,
                        range: atk.range,
                        long_range: atk.long_range,
                        target_creature_only: atk.target_creature_only,
                        damage_die_count: atk.damage_die_count,
                        damage_die_type: atk.damage_die_type.clone(),
                        damage_bonus: atk.damage_bonus,
                        damage_type: atk
                            .damage_type
                            .as_ref()
                            .filter(|_| trust_attack_damage_types)
                            .and_then(|s| damage_type_from_str(&s.key).ok()),
                        extra_damage_die_count: atk.extra_damage_die_count,
                        extra_damage_die_type: atk.extra_damage_die_type.clone(),
                        extra_damage_bonus: atk.extra_damage_bonus,
                        extra_damage_type: atk
                            .extra_damage_type
                            .as_ref()
                            .filter(|_| trust_attack_damage_types)
                            .and_then(|s| damage_type_from_str(&s.key).ok()),
                    })
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Creature {
        uuid: content_uuid("creature", &rec.key),
        content_module_uuid: ctx.module_uuid,
        document_uuid: ctx.document_uuid(&rec.document.key)?,
        name: rec.name.clone(),
        slug: slug_from_key(&rec.key),
        key: rec.key.clone(),
        kind,
        size,
        alignment: rec.alignment.clone(),
        challenge_rating: rec.challenge_rating as f32,
        proficiency_bonus: rec
            .proficiency_bonus
            .unwrap_or_else(|| proficiency_bonus_for_cr(rec.challenge_rating)),
        experience_points: rec.experience_points.unwrap_or_default(),
        armor_class: rec.armor_class,
        armor_detail: rec.armor_detail.clone(),
        hit_points: rec.hit_points,
        hit_dice: rec.hit_dice.clone(),
        speed: MovementSpeed {
            walk: rec.speed_all.walk as i32,
            fly: rec.speed_all.fly as i32,
            swim: rec.speed_all.swim as i32,
            climb: rec.speed_all.climb as i32,
            burrow: rec.speed_all.burrow as i32,
            crawl: rec.speed_all.crawl as i32,
            hover: rec.speed_all.hover,
        },
        ability_scores: ability_scores_from(&rec.ability_scores),
        modifiers: ability_scores_from(&rec.modifiers),
        initiative_bonus: rec.initiative_bonus,
        saving_throws: named_modifiers(&rec.saving_throws),
        skill_bonuses: named_modifiers(&rec.skill_bonuses),
        damage_resistances: parse_damage_list(&res.damage_resistances),
        damage_resistances_display: res.damage_resistances_display.clone(),
        damage_vulnerabilities: parse_damage_list(&res.damage_vulnerabilities),
        damage_vulnerabilities_display: res.damage_vulnerabilities_display.clone(),
        damage_immunities: parse_damage_list(&res.damage_immunities),
        damage_immunities_display: res.damage_immunities_display.clone(),
        condition_immunities: res
            .condition_immunities
            .iter()
            .filter_map(|s| condition_from_str(&s.key).ok())
            .collect(),
        condition_immunities_display: res.condition_immunities_display.clone(),
        senses,
        languages: rec.languages.as_string.clone(),
        languages_list: rec
            .languages
            .data
            .iter()
            .filter_map(|stub| ctx.languages.get(&stub.key).copied())
            .collect(),
        actions,
        traits: rec
            .traits
            .iter()
            .map(|t| CreatureTrait {
                name: t.name.clone(),
                desc: t.desc.clone(),
            })
            .collect(),
        legendary_desc: None,
        environments: rec
            .environments
            .iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.clone()),
                Value::Object(o) => o.get("key").and_then(Value::as_str).map(String::from),
                _ => None,
            })
            .collect(),
        is_restricted: false,
        created_at: ctx.epoch,
        updated_at: ctx.epoch,
    })
}

fn ability_scores_from(map: &v2::AbilityMap) -> AbilityScores {
    AbilityScores {
        strength: map.strength,
        dexterity: map.dexterity,
        constitution: map.constitution,
        intelligence: map.intelligence,
        wisdom: map.wisdom,
        charisma: map.charisma,
    }
}

/// Sheet-math overrides loaded from `data/overrides.json`.
pub struct Overrides {
    pub spell_slot_tables: BTreeMap<String, Value>,
    pub class_caster_kind: BTreeMap<String, String>,
    pub name_aliases: BTreeMap<String, String>,
    pub srd24_species_asi_desc: String,
    pub document_attribution: BTreeMap<String, String>,
}

impl Overrides {
    pub fn parse(raw: &str) -> Result<Self> {
        let v: Value = serde_json::from_str(raw).context("parsing overrides.json")?;
        let obj_map = |key: &str| -> BTreeMap<String, Value> {
            v.get(key)
                .and_then(Value::as_object)
                .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default()
        };
        let str_map = |key: &str| -> BTreeMap<String, String> {
            obj_map(key)
                .into_iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                .collect()
        };
        Ok(Self {
            spell_slot_tables: obj_map("spell_slot_tables"),
            class_caster_kind: str_map("class_caster_kind"),
            name_aliases: str_map("name_aliases"),
            srd24_species_asi_desc: v
                .get("srd24_species_asi_desc")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            document_attribution: v
                .get("documents")
                .and_then(Value::as_object)
                .map(|docs| {
                    docs.iter()
                        .filter_map(|(k, d)| {
                            d.get("attribution")
                                .and_then(Value::as_str)
                                .map(|a| (k.clone(), a.to_string()))
                        })
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

pub fn map_class(
    ctx: &Ctx,
    rec: &v2::ClassRec,
    v1_class: Option<&v1::V1Class>,
    overrides: &Overrides,
    parent_uuid: Option<EntityId>,
) -> Result<Class> {
    let slug = slug_from_key(&rec.key);
    let caster_kind = overrides.class_caster_kind.get(&slug);
    let spell_slot_table = caster_kind
        .and_then(|kind| overrides.spell_slot_tables.get(kind))
        .cloned();
    let is_subclass = parent_uuid.is_some();
    Ok(Class {
        uuid: content_uuid("class", &rec.key),
        content_module_uuid: ctx.module_uuid,
        document_uuid: ctx.document_uuid(&rec.document.key)?,
        name: rec.name.clone(),
        slug,
        key: rec.key.clone(),
        desc: rec.desc.clone(),
        subclass_of: parent_uuid,
        hit_dice: first_int(&rec.hit_dice).map(|n| n as u8),
        hp_at_1st_level: none_if_none_str(&rec.hit_points.hit_points_at_1st_level),
        hp_at_higher_levels: none_if_none_str(&rec.hit_points.hit_points_at_higher_levels),
        caster_type: rec.caster_type.clone(),
        prof_armor: v1_class.and_then(|c| none_if_none_str(&c.prof_armor)),
        prof_weapons: v1_class.and_then(|c| none_if_none_str(&c.prof_weapons)),
        prof_tools: v1_class.and_then(|c| none_if_none_str(&c.prof_tools)),
        prof_saving_throws: rec
            .saving_throws
            .iter()
            .filter_map(|s| ability_from_str(&s.name).ok())
            .collect(),
        prof_skills: v1_class
            .filter(|_| !is_subclass)
            .and_then(|c| none_if_none_str(&c.prof_skills))
            .map(|p| choice_from_prose(&p)),
        equipment: v1_class
            .filter(|_| !is_subclass)
            .and_then(|c| none_if_none_str(&c.equipment))
            .map(Value::String),
        spellcasting_ability: v1_class.and_then(|c| {
            c.spellcasting_ability
                .split_whitespace()
                .next()
                .and_then(|w| ability_from_str(w).ok())
        }),
        spell_slot_table: if is_subclass { None } else { spell_slot_table },
        subtypes_name: v1_class
            .filter(|_| !is_subclass)
            .and_then(|c| none_if_none_str(&c.subtypes_name)),
        features: rec
            .features
            .iter()
            .map(|f| ClassFeature {
                key: f.key.clone(),
                name: f.name.clone(),
                desc: f.desc.clone(),
                feature_type: f.feature_type.clone(),
                gained_at: f
                    .gained_at
                    .iter()
                    .map(|g| FeatureLevel {
                        level: g.level,
                        detail: g.detail.clone(),
                    })
                    .collect(),
            })
            .collect(),
        is_restricted: false,
        created_at: ctx.epoch,
        updated_at: ctx.epoch,
    })
}

/// Recovered sheet-math data for one species, sourced from the v1 API
/// (2014 species) or parsed from v2 prose traits (2024 species).
pub struct SpeciesSheetMath {
    pub asi: AbilityScores,
    pub asi_desc: String,
    pub speed: i32,
    pub size_name: String,
    pub languages_base: Vec<EntityId>,
    pub vision_base: Option<Senses>,
}

pub fn sheet_math_from_v1(ctx: &Ctx, race: &v1::V1Race) -> SpeciesSheetMath {
    let mut asi = AbilityScores::default();
    for bump in &race.asi {
        for attr in &bump.attributes {
            if let Ok(ability) = ability_from_str(attr) {
                let slot = match ability {
                    AbilityScore::Strength => &mut asi.strength,
                    AbilityScore::Dexterity => &mut asi.dexterity,
                    AbilityScore::Constitution => &mut asi.constitution,
                    AbilityScore::Intelligence => &mut asi.intelligence,
                    AbilityScore::Wisdom => &mut asi.wisdom,
                    AbilityScore::Charisma => &mut asi.charisma,
                };
                *slot += bump.value;
            }
        }
    }
    let languages_base = ctx
        .language_uuid_by_name
        .iter()
        .filter(|(name, _)| {
            race.languages
                .to_lowercase()
                .contains(&format!(" {name}").to_lowercase())
                || race.languages.to_lowercase().starts_with(name.as_str())
        })
        .map(|(_, uuid)| *uuid)
        .collect();
    let vision_base = race
        .vision
        .to_lowercase()
        .contains("darkvision")
        .then(|| Senses {
            darkvision: first_int(&race.vision).unwrap_or(60),
            summary: race.vision.clone(),
            ..Senses::default()
        });
    SpeciesSheetMath {
        asi,
        asi_desc: race.asi_desc.clone(),
        speed: race.speed.walk,
        size_name: race.size_raw.to_lowercase(),
        languages_base,
        vision_base,
    }
}

/// Parses speed/size out of a 2024 species' typed prose traits. ASI is
/// intentionally zero: the 2024 rules grant ability increases via
/// backgrounds (`overrides.srd24_species_asi_desc` explains this).
pub fn sheet_math_from_v2_traits(
    traits: &[v2::TraitRec],
    asi_desc: &str,
) -> SpeciesSheetMath {
    let trait_desc = |kind: &str| -> Option<&str> {
        traits
            .iter()
            .find(|t| t.kind.as_deref() == Some(kind))
            .map(|t| t.desc.as_str())
    };
    let speed = trait_desc("SPEED").and_then(first_int).unwrap_or(30);
    let size_name = trait_desc("SIZE")
        .and_then(|d| d.split_whitespace().next())
        .unwrap_or("Medium")
        .to_lowercase();
    let vision_base = traits
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case("darkvision"))
        .map(|t| Senses {
            darkvision: first_int(&t.desc).unwrap_or(60),
            summary: format!("darkvision {} ft.", first_int(&t.desc).unwrap_or(60)),
            ..Senses::default()
        });
    SpeciesSheetMath {
        asi: AbilityScores::default(),
        asi_desc: asi_desc.to_string(),
        speed,
        size_name,
        languages_base: Vec::new(),
        vision_base,
    }
}

pub fn map_species(
    ctx: &Ctx,
    rec: &v2::SpeciesRec,
    sheet: SpeciesSheetMath,
    parent_uuid: Option<EntityId>,
) -> Result<Species> {
    let size = ctx
        .size_uuid_by_name
        .get(&sheet.size_name)
        .copied()
        .ok_or_else(|| anyhow!("species {} has unknown size {:?}", rec.key, sheet.size_name))?;
    Ok(Species {
        uuid: content_uuid("species", &rec.key),
        content_module_uuid: ctx.module_uuid,
        document_uuid: ctx.document_uuid(&rec.document.key)?,
        name: rec.name.clone(),
        slug: slug_from_key(&rec.key),
        key: rec.key.clone(),
        desc: rec.desc.clone(),
        is_subspecies: rec.is_subspecies,
        subspecies_of: parent_uuid,
        traits: rec
            .traits
            .iter()
            .map(|t| SpeciesTrait {
                name: t.name.clone(),
                desc: t.desc.clone(),
                kind: t.kind.clone(),
                order: t.order.unwrap_or_default(),
            })
            .collect(),
        asi: sheet.asi,
        asi_desc: sheet.asi_desc,
        speed: sheet.speed,
        size,
        languages_base: sheet.languages_base,
        languages_additional: None,
        vision_base: sheet.vision_base,
        is_restricted: false,
        created_at: ctx.epoch,
        updated_at: ctx.epoch,
    })
}

pub fn map_background(ctx: &Ctx, rec: &v2::BackgroundRec) -> Result<Background> {
    Ok(Background {
        uuid: content_uuid("background", &rec.key),
        content_module_uuid: ctx.module_uuid,
        document_uuid: ctx.document_uuid(&rec.document.key)?,
        name: rec.name.clone(),
        slug: slug_from_key(&rec.key),
        key: rec.key.clone(),
        desc: rec.desc.clone(),
        benefits: rec
            .benefits
            .iter()
            .map(|b| BackgroundBenefit {
                name: b.name.clone().unwrap_or_default(),
                desc: b.desc.clone(),
                kind: b.kind.clone().unwrap_or_default(),
            })
            .collect(),
        is_restricted: false,
        created_at: ctx.epoch,
        updated_at: ctx.epoch,
    })
}

pub fn map_feat(ctx: &Ctx, rec: &v2::FeatRec) -> Result<Feat> {
    Ok(Feat {
        uuid: content_uuid("feat", &rec.key),
        content_module_uuid: ctx.module_uuid,
        document_uuid: ctx.document_uuid(&rec.document.key)?,
        name: rec.name.clone(),
        slug: slug_from_key(&rec.key),
        key: rec.key.clone(),
        desc: rec.desc.clone(),
        kind: rec.kind.clone(),
        has_prerequisite: rec.has_prerequisite,
        prerequisite: rec.prerequisite.clone(),
        benefits: rec
            .benefits
            .iter()
            .map(|b| FeatBenefit {
                name: b.name.clone(),
                desc: b.desc.clone(),
            })
            .collect(),
        is_restricted: false,
        created_at: ctx.epoch,
        updated_at: ctx.epoch,
    })
}

pub fn map_weapon(ctx: &Ctx, rec: &v2::WeaponRec) -> Result<Weapon> {
    Ok(Weapon {
        uuid: content_uuid("weapon", &rec.key),
        content_module_uuid: ctx.module_uuid,
        document_uuid: ctx.document_uuid(&rec.document.key)?,
        name: rec.name.clone(),
        slug: slug_from_key(&rec.key),
        key: rec.key.clone(),
        damage_dice: rec.damage_dice.clone(),
        damage_type: rec
            .damage_type
            .as_ref()
            .and_then(|s| damage_type_from_str(&s.key).ok()),
        range: (rec.range > 0.0).then_some(rec.range),
        long_range: (rec.long_range > 0.0).then_some(rec.long_range),
        is_simple: rec.is_simple,
        is_improvised: rec.is_improvised,
        properties: rec
            .properties
            .iter()
            .filter_map(|p| {
                ctx.weapon_property_uuid_by_name
                    .get(&p.property.name.to_lowercase())
                    .map(|uuid| WeaponPropertyRef {
                        property_uuid: *uuid,
                        detail: p.detail.clone(),
                    })
            })
            .collect(),
        is_restricted: false,
        created_at: ctx.epoch,
        updated_at: ctx.epoch,
    })
}

pub fn map_armor(ctx: &Ctx, rec: &v2::ArmorRec) -> Result<Armor> {
    Ok(Armor {
        uuid: content_uuid("armor", &rec.key),
        content_module_uuid: ctx.module_uuid,
        document_uuid: ctx.document_uuid(&rec.document.key)?,
        name: rec.name.clone(),
        slug: slug_from_key(&rec.key),
        key: rec.key.clone(),
        category: rec.category.clone(),
        ac_base: rec.ac_base,
        ac_add_dexmod: rec.ac_add_dexmod,
        ac_cap_dexmod: rec.ac_cap_dexmod,
        ac_display: rec.ac_display.clone(),
        grants_stealth_disadvantage: rec.grants_stealth_disadvantage,
        strength_score_required: rec.strength_score_required,
        is_restricted: false,
        created_at: ctx.epoch,
        updated_at: ctx.epoch,
    })
}

pub fn map_item(
    ctx: &Ctx,
    rec: &v2::ItemRec,
    is_magic: bool,
    weapon_uuid: Option<EntityId>,
    armor_uuid: Option<EntityId>,
) -> Result<Item> {
    let category = rec
        .category
        .as_ref()
        .ok_or_else(|| anyhow!("item {} has no category", rec.key))?;
    let category_uuid = ctx
        .item_categories
        .get(&category.key)
        .copied()
        .ok_or_else(|| anyhow!("item {} has unknown category {}", rec.key, category.key))?;
    Ok(Item {
        uuid: content_uuid("item", &rec.key),
        content_module_uuid: ctx.module_uuid,
        document_uuid: ctx.document_uuid(&rec.document.key)?,
        name: rec.name.clone(),
        slug: slug_from_key(&rec.key),
        key: rec.key.clone(),
        desc: rec.desc.clone(),
        category_uuid,
        weight: rec
            .weight
            .as_deref()
            .and_then(|w| w.parse().ok())
            .unwrap_or_default(),
        cost: rec.cost.clone(),
        weapon_uuid,
        armor_uuid,
        rarity: rec
            .rarity
            .as_ref()
            .map(|r| rarity_from_str(&r.name))
            .transpose()
            .with_context(|| format!("item {}", rec.key))?,
        requires_attunement: rec.requires_attunement,
        attunement_detail: rec.attunement_detail.clone(),
        is_magic,
        is_restricted: false,
        created_at: ctx.epoch,
        updated_at: ctx.epoch,
    })
}
