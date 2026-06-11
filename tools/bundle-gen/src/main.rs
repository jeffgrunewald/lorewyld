//! Generates `content/srd-bundle.json` (and the byte-identical mobile
//! asset copy) from the Open5e v2 API: SRD 5.2 as the base, gap-filled
//! by name with SRD 5.1 records, plus a v1-API join recovering the
//! sheet-math data v2 dropped to prose.

mod dedup;
mod fetch;
mod map;
mod v1;
mod v2;

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, Result, bail, ensure};
use chrono::NaiveDate;
use clap::Parser;
use lorewyld_types::*;
use serde_json::Value;

use fetch::Fetcher;
use map::{Ctx, Overrides, pick_desc, slug_from_key};

const API: &str = "https://api.open5e.com";
const BASE_DOC: &str = "srd-2024";
const LEGACY_DOC: &str = "srd-2014";
const V1_DOC_SLUG: &str = "wotc-srd";

#[derive(Parser)]
#[command(about = "Generate the shipped SRD content bundle from Open5e")]
struct Cli {
    /// Cache directory for raw API pages (gitignored; delete to refetch).
    #[arg(long, default_value = "tools/bundle-gen/.cache")]
    cache_dir: PathBuf,
    /// Canonical output path.
    #[arg(long, default_value = "content/srd-bundle.json")]
    out: PathBuf,
    /// Mobile asset copy (byte-identical to the canonical output).
    #[arg(long, default_value = "mobile/assets/content/srd-bundle.json")]
    mobile_out: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let fetcher = Fetcher::new(cli.cache_dir.clone())?;
    let overrides = Overrides::parse(include_str!("../data/overrides.json"))?;
    let epoch = map::bundle_epoch();
    let module_uuid = content_uuid("module", "srd");

    let mut report = Report::default();
    let bundle = build_bundle(&fetcher, &overrides, module_uuid, epoch, &mut report)?;

    let mut json = serde_json::to_string_pretty(&bundle)?;
    json.push('\n');
    for path in [&cli.out, &cli.mobile_out] {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, &json).with_context(|| format!("writing {}", path.display()))?;
    }

    report.print(&bundle, &json);
    Ok(())
}

#[derive(Default)]
struct Report {
    gap_filled: BTreeMap<&'static str, Vec<String>>,
    skipped: Vec<String>,
}

impl Report {
    fn print(&self, bundle: &ContentBundle, json: &str) {
        eprintln!("\n── bundle contents ──");
        let counts: Vec<(&str, usize)> = vec![
            ("licenses", bundle.licenses.len()),
            ("publishers", bundle.publishers.len()),
            ("documents", bundle.documents.len()),
            ("ability_scores", bundle.ability_scores.len()),
            ("skills", bundle.skills.len()),
            ("alignments", bundle.alignments.len()),
            ("damage_types", bundle.damage_types.len()),
            ("conditions", bundle.conditions.len()),
            ("languages", bundle.languages.len()),
            ("sizes", bundle.sizes.len()),
            ("environments", bundle.environments.len()),
            ("spell_schools", bundle.spell_schools.len()),
            ("creature_types", bundle.creature_types.len()),
            ("item_categories", bundle.item_categories.len()),
            ("weapon_properties", bundle.weapon_properties.len()),
            ("spells", bundle.spells.len()),
            ("creatures", bundle.creatures.len()),
            ("classes", bundle.classes.len()),
            ("species", bundle.species.len()),
            ("feats", bundle.feats.len()),
            ("backgrounds", bundle.backgrounds.len()),
            ("weapons", bundle.weapons.len()),
            ("armors", bundle.armors.len()),
            ("items", bundle.items.len()),
        ];
        for (name, count) in counts {
            eprintln!("{name:>20}: {count}");
        }
        eprintln!("{:>20}: {:.1} MiB", "size", json.len() as f64 / 1048576.0);
        eprintln!("\n── gap-filled from SRD 5.1 (review for unintended duplicates) ──");
        for (family, names) in &self.gap_filled {
            eprintln!("{family} (+{}): {}", names.len(), names.join(", "));
        }
        if !self.skipped.is_empty() {
            eprintln!("\n── skipped records ──");
            for line in &self.skipped {
                eprintln!("  {line}");
            }
        }
    }
}

fn doc_key_of(v: &Value) -> &str {
    match v {
        Value::String(s) => s.as_str(),
        Value::Object(o) => o.get("key").and_then(Value::as_str).unwrap_or(""),
        _ => "",
    }
}

fn build_bundle(
    fetcher: &Fetcher,
    overrides: &Overrides,
    module_uuid: EntityId,
    epoch: Timestamp,
    report: &mut Report,
) -> Result<ContentBundle> {
    let aliases = &overrides.name_aliases;
    let in_docs = |key: &str| key == BASE_DOC || key == LEGACY_DOC;

    // ── Provenance ──────────────────────────────────────────────────────
    let all_docs: Vec<v2::DocumentRec> = fetcher.fetch_all(&format!("{API}/v2/documents/"))?;
    let all_licenses: Vec<v2::LicenseRec> = fetcher.fetch_all(&format!("{API}/v2/licenses/"))?;
    let our_docs: Vec<&v2::DocumentRec> =
        all_docs.iter().filter(|d| in_docs(&d.key)).collect();
    ensure!(our_docs.len() == 2, "expected both SRD documents upstream");

    let cc_license = |doc: &v2::DocumentRec| -> Result<String> {
        doc.licenses
            .iter()
            .find(|l| l.key == "cc-by-40")
            .map(|l| l.key.clone())
            .ok_or_else(|| anyhow::anyhow!("document {} lacks a CC-BY-4.0 license", doc.key))
    };

    let mut licenses = Vec::new();
    let mut publishers = Vec::new();
    let mut documents = Vec::new();
    for doc in &our_docs {
        let license_key = cc_license(doc)?;
        let license_rec = all_licenses
            .iter()
            .find(|l| l.key == license_key)
            .context("license record missing upstream")?;
        let license_uuid = content_uuid("license", &license_key);
        if !licenses.iter().any(|l: &License| l.key == license_key) {
            licenses.push(License {
                uuid: license_uuid,
                content_module_uuid: module_uuid,
                name: license_rec.name.clone(),
                slug: slug_from_key(&license_key),
                key: license_key.clone(),
                url: Some("https://creativecommons.org/licenses/by/4.0/legalcode".into()),
                text: license_rec.desc.clone(),
                is_restricted: false,
                created_at: epoch,
                updated_at: epoch,
            });
        }
        let publisher_uuid = content_uuid("publisher", &doc.publisher.key);
        if !publishers.iter().any(|p: &Publisher| p.key == doc.publisher.key) {
            publishers.push(Publisher {
                uuid: publisher_uuid,
                content_module_uuid: module_uuid,
                name: doc.publisher.name.clone(),
                slug: slug_from_key(&doc.publisher.key),
                key: doc.publisher.key.clone(),
                url: None,
                is_restricted: false,
                created_at: epoch,
                updated_at: epoch,
            });
        }
        documents.push(Document {
            uuid: content_uuid("document", &doc.key),
            content_module_uuid: module_uuid,
            name: doc.name.clone(),
            slug: slug_from_key(&doc.key),
            key: doc.key.clone(),
            desc: overrides
                .document_attribution
                .get(&doc.key)
                .cloned()
                .or_else(|| doc.desc.clone()),
            license_uuid,
            publisher_uuid,
            gamesystem_key: doc.gamesystem.key.clone(),
            permalink: doc.permalink.clone(),
            author: doc.author.clone(),
            published_on: doc
                .publication_date
                .as_deref()
                .and_then(|d| NaiveDate::parse_from_str(&d[..d.len().min(10)], "%Y-%m-%d").ok()),
            is_restricted: false,
            created_at: epoch,
            updated_at: epoch,
        });
    }

    // ── Lookup tables ───────────────────────────────────────────────────
    let school_recs: Vec<v2::SpellSchoolRec> =
        fetcher.fetch_all(&format!("{API}/v2/spellschools/"))?;
    let spell_schools: Vec<SpellSchool> = school_recs
        .iter()
        .filter_map(|r| {
            let name = map::school_from_str(&r.name).ok()?;
            Some(SpellSchool {
                uuid: content_uuid("spellschool", &r.key),
                content_module_uuid: module_uuid,
                name,
                slug: slug_from_key(&r.key),
                key: r.key.clone(),
                description: r.desc.clone(),
                is_restricted: false,
                created_at: epoch,
                updated_at: epoch,
            })
        })
        .collect();

    let ct_recs: Vec<v2::CreatureTypeRec> =
        fetcher.fetch_all(&format!("{API}/v2/creaturetypes/"))?;
    let creature_types: Vec<CreatureType> = ct_recs
        .iter()
        .filter_map(|r| {
            let name = map::creature_type_from_str(&r.name).ok()?;
            Some(CreatureType {
                uuid: content_uuid("creaturetype", &r.key),
                content_module_uuid: module_uuid,
                name,
                slug: slug_from_key(&r.key),
                key: r.key.clone(),
                description: Some(pick_desc(&r.descriptions)).filter(|d| !d.is_empty()),
                is_restricted: false,
                created_at: epoch,
                updated_at: epoch,
            })
        })
        .collect();

    let size_recs: Vec<v2::SizeRec> = fetcher.fetch_all(&format!("{API}/v2/sizes/"))?;
    let sizes: Vec<Size> = size_recs
        .iter()
        .map(|r| Size {
            uuid: content_uuid("size", &r.key),
            content_module_uuid: module_uuid,
            name: r.name.clone(),
            slug: slug_from_key(&r.key),
            key: r.key.clone(),
            rank: r.rank,
            space_diameter: r.space_diameter,
            is_restricted: false,
            created_at: epoch,
            updated_at: epoch,
        })
        .collect();

    let language_recs: Vec<v2::LanguageRec> =
        fetcher.fetch_all(&format!("{API}/v2/languages/"))?;
    let languages: Vec<Language> = language_recs
        .iter()
        .map(|r| Language {
            uuid: content_uuid("language", &r.key),
            content_module_uuid: module_uuid,
            name: r.name.clone(),
            slug: slug_from_key(&r.key),
            key: r.key.clone(),
            desc: r.desc.clone(),
            is_exotic: r.is_exotic,
            is_secret: r.is_secret,
            script: r.script_language.clone(),
            is_restricted: false,
            created_at: epoch,
            updated_at: epoch,
        })
        .collect();

    let skill_recs: Vec<v2::SkillRec> = fetcher.fetch_all(&format!("{API}/v2/skills/"))?;
    let skills: Vec<Skill> = skill_recs
        .iter()
        // Unprefixed keys are the core/SRD skill set; prefixed ones
        // (a5e-ag_culture) belong to other game systems.
        .filter(|r| !r.key.contains('_'))
        .filter_map(|r| {
            let ability = map::ability_from_str(&r.ability).ok()?;
            Some(Skill {
                uuid: content_uuid("skill", &r.key),
                content_module_uuid: module_uuid,
                name: r.name.clone(),
                slug: slug_from_key(&r.key),
                key: r.key.clone(),
                ability,
                description: pick_desc(&r.descriptions),
                is_restricted: false,
                created_at: epoch,
                updated_at: epoch,
            })
        })
        .collect();

    let ability_recs: Vec<v2::AbilityRec> = fetcher.fetch_all(&format!("{API}/v2/abilities/"))?;
    let modifier_table: Value = (1..=30)
        .map(|score| (score.to_string(), Value::from((score - 10) / 2)))
        .collect::<serde_json::Map<_, _>>()
        .into();
    let ability_scores: Vec<AbilityScoreEntry> = ability_recs
        .iter()
        .filter_map(|r| {
            let name = map::ability_from_str(&r.key).ok()?;
            Some(AbilityScoreEntry {
                uuid: content_uuid("ability", &r.key),
                content_module_uuid: module_uuid,
                name,
                slug: r.name.to_lowercase(),
                key: r.key.clone(),
                short_name: r.key.to_uppercase(),
                description: pick_desc(&r.descriptions),
                modifier_table: modifier_table.clone(),
                is_restricted: false,
                created_at: epoch,
                updated_at: epoch,
            })
        })
        .collect();

    let alignment_recs: Vec<v2::AlignmentRec> =
        fetcher.fetch_all(&format!("{API}/v2/alignments/"))?;
    let alignments: Vec<Alignment> = alignment_recs
        .iter()
        .filter_map(|r| {
            let name = map::alignment_from_key(&r.key).ok()?;
            Some(Alignment {
                uuid: content_uuid("alignment", &r.key),
                content_module_uuid: module_uuid,
                name,
                slug: slug_from_key(&r.key),
                key: r.key.clone(),
                is_lawful: r.societal_attitude == "lawful",
                is_neutral: r.societal_attitude == "neutral" || r.morality == "neutral",
                is_chaotic: r.societal_attitude == "chaotic",
                is_good: r.morality == "good",
                is_evil: r.morality == "evil",
                is_restricted: false,
                created_at: epoch,
                updated_at: epoch,
            })
        })
        .collect();

    let dt_recs: Vec<v2::DamageTypeRec> = fetcher.fetch_all(&format!("{API}/v2/damagetypes/"))?;
    let damage_types: Vec<DamageType> = dt_recs
        .iter()
        .filter_map(|r| {
            let name = map::damage_type_from_str(&r.name).ok()?;
            Some(DamageType {
                uuid: content_uuid("damagetype", &r.key),
                content_module_uuid: module_uuid,
                name,
                slug: slug_from_key(&r.key),
                key: r.key.clone(),
                description: pick_desc(&r.descriptions),
                resistances_against: Vec::new(),
                immunities_against: Vec::new(),
                is_restricted: false,
                created_at: epoch,
                updated_at: epoch,
            })
        })
        .collect();

    let condition_recs: Vec<v2::ConditionRec> =
        fetcher.fetch_all(&format!("{API}/v2/conditions/"))?;
    let mut conditions_by_name: BTreeMap<String, &v2::ConditionRec> = BTreeMap::new();
    for rec in &condition_recs {
        if map::condition_from_str(&rec.name).is_err() {
            continue;
        }
        let entry = conditions_by_name.entry(rec.name.to_lowercase());
        // Prefer the 5.2 record, then 5.1, then anything.
        let rank = |r: &v2::ConditionRec| match doc_key_of(&r.document) {
            BASE_DOC => 0,
            LEGACY_DOC => 1,
            _ => 2,
        };
        entry
            .and_modify(|existing| {
                if rank(rec) < rank(existing) {
                    *existing = rec;
                }
            })
            .or_insert(rec);
    }
    let conditions: Vec<Condition> = conditions_by_name
        .values()
        .map(|r| Condition {
            uuid: content_uuid("condition", &r.key),
            content_module_uuid: module_uuid,
            name: map::condition_from_str(&r.name).expect("filtered above"),
            slug: slug_from_key(&r.key),
            key: r.key.clone(),
            desc: pick_desc(&r.descriptions),
            is_restricted: false,
            created_at: epoch,
            updated_at: epoch,
        })
        .collect();

    let env_recs: Vec<v2::EnvironmentRec> =
        fetcher.fetch_all(&format!("{API}/v2/environments/"))?;
    let environments: Vec<Environment> = env_recs
        .iter()
        .map(|r| Environment {
            uuid: content_uuid("environment", &r.key),
            content_module_uuid: module_uuid,
            name: r.name.clone(),
            slug: slug_from_key(&r.key),
            key: r.key.clone(),
            description: Some(r.desc.clone())
                .filter(|d| !d.is_empty() && d != "[None provided]"),
            is_restricted: false,
            created_at: epoch,
            updated_at: epoch,
        })
        .collect();

    let ic_recs: Vec<v2::ItemCategoryRec> =
        fetcher.fetch_all(&format!("{API}/v2/itemcategories/"))?;
    let item_categories: Vec<ItemCategory> = ic_recs
        .iter()
        .map(|r| ItemCategory {
            uuid: content_uuid("itemcategory", &r.key),
            content_module_uuid: module_uuid,
            name: r.name.clone(),
            slug: slug_from_key(&r.key),
            key: r.key.clone(),
            is_restricted: false,
            created_at: epoch,
            updated_at: epoch,
        })
        .collect();

    let wp_recs: Vec<v2::WeaponPropertyRec> =
        fetcher.fetch_all(&format!("{API}/v2/weaponproperties/"))?;
    let mut wp_by_name: BTreeMap<String, &v2::WeaponPropertyRec> = BTreeMap::new();
    for rec in &wp_recs {
        let rank = |r: &v2::WeaponPropertyRec| {
            if r.key.starts_with(BASE_DOC) {
                0
            } else if r.key.starts_with("srd") {
                1
            } else {
                2
            }
        };
        if rank(rec) == 2 {
            continue; // non-SRD game systems
        }
        wp_by_name
            .entry(rec.name.to_lowercase())
            .and_modify(|existing| {
                if rank(rec) < rank(existing) {
                    *existing = rec;
                }
            })
            .or_insert(rec);
    }
    let weapon_properties: Vec<WeaponPropertyDef> = wp_by_name
        .values()
        .map(|r| WeaponPropertyDef {
            uuid: content_uuid("weaponproperty", &r.key),
            content_module_uuid: module_uuid,
            name: r.name.clone(),
            slug: slug_from_key(&r.key),
            key: r.key.clone(),
            kind: r.kind.clone(),
            description: r.desc.clone(),
            is_restricted: false,
            created_at: epoch,
            updated_at: epoch,
        })
        .collect();

    // ── Mapping context ─────────────────────────────────────────────────
    let mut ctx = Ctx {
        module_uuid,
        epoch,
        documents: documents.iter().map(|d| (d.key.clone(), d.uuid)).collect(),
        schools: spell_schools.iter().map(|s| (s.key.clone(), s.uuid)).collect(),
        creature_types: creature_types.iter().map(|c| (c.key.clone(), c.uuid)).collect(),
        sizes: sizes.iter().map(|s| (s.key.clone(), s.uuid)).collect(),
        size_uuid_by_name: sizes
            .iter()
            .map(|s| (s.name.to_lowercase(), s.uuid))
            .collect(),
        languages: languages.iter().map(|l| (l.key.clone(), l.uuid)).collect(),
        language_uuid_by_name: languages
            .iter()
            .map(|l| (l.name.to_lowercase(), l.uuid))
            .collect(),
        item_categories: item_categories.iter().map(|c| (c.key.clone(), c.uuid)).collect(),
        weapon_property_uuid_by_name: weapon_properties
            .iter()
            .map(|w| (w.name.to_lowercase(), w.uuid))
            .collect(),
        class_key_by_name: BTreeMap::new(),
    };

    // ── Classes (needed before spells for class-key resolution) ────────
    let fetch_doc = |endpoint: &str, doc: &str| -> String {
        format!("{API}/v2/{endpoint}/?document__key={doc}&limit=200")
    };
    let class_recs_base: Vec<v2::ClassRec> =
        fetcher.fetch_all(&fetch_doc("classes", BASE_DOC))?;
    let class_recs_legacy: Vec<v2::ClassRec> =
        fetcher.fetch_all(&fetch_doc("classes", LEGACY_DOC))?;
    ensure!(class_recs_base.len() >= 20, "suspiciously few SRD 5.2 classes");
    ensure!(class_recs_legacy.len() >= 20, "suspiciously few SRD 5.1 classes");

    let split = |recs: Vec<v2::ClassRec>| -> (Vec<v2::ClassRec>, Vec<v2::ClassRec>) {
        recs.into_iter().partition(|r| r.subclass_of.is_none())
    };
    let (base_parents, base_subs) = split(class_recs_base);
    let (legacy_parents, legacy_subs) = split(class_recs_legacy);
    let (parent_recs, filled) =
        dedup::gap_fill(base_parents, legacy_parents, |r| r.name.clone(), aliases);
    report.gap_filled.insert("classes", filled);
    let (sub_recs, filled) =
        dedup::gap_fill(base_subs, legacy_subs, |r| r.name.clone(), aliases);
    report.gap_filled.insert("subclasses", filled);

    let v1_classes: Vec<v1::V1Class> =
        fetcher.fetch_all(&format!("{API}/v1/classes/?document__slug={V1_DOC_SLUG}&limit=50"))?;
    let v1_class_by_name: BTreeMap<String, &v1::V1Class> = v1_classes
        .iter()
        .map(|c| (c.name.to_lowercase(), c))
        .collect();

    let parent_key_by_name: BTreeMap<String, String> = parent_recs
        .iter()
        .map(|r| (r.name.to_lowercase(), r.key.clone()))
        .collect();
    ctx.class_key_by_name = parent_key_by_name.clone();

    let mut classes = Vec::new();
    for rec in &parent_recs {
        let v1c = v1_class_by_name.get(&rec.name.to_lowercase()).copied();
        classes.push(map::map_class(&ctx, rec, v1c, overrides, None)?);
    }
    for rec in &sub_recs {
        let parent_stub = rec.subclass_of.as_ref().expect("partitioned as subclass");
        let Some(parent_key) = parent_key_by_name.get(&parent_stub.name.to_lowercase()) else {
            report
                .skipped
                .push(format!("subclass {} (parent {} not shipped)", rec.key, parent_stub.name));
            continue;
        };
        let parent_uuid = content_uuid("class", parent_key);
        let v1c = v1_class_by_name
            .get(&parent_stub.name.to_lowercase())
            .copied();
        classes.push(map::map_class(&ctx, rec, v1c, overrides, Some(parent_uuid))?);
    }

    // ── Spells ──────────────────────────────────────────────────────────
    let spells_base: Vec<v2::SpellRec> = fetcher.fetch_all(&fetch_doc("spells", BASE_DOC))?;
    let spells_legacy: Vec<v2::SpellRec> =
        fetcher.fetch_all(&fetch_doc("spells", LEGACY_DOC))?;
    ensure!(spells_base.len() >= 300, "suspiciously few SRD 5.2 spells");
    ensure!(spells_legacy.len() >= 300, "suspiciously few SRD 5.1 spells");
    let (spell_recs, filled) =
        dedup::gap_fill(spells_base, spells_legacy, |r| r.name.clone(), aliases);
    report.gap_filled.insert("spells", filled);
    let spells = spell_recs
        .iter()
        .map(|r| map::map_spell(&ctx, r))
        .collect::<Result<Vec<_>>>()?;

    // ── Creatures ───────────────────────────────────────────────────────
    let creatures_base: Vec<v2::CreatureRec> =
        fetcher.fetch_all(&fetch_doc("creatures", BASE_DOC))?;
    let creatures_legacy: Vec<v2::CreatureRec> =
        fetcher.fetch_all(&fetch_doc("creatures", LEGACY_DOC))?;
    ensure!(creatures_base.len() >= 300, "suspiciously few SRD 5.2 creatures");
    ensure!(creatures_legacy.len() >= 300, "suspiciously few SRD 5.1 creatures");
    let (creature_recs, filled) =
        dedup::gap_fill(creatures_base, creatures_legacy, |r| r.name.clone(), aliases);
    report.gap_filled.insert("creatures", filled);
    let creatures = creature_recs
        .iter()
        .map(|r| map::map_creature(&ctx, r))
        .collect::<Result<Vec<_>>>()?;

    // ── Species ─────────────────────────────────────────────────────────
    let species_base: Vec<v2::SpeciesRec> =
        fetcher.fetch_all(&fetch_doc("species", BASE_DOC))?;
    let species_legacy: Vec<v2::SpeciesRec> =
        fetcher.fetch_all(&fetch_doc("species", LEGACY_DOC))?;
    let legacy_species_name_by_key: BTreeMap<String, String> = species_legacy
        .iter()
        .chain(species_base.iter())
        .map(|r| (r.key.clone(), r.name.clone()))
        .collect();
    let (sp_parents, sp_subs): (Vec<_>, Vec<_>) = {
        let split = |recs: Vec<v2::SpeciesRec>| -> (Vec<v2::SpeciesRec>, Vec<v2::SpeciesRec>) {
            recs.into_iter().partition(|r| !r.is_subspecies)
        };
        let (bp, bs) = split(species_base);
        let (lp, ls) = split(species_legacy);
        let (parents, filled) = dedup::gap_fill(bp, lp, |r| r.name.clone(), aliases);
        report.gap_filled.insert("species", filled);
        let (subs, filled) = dedup::gap_fill(bs, ls, |r| r.name.clone(), aliases);
        report.gap_filled.insert("subspecies", filled);
        (parents, subs)
    };

    let v1_races: Vec<v1::V1Race> =
        fetcher.fetch_all(&format!("{API}/v1/races/?document__slug={V1_DOC_SLUG}&limit=50"))?;
    let v1_race_by_name: BTreeMap<String, &v1::V1Race> =
        v1_races.iter().map(|r| (r.name.to_lowercase(), r)).collect();
    let v1_subrace_by_name: BTreeMap<String, (&v1::V1Race, &v1::V1Subrace)> = v1_races
        .iter()
        .flat_map(|race| race.subraces.iter().map(move |sub| (sub, race)))
        .map(|(sub, race)| (sub.name.to_lowercase(), (race, sub)))
        .collect();

    // Returns the sheet math plus an optional "skipped" note for the
    // report (kept out of the closure to avoid a unique borrow on it).
    let species_sheet = |rec: &v2::SpeciesRec,
                         parent_name: Option<&str>|
     -> (map::SpeciesSheetMath, Option<String>) {
        if rec.document.key == BASE_DOC {
            return (
                map::sheet_math_from_v2_traits(&rec.traits, &overrides.srd24_species_asi_desc),
                None,
            );
        }
        let name = rec.name.to_lowercase();
        if let Some(race) = v1_race_by_name.get(&name) {
            return (map::sheet_math_from_v1(&ctx, race), None);
        }
        if let Some((race, sub)) = v1_subrace_by_name.get(&name) {
            let parent = map::sheet_math_from_v1(&ctx, race);
            let sub_asi = map::sheet_math_from_v1(
                &ctx,
                &v1::V1Race {
                    name: sub.name.clone(),
                    asi: sub
                        .asi
                        .iter()
                        .map(|a| v1::V1Asi {
                            attributes: a.attributes.clone(),
                            value: a.value,
                        })
                        .collect(),
                    asi_desc: sub.asi_desc.clone(),
                    size_raw: race.size_raw.clone(),
                    speed: v1::V1Speed { walk: race.speed.walk },
                    languages: String::new(),
                    vision: String::new(),
                    subraces: Vec::new(),
                },
            );
            return (
                map::SpeciesSheetMath {
                    asi: sub_asi.asi,
                    asi_desc: sub.asi_desc.clone(),
                    speed: parent.speed,
                    size_name: parent.size_name,
                    languages_base: Vec::new(),
                    vision_base: None,
                },
                None,
            );
        }
        let note = format!(
            "species {} ({}): no v1 sheet-math match{}",
            rec.key,
            rec.name,
            parent_name
                .map(|p| format!(" (parent {p})"))
                .unwrap_or_default()
        );
        (
            map::sheet_math_from_v2_traits(&rec.traits, &rec.name),
            Some(note),
        )
    };

    let parent_species_key_by_name: BTreeMap<String, String> = sp_parents
        .iter()
        .map(|r| (r.name.to_lowercase(), r.key.clone()))
        .collect();
    let mut species = Vec::new();
    for rec in &sp_parents {
        let (sheet, note) = species_sheet(rec, None);
        report.skipped.extend(note);
        species.push(map::map_species(&ctx, rec, sheet, None)?);
    }
    for rec in &sp_subs {
        let parent_name = rec
            .subspecies_of
            .as_ref()
            .and_then(|k| legacy_species_name_by_key.get(k))
            .map(|n| n.to_lowercase());
        let Some(parent_key) = parent_name
            .as_ref()
            .and_then(|n| parent_species_key_by_name.get(n))
        else {
            report.skipped.push(format!(
                "subspecies {} (parent {:?} not shipped)",
                rec.key, rec.subspecies_of
            ));
            continue;
        };
        let (sheet, note) = species_sheet(rec, parent_name.as_deref());
        report.skipped.extend(note);
        let parent_uuid = content_uuid("species", parent_key);
        species.push(map::map_species(&ctx, rec, sheet, Some(parent_uuid))?);
    }

    // ── Backgrounds & feats ─────────────────────────────────────────────
    let bg_base: Vec<v2::BackgroundRec> =
        fetcher.fetch_all(&fetch_doc("backgrounds", BASE_DOC))?;
    let bg_legacy: Vec<v2::BackgroundRec> =
        fetcher.fetch_all(&fetch_doc("backgrounds", LEGACY_DOC))?;
    let (bg_recs, filled) = dedup::gap_fill(bg_base, bg_legacy, |r| r.name.clone(), aliases);
    report.gap_filled.insert("backgrounds", filled);
    let backgrounds = bg_recs
        .iter()
        .map(|r| map::map_background(&ctx, r))
        .collect::<Result<Vec<_>>>()?;

    let feat_base: Vec<v2::FeatRec> = fetcher.fetch_all(&fetch_doc("feats", BASE_DOC))?;
    let feat_legacy: Vec<v2::FeatRec> = fetcher.fetch_all(&fetch_doc("feats", LEGACY_DOC))?;
    let (feat_recs, filled) =
        dedup::gap_fill(feat_base, feat_legacy, |r| r.name.clone(), aliases);
    report.gap_filled.insert("feats", filled);
    let feats = feat_recs
        .iter()
        .map(|r| map::map_feat(&ctx, r))
        .collect::<Result<Vec<_>>>()?;

    // ── Gear: weapons, armor, items ─────────────────────────────────────
    // The document__key filter is broken on some of these endpoints
    // (returns the unfiltered set), so always filter client-side on the
    // embedded document key.
    let weapon_recs_all: Vec<v2::WeaponRec> =
        fetcher.fetch_all(&format!("{API}/v2/weapons/?limit=200"))?;
    let (w_base, w_legacy): (Vec<_>, Vec<_>) = weapon_recs_all
        .into_iter()
        .filter(|r| in_docs(&r.document.key))
        .partition(|r| r.document.key == BASE_DOC);
    ensure!(w_base.len() >= 30, "suspiciously few SRD 5.2 weapons");
    let (weapon_recs, filled) =
        dedup::gap_fill(w_base, w_legacy, |r| r.name.clone(), aliases);
    report.gap_filled.insert("weapons", filled);
    let weapons = weapon_recs
        .iter()
        .map(|r| map::map_weapon(&ctx, r))
        .collect::<Result<Vec<_>>>()?;

    let armor_recs_all: Vec<v2::ArmorRec> =
        fetcher.fetch_all(&format!("{API}/v2/armor/?limit=200"))?;
    let (a_base, a_legacy): (Vec<_>, Vec<_>) = armor_recs_all
        .into_iter()
        .filter(|r| in_docs(&r.document.key))
        .partition(|r| r.document.key == BASE_DOC);
    ensure!(a_base.len() >= 10, "suspiciously few SRD 5.2 armors");
    let (armor_recs, filled) = dedup::gap_fill(a_base, a_legacy, |r| r.name.clone(), aliases);
    report.gap_filled.insert("armors", filled);
    let armors = armor_recs
        .iter()
        .map(|r| map::map_armor(&ctx, r))
        .collect::<Result<Vec<_>>>()?;

    let weapon_uuid_by_key: BTreeMap<&str, EntityId> =
        weapons.iter().map(|w| (w.key.as_str(), w.uuid)).collect();
    let weapon_uuid_by_name: BTreeMap<String, EntityId> = weapons
        .iter()
        .map(|w| (dedup::normalize(&w.name, aliases), w.uuid))
        .collect();
    let armor_uuid_by_key: BTreeMap<&str, EntityId> =
        armors.iter().map(|a| (a.key.as_str(), a.uuid)).collect();
    let armor_uuid_by_name: BTreeMap<String, EntityId> = armors
        .iter()
        .map(|a| (dedup::normalize(&a.name, aliases), a.uuid))
        .collect();

    let item_recs_all: Vec<v2::ItemRec> =
        fetcher.fetch_all(&format!("{API}/v2/items/?limit=200"))?;
    let magic_recs_all: Vec<v2::ItemRec> =
        fetcher.fetch_all(&format!("{API}/v2/magicitems/?limit=200"))?;
    type TaggedItems = Vec<(v2::ItemRec, bool)>;
    let partition_items = |recs: Vec<v2::ItemRec>, is_magic: bool| -> (TaggedItems, TaggedItems) {
        recs.into_iter()
            .filter(|r| in_docs(&r.document.key))
            .map(|r| (r, is_magic))
            .partition(|(r, _)| r.document.key == BASE_DOC)
    };
    let (mut i_base, mut i_legacy) = partition_items(item_recs_all, false);
    let (m_base, m_legacy) = partition_items(magic_recs_all, true);
    i_base.extend(m_base);
    i_legacy.extend(m_legacy);
    ensure!(i_base.len() >= 200, "suspiciously few SRD 5.2 items");
    ensure!(i_legacy.len() >= 200, "suspiciously few SRD 5.1 items");
    let (item_recs, filled) =
        dedup::gap_fill(i_base, i_legacy, |(r, _)| r.name.clone(), aliases);
    report.gap_filled.insert("items", filled);

    let mut items = Vec::new();
    for (rec, is_magic) in &item_recs {
        let resolve = |embed: &Option<v2::EmbeddedKeyed>,
                       by_key: &BTreeMap<&str, EntityId>,
                       by_name: &BTreeMap<String, EntityId>|
         -> Option<EntityId> {
            let embed = embed.as_ref()?;
            by_key
                .get(embed.key.as_str())
                .or_else(|| by_name.get(&dedup::normalize(&embed.name, aliases)))
                .copied()
        };
        let weapon_uuid = resolve(&rec.weapon, &weapon_uuid_by_key, &weapon_uuid_by_name);
        let armor_uuid = resolve(&rec.armor, &armor_uuid_by_key, &armor_uuid_by_name);
        if rec.weapon.is_some() && weapon_uuid.is_none() {
            report
                .skipped
                .push(format!("item {}: unresolved base weapon ref", rec.key));
        }
        if rec.armor.is_some() && armor_uuid.is_none() {
            report
                .skipped
                .push(format!("item {}: unresolved base armor ref", rec.key));
        }
        items.push(map::map_item(&ctx, rec, *is_magic, weapon_uuid, armor_uuid)?);
    }

    // ── Module + assembly ───────────────────────────────────────────────
    let module = ContentModule {
        uuid: module_uuid,
        name: "System Reference Document".to_string(),
        slug: "srd".to_string(),
        license: "CC-BY-4.0".to_string(),
        license_url: Some("https://creativecommons.org/licenses/by/4.0/legalcode".to_string()),
        schema_version: SCHEMA_VERSION,
        release_date: None,
        authors: vec!["Wizards of the Coast".to_string()],
        publisher: Some("Wizards of the Coast".to_string()),
        description: Some(
            "D&D 5e System Reference Document content: SRD 5.2 (2024 rules) as the base, \
             gap-filled with SRD 5.1 (2014 rules) records absent from 5.2. \
             Sourced via the Open5e API."
                .to_string(),
        ),
        website_url: Some("https://open5e.com".to_string()),
        is_active: true,
        ordering: 0,
        version_string: "1.0.0".to_string(),
        previous_version_uuid: None,
        published_at: None,
        created_at: epoch,
        updated_at: epoch,
    };

    let mut bundle = ContentBundle {
        schema: SchemaVersion::current(),
        modules: vec![module],
        licenses,
        publishers,
        documents,
        ability_scores,
        skills,
        alignments,
        damage_types,
        conditions,
        languages,
        sizes,
        environments,
        spell_schools,
        creature_types,
        item_categories,
        weapon_properties,
        spells,
        creatures,
        classes,
        species,
        feats,
        backgrounds,
        weapons,
        armors,
        items,
    };
    sort_bundle(&mut bundle);
    validate_unique_keys(&bundle)?;
    Ok(bundle)
}

fn sort_bundle(bundle: &mut ContentBundle) {
    bundle.licenses.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.publishers.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.documents.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.ability_scores.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.skills.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.alignments.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.damage_types.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.conditions.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.languages.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.sizes.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.environments.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.spell_schools.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.creature_types.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.item_categories.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.weapon_properties.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.spells.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.creatures.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.classes.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.species.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.feats.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.backgrounds.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.weapons.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.armors.sort_by(|a, b| a.key.cmp(&b.key));
    bundle.items.sort_by(|a, b| a.key.cmp(&b.key));
}

fn validate_unique_keys(bundle: &ContentBundle) -> Result<()> {
    fn check(family: &str, keys: impl Iterator<Item = String>) -> Result<()> {
        let mut seen = std::collections::BTreeSet::new();
        for key in keys {
            if !seen.insert(key.clone()) {
                bail!("duplicate {family} key in bundle: {key}");
            }
        }
        Ok(())
    }
    check("spell", bundle.spells.iter().map(|r| r.key.clone()))?;
    check("creature", bundle.creatures.iter().map(|r| r.key.clone()))?;
    check("class", bundle.classes.iter().map(|r| r.key.clone()))?;
    check("species", bundle.species.iter().map(|r| r.key.clone()))?;
    check("item", bundle.items.iter().map(|r| r.key.clone()))?;
    check("weapon", bundle.weapons.iter().map(|r| r.key.clone()))?;
    check("armor", bundle.armors.iter().map(|r| r.key.clone()))?;
    check("feat", bundle.feats.iter().map(|r| r.key.clone()))?;
    check("background", bundle.backgrounds.iter().map(|r| r.key.clone()))?;
    Ok(())
}
