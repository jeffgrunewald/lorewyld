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
    /// Tiny manifest of module slugs — the mobile app's fast seeding
    /// check, so launches don't decode the full bundle just to learn
    /// nothing is missing.
    #[arg(long, default_value = "mobile/assets/content/srd-bundle.meta.json")]
    mobile_meta_out: PathBuf,
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
    let meta = serde_json::json!({
        "module_slugs": bundle.modules.iter().map(|m| m.slug.clone()).collect::<Vec<_>>(),
    });
    let mut meta_json = serde_json::to_string_pretty(&meta)?;
    meta_json.push('\n');
    std::fs::write(&cli.mobile_meta_out, &meta_json)
        .with_context(|| format!("writing {}", cli.mobile_meta_out.display()))?;

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
            ("modules", bundle.modules.len()),
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
    let srd_module_uuid = module_uuid;
    let is_srd = |key: &str| key == BASE_DOC || key == LEGACY_DOC;

    // ── Provenance ──────────────────────────────────────────────────────
    // Every upstream document carrying a supported license ships. The
    // SRD pair shares the 'srd' module (5.1 gap-fills 5.2); every other
    // document becomes its own content module, so sources group cleanly
    // and attribution stays per-book.
    let all_docs: Vec<v2::DocumentRec> = fetcher.fetch_all(&format!("{API}/v2/documents/"))?;
    let all_licenses: Vec<v2::LicenseRec> = fetcher.fetch_all(&format!("{API}/v2/licenses/"))?;

    // Preference order when a document carries several licenses.
    let preferred_license = |doc: &v2::DocumentRec| -> Option<(LicenseKind, String)> {
        ["cc-by-40", "ogl-10a"].iter().find_map(|key| {
            doc.licenses
                .iter()
                .find(|l| l.key == *key)
                .and_then(|l| LicenseKind::from_open5e_key(&l.key).map(|k| (k, l.key.clone())))
        })
    };

    let mut included: Vec<&v2::DocumentRec> = all_docs
        .iter()
        .filter(|d| {
            let supported = preferred_license(d).is_some();
            if !supported {
                report
                    .skipped
                    .push(format!("document {} ({}): no supported license", d.key, d.name));
            }
            supported
        })
        .collect();
    included.sort_by(|a, b| a.key.cmp(&b.key));
    ensure!(
        included.iter().any(|d| d.key == BASE_DOC) && included.iter().any(|d| d.key == LEGACY_DOC),
        "expected both SRD documents upstream"
    );

    let module_uuid_for_doc = |doc_key: &str| -> EntityId {
        if is_srd(doc_key) {
            srd_module_uuid
        } else {
            content_uuid("module", doc_key)
        }
    };
    let license_url_for = |kind: LicenseKind| -> Option<String> {
        match kind {
            LicenseKind::CcBy40 => {
                Some("https://creativecommons.org/licenses/by/4.0/legalcode".into())
            }
            LicenseKind::Ogl10a | LicenseKind::Unlicensed => None,
        }
    };

    let mut licenses: Vec<License> = Vec::new();
    let mut publishers: Vec<Publisher> = Vec::new();
    let mut documents = Vec::new();
    let mut modules: Vec<ContentModule> = vec![srd_module(srd_module_uuid, epoch)];
    for doc in &included {
        let (license_kind, license_key) =
            preferred_license(doc).expect("filtered to supported licenses");
        let license_uuid = content_uuid("license", &license_key);
        if !licenses.iter().any(|l| l.key == license_key) {
            let license_rec = all_licenses
                .iter()
                .find(|l| l.key == license_key)
                .context("license record missing upstream")?;
            licenses.push(License {
                uuid: license_uuid,
                // Shared vocabulary rows live in the always-present SRD
                // module.
                content_module_uuid: srd_module_uuid,
                name: license_rec.name.clone(),
                slug: slug_from_key(&license_key),
                key: license_key.clone(),
                url: license_url_for(license_kind),
                text: license_rec.desc.clone(),
                is_restricted: false,
                created_at: epoch,
                updated_at: epoch,
            });
        }
        let publisher_uuid = content_uuid("publisher", &doc.publisher.key);
        if !publishers.iter().any(|p| p.key == doc.publisher.key) {
            publishers.push(Publisher {
                uuid: publisher_uuid,
                content_module_uuid: srd_module_uuid,
                name: doc.publisher.name.clone(),
                slug: slug_from_key(&doc.publisher.key),
                key: doc.publisher.key.clone(),
                url: None,
                is_restricted: false,
                created_at: epoch,
                updated_at: epoch,
            });
        }

        let owning_module = module_uuid_for_doc(&doc.key);
        let published_on = doc
            .publication_date
            .as_deref()
            .and_then(|d| NaiveDate::parse_from_str(&d[..d.len().min(10)], "%Y-%m-%d").ok());
        documents.push(Document {
            uuid: content_uuid("document", &doc.key),
            content_module_uuid: owning_module,
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
            published_on,
            is_restricted: false,
            created_at: epoch,
            updated_at: epoch,
        });
        if !is_srd(&doc.key) {
            modules.push(ContentModule {
                uuid: owning_module,
                name: doc.name.clone(),
                slug: slug_from_key(&doc.key),
                license: license_kind,
                license_url: license_url_for(license_kind),
                schema_version: SCHEMA_VERSION,
                release_date: published_on,
                authors: doc.author.clone().into_iter().collect(),
                publisher: Some(doc.publisher.name.clone()),
                description: doc.desc.clone(),
                website_url: doc.permalink.clone(),
                is_active: true,
                ordering: modules.len() as i32,
                version_string: "1.0.0".to_string(),
                previous_version_uuid: None,
                published_at: None,
                created_at: epoch,
                updated_at: epoch,
            });
        }
    }
    let included_doc_keys: Vec<String> = included.iter().map(|d| d.key.clone()).collect();
    let in_docs = |key: &str| included_doc_keys.iter().any(|k| k == key);
    // Non-SRD documents whose content imports verbatim (no gap-fill).
    let extra_doc_keys: Vec<String> = included_doc_keys
        .iter()
        .filter(|k| !is_srd(k))
        .cloned()
        .collect();

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
        modules_by_doc: included_doc_keys
            .iter()
            .map(|k| (k.clone(), module_uuid_for_doc(k)))
            .collect(),
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
    // Every family fetches the endpoint once unfiltered and partitions
    // client-side by embedded document key: the upstream document__key
    // filter is broken on several endpoints, and one fetch covers all
    // shipped documents.
    let class_recs_all: Vec<v2::ClassRec> =
        fetcher.fetch_all(&format!("{API}/v2/classes/?limit=200"))?;
    let mut class_groups = group_by_doc(class_recs_all, |r| r.document.key.clone());
    let class_recs_base = class_groups.remove(BASE_DOC).unwrap_or_default();
    let class_recs_legacy = class_groups.remove(LEGACY_DOC).unwrap_or_default();
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

    // Non-SRD documents import verbatim: own parents resolve first,
    // then SRD parents (a sourcebook subclass of an SRD class). The v1
    // sheet-math join is SRD-only. Mapping is tolerant — a record that
    // doesn't fit the schema is reported, not fatal.
    let mut class_keys_by_doc: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for doc_key in &extra_doc_keys {
        let recs = class_groups.remove(doc_key).unwrap_or_default();
        let (parents, subs) = split(recs);
        let doc_parent_keys: BTreeMap<String, String> = parents
            .iter()
            .map(|r| (r.name.to_lowercase(), r.key.clone()))
            .collect();
        for rec in &parents {
            match map::map_class(&ctx, rec, None, overrides, None) {
                Ok(c) => classes.push(c),
                Err(e) => report.skipped.push(format!("class {}: {e}", rec.key)),
            }
        }
        for rec in &subs {
            let parent_stub = rec.subclass_of.as_ref().expect("partitioned as subclass");
            let parent_name = parent_stub.name.to_lowercase();
            let Some(parent_key) = doc_parent_keys
                .get(&parent_name)
                .or_else(|| parent_key_by_name.get(&parent_name))
            else {
                report.skipped.push(format!(
                    "subclass {} (parent {} not shipped)",
                    rec.key, parent_stub.name
                ));
                continue;
            };
            let parent_uuid = content_uuid("class", parent_key);
            match map::map_class(&ctx, rec, None, overrides, Some(parent_uuid)) {
                Ok(c) => classes.push(c),
                Err(e) => report.skipped.push(format!("subclass {}: {e}", rec.key)),
            }
        }
        class_keys_by_doc.insert(doc_key.clone(), doc_parent_keys);
    }

    // ── Spells ──────────────────────────────────────────────────────────
    let spell_recs_all: Vec<v2::SpellRec> =
        fetcher.fetch_all(&format!("{API}/v2/spells/?limit=200"))?;
    let mut spell_groups = group_by_doc(spell_recs_all, |r| r.document.key.clone());
    let spells_base = spell_groups.remove(BASE_DOC).unwrap_or_default();
    let spells_legacy = spell_groups.remove(LEGACY_DOC).unwrap_or_default();
    ensure!(spells_base.len() >= 300, "suspiciously few SRD 5.2 spells");
    ensure!(spells_legacy.len() >= 300, "suspiciously few SRD 5.1 spells");
    let (spell_recs, filled) =
        dedup::gap_fill(spells_base, spells_legacy, |r| r.name.clone(), aliases);
    report.gap_filled.insert("spells", filled);
    let mut spells = spell_recs
        .iter()
        .map(|r| map::map_spell(&ctx, r))
        .collect::<Result<Vec<_>>>()?;
    for doc_key in &extra_doc_keys {
        let recs = spell_groups.remove(doc_key).unwrap_or_default();
        if recs.is_empty() {
            continue;
        }
        // Spell→class references resolve against the spell's own
        // document first, falling back to SRD classes.
        let mut merged = parent_key_by_name.clone();
        if let Some(doc_classes) = class_keys_by_doc.get(doc_key) {
            merged.extend(doc_classes.clone());
        }
        ctx.class_key_by_name = merged;
        for rec in &recs {
            match map::map_spell(&ctx, rec) {
                Ok(s) => spells.push(s),
                Err(e) => report.skipped.push(format!("spell {}: {e}", rec.key)),
            }
        }
    }
    ctx.class_key_by_name = parent_key_by_name.clone();

    // ── Creatures ───────────────────────────────────────────────────────
    let creature_recs_all: Vec<v2::CreatureRec> =
        fetcher.fetch_all(&format!("{API}/v2/creatures/?limit=200"))?;
    let mut creature_groups = group_by_doc(creature_recs_all, |r| r.document.key.clone());
    let creatures_base = creature_groups.remove(BASE_DOC).unwrap_or_default();
    let creatures_legacy = creature_groups.remove(LEGACY_DOC).unwrap_or_default();
    ensure!(creatures_base.len() >= 300, "suspiciously few SRD 5.2 creatures");
    ensure!(creatures_legacy.len() >= 300, "suspiciously few SRD 5.1 creatures");
    let (creature_recs, filled) =
        dedup::gap_fill(creatures_base, creatures_legacy, |r| r.name.clone(), aliases);
    report.gap_filled.insert("creatures", filled);
    let mut creatures = creature_recs
        .iter()
        .map(|r| map::map_creature(&ctx, r))
        .collect::<Result<Vec<_>>>()?;
    for doc_key in &extra_doc_keys {
        for rec in &creature_groups.remove(doc_key).unwrap_or_default() {
            match map::map_creature(&ctx, rec) {
                Ok(c) => creatures.push(c),
                Err(e) => report.skipped.push(format!("creature {}: {e}", rec.key)),
            }
        }
    }

    // ── Species ─────────────────────────────────────────────────────────
    let species_recs_all: Vec<v2::SpeciesRec> =
        fetcher.fetch_all(&format!("{API}/v2/species/?limit=200"))?;
    let species_name_by_key: BTreeMap<String, String> = species_recs_all
        .iter()
        .map(|r| (r.key.clone(), r.name.clone()))
        .collect();
    let mut species_groups = group_by_doc(species_recs_all, |r| r.document.key.clone());
    let species_base = species_groups.remove(BASE_DOC).unwrap_or_default();
    let species_legacy = species_groups.remove(LEGACY_DOC).unwrap_or_default();
    let legacy_species_name_by_key = species_name_by_key;
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
        // The v1 join only describes the 2014 SRD; species from other
        // sourcebooks parse their own typed traits (a name like "Human"
        // must not inherit WotC's 2014 numbers).
        if rec.document.key != LEGACY_DOC {
            return (map::sheet_math_from_v2_traits(&rec.traits, ""), None);
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

    for doc_key in &extra_doc_keys {
        let recs = species_groups.remove(doc_key).unwrap_or_default();
        let (doc_parents, doc_subs): (Vec<_>, Vec<_>) =
            recs.into_iter().partition(|r| !r.is_subspecies);
        let doc_parent_key_by_name: BTreeMap<String, String> = doc_parents
            .iter()
            .map(|r| (r.name.to_lowercase(), r.key.clone()))
            .collect();
        for rec in &doc_parents {
            let (sheet, note) = species_sheet(rec, None);
            report.skipped.extend(note);
            match map::map_species(&ctx, rec, sheet, None) {
                Ok(s) => species.push(s),
                Err(e) => report.skipped.push(format!("species {}: {e}", rec.key)),
            }
        }
        for rec in &doc_subs {
            let parent_name = rec
                .subspecies_of
                .as_ref()
                .and_then(|k| legacy_species_name_by_key.get(k))
                .map(|n| n.to_lowercase());
            let Some(parent_key) = parent_name.as_ref().and_then(|n| {
                doc_parent_key_by_name
                    .get(n)
                    .or_else(|| parent_species_key_by_name.get(n))
            }) else {
                report.skipped.push(format!(
                    "subspecies {} (parent {:?} not shipped)",
                    rec.key, rec.subspecies_of
                ));
                continue;
            };
            let (sheet, note) = species_sheet(rec, parent_name.as_deref());
            report.skipped.extend(note);
            let parent_uuid = content_uuid("species", parent_key);
            match map::map_species(&ctx, rec, sheet, Some(parent_uuid)) {
                Ok(s) => species.push(s),
                Err(e) => report.skipped.push(format!("subspecies {}: {e}", rec.key)),
            }
        }
    }

    // ── Backgrounds & feats ─────────────────────────────────────────────
    let bg_recs_all: Vec<v2::BackgroundRec> =
        fetcher.fetch_all(&format!("{API}/v2/backgrounds/?limit=200"))?;
    let mut bg_groups = group_by_doc(bg_recs_all, |r| r.document.key.clone());
    let bg_base = bg_groups.remove(BASE_DOC).unwrap_or_default();
    let bg_legacy = bg_groups.remove(LEGACY_DOC).unwrap_or_default();
    let (bg_recs, filled) = dedup::gap_fill(bg_base, bg_legacy, |r| r.name.clone(), aliases);
    report.gap_filled.insert("backgrounds", filled);
    let mut backgrounds = bg_recs
        .iter()
        .map(|r| map::map_background(&ctx, r))
        .collect::<Result<Vec<_>>>()?;
    for doc_key in &extra_doc_keys {
        for rec in &bg_groups.remove(doc_key).unwrap_or_default() {
            match map::map_background(&ctx, rec) {
                Ok(b) => backgrounds.push(b),
                Err(e) => report.skipped.push(format!("background {}: {e}", rec.key)),
            }
        }
    }

    let feat_recs_all: Vec<v2::FeatRec> =
        fetcher.fetch_all(&format!("{API}/v2/feats/?limit=200"))?;
    let mut feat_groups = group_by_doc(feat_recs_all, |r| r.document.key.clone());
    let feat_base = feat_groups.remove(BASE_DOC).unwrap_or_default();
    let feat_legacy = feat_groups.remove(LEGACY_DOC).unwrap_or_default();
    let (feat_recs, filled) =
        dedup::gap_fill(feat_base, feat_legacy, |r| r.name.clone(), aliases);
    report.gap_filled.insert("feats", filled);
    let mut feats = feat_recs
        .iter()
        .map(|r| map::map_feat(&ctx, r))
        .collect::<Result<Vec<_>>>()?;
    for doc_key in &extra_doc_keys {
        for rec in &feat_groups.remove(doc_key).unwrap_or_default() {
            match map::map_feat(&ctx, rec) {
                Ok(f) => feats.push(f),
                Err(e) => report.skipped.push(format!("feat {}: {e}", rec.key)),
            }
        }
    }

    // ── Gear: weapons, armor, items ─────────────────────────────────────
    let weapon_recs_all: Vec<v2::WeaponRec> =
        fetcher.fetch_all(&format!("{API}/v2/weapons/?limit=200"))?;
    let mut weapon_groups =
        group_by_doc(weapon_recs_all, |r| r.document.key.clone());
    let w_base = weapon_groups.remove(BASE_DOC).unwrap_or_default();
    let w_legacy = weapon_groups.remove(LEGACY_DOC).unwrap_or_default();
    ensure!(w_base.len() >= 30, "suspiciously few SRD 5.2 weapons");
    let (weapon_recs, filled) =
        dedup::gap_fill(w_base, w_legacy, |r| r.name.clone(), aliases);
    report.gap_filled.insert("weapons", filled);
    let mut weapons = weapon_recs
        .iter()
        .map(|r| map::map_weapon(&ctx, r))
        .collect::<Result<Vec<_>>>()?;
    for doc_key in &extra_doc_keys {
        for rec in &weapon_groups.remove(doc_key).unwrap_or_default() {
            match map::map_weapon(&ctx, rec) {
                Ok(w) => weapons.push(w),
                Err(e) => report.skipped.push(format!("weapon {}: {e}", rec.key)),
            }
        }
    }

    let armor_recs_all: Vec<v2::ArmorRec> =
        fetcher.fetch_all(&format!("{API}/v2/armor/?limit=200"))?;
    let mut armor_groups = group_by_doc(armor_recs_all, |r| r.document.key.clone());
    let a_base = armor_groups.remove(BASE_DOC).unwrap_or_default();
    let a_legacy = armor_groups.remove(LEGACY_DOC).unwrap_or_default();
    ensure!(a_base.len() >= 10, "suspiciously few SRD 5.2 armors");
    let (armor_recs, filled) = dedup::gap_fill(a_base, a_legacy, |r| r.name.clone(), aliases);
    report.gap_filled.insert("armors", filled);
    let mut armors = armor_recs
        .iter()
        .map(|r| map::map_armor(&ctx, r))
        .collect::<Result<Vec<_>>>()?;
    for doc_key in &extra_doc_keys {
        for rec in &armor_groups.remove(doc_key).unwrap_or_default() {
            match map::map_armor(&ctx, rec) {
                Ok(a) => armors.push(a),
                Err(e) => report.skipped.push(format!("armor {}: {e}", rec.key)),
            }
        }
    }

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
    let mut i_base: TaggedItems = Vec::new();
    let mut i_legacy: TaggedItems = Vec::new();
    let mut extra_items: TaggedItems = Vec::new();
    for (recs, is_magic) in [(item_recs_all, false), (magic_recs_all, true)] {
        for rec in recs {
            match rec.document.key.as_str() {
                BASE_DOC => i_base.push((rec, is_magic)),
                LEGACY_DOC => i_legacy.push((rec, is_magic)),
                key if in_docs(key) => extra_items.push((rec, is_magic)),
                _ => {}
            }
        }
    }
    ensure!(i_base.len() >= 200, "suspiciously few SRD 5.2 items");
    ensure!(i_legacy.len() >= 200, "suspiciously few SRD 5.1 items");
    let (item_recs, filled) =
        dedup::gap_fill(i_base, i_legacy, |(r, _)| r.name.clone(), aliases);
    report.gap_filled.insert("items", filled);

    // Extra-doc items sort by key for deterministic output.
    extra_items.sort_by(|(a, _), (b, _)| a.key.cmp(&b.key));

    let mut items = Vec::new();
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
    for (strict, group) in [(true, &item_recs), (false, &extra_items)] {
        for (rec, is_magic) in group {
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
            match map::map_item(&ctx, rec, *is_magic, weapon_uuid, armor_uuid) {
                Ok(i) => items.push(i),
                Err(e) if !strict => {
                    report.skipped.push(format!("item {}: {e}", rec.key));
                }
                Err(e) => return Err(e),
            }
        }
    }

    // ── Assembly ────────────────────────────────────────────────────────
    let mut bundle = ContentBundle {
        schema: SchemaVersion::current(),
        modules,
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

/// The 'srd' module is special-cased: it merges the SRD 5.2 base with
/// SRD 5.1 gap-fill into one logical WotC source.
fn srd_module(uuid: EntityId, epoch: Timestamp) -> ContentModule {
    ContentModule {
        uuid,
        name: "System Reference Document".to_string(),
        slug: "srd".to_string(),
        license: LicenseKind::CcBy40,
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
    }
}

/// Buckets fetched records by their embedded document key.
fn group_by_doc<T>(recs: Vec<T>, doc_key: impl Fn(&T) -> String) -> BTreeMap<String, Vec<T>> {
    let mut groups: BTreeMap<String, Vec<T>> = BTreeMap::new();
    for rec in recs {
        groups.entry(doc_key(&rec)).or_default().push(rec);
    }
    groups
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
