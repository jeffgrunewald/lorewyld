//! Content-bundle import shared by boot seeding and admin installs.
//!
//! Content tables use the doc-style hybrid layout (identity + indexed
//! filter columns + full record JSON in `data`), so the importer is a
//! thin spec-driven loop rather than per-type column mapping.
//! [`CATEGORIES`] is the single source of truth for the table set:
//! import order, uninstall order (reversed), compendium projections,
//! and record counts all derive from it.

use anyhow::{Context, Result, bail};
use chrono::Utc;
use lorewyld_types::{
    AbilityScoreEntry, Alignment, Armor, Background, Class, Condition, ContentBundle,
    ContentModule, Creature, CreatureType, DamageType, Document, Environment, Feat, Item,
    ItemCategory, Language, License, LicenseKind, MIN_SUPPORTED_SCHEMA_VERSION, ModuleOrigin,
    Publisher, SCHEMA_VERSION, SchemaVersion, Size, Skill, Species, Spell, SpellSchool, Weapon,
    WeaponPropertyDef,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::collections::HashSet;
use uuid::Uuid;

const SRD_BUNDLE_JSON: &str = include_str!("../../content/srd-bundle.json");

/// The SRD module hosts the shared rules vocabulary (licenses,
/// publishers, schools, sizes, conditions, …) every other module
/// references — it can never be disabled or removed. Mirrors the
/// mobile app's `ContentStore.pinnedModuleSlug`.
pub const PINNED_MODULE_SLUG: &str = "srd";

/// Reserved slug of the per-server default homebrew module. Authored
/// content lands here unless reassigned to another `local` module. Like
/// [`PINNED_MODULE_SLUG`] it is created lazily, on the first authoring
/// action, by [`ensure_homebrew_module`].
pub const HOMEBREW_MODULE_SLUG: &str = "homebrew";

/// One content table's shape, as far as the importer and the
/// compendium API need to know it.
pub struct CategorySpec {
    /// Table name; also the public category segment in
    /// `/api/content/{category}`.
    pub table: &'static str,
    /// Indexed filter columns beyond the identity set, with the JSON
    /// pointer each is populated from at import time.
    pub extras: &'static [(&'static str, &'static str)],
    /// Record-JSON fields surfaced in list projections via
    /// `json_extract` (subtitle/filter/sort inputs that aren't indexed
    /// columns). Missing fields extract as NULL, which is harmless.
    pub summary_fields: &'static [&'static str],
    /// Small lookup table: list responses include the full record JSON
    /// since clients want fields like `rank` wholesale.
    pub include_data: bool,
}

/// All content tables in import-dependency order (lookups before
/// referents). Uninstall deletes in reverse order.
pub const CATEGORIES: &[CategorySpec] = &[
    CategorySpec {
        table: "license",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "publisher",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "document",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "ability_score",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "skill",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "alignment",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "damage_type",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "condition",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "language",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "size",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "environment",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "spell_school",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "creature_type",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "item_category",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "weapon_property",
        extras: &[],
        summary_fields: &[],
        include_data: true,
    },
    CategorySpec {
        table: "spell",
        extras: &[
            ("level", "/level"),
            ("school_uuid", "/school"),
            ("concentration", "/concentration"),
            ("ritual", "/ritual"),
        ],
        summary_fields: &["document_uuid", "verbal", "somatic", "material"],
        include_data: false,
    },
    CategorySpec {
        table: "creature",
        extras: &[
            ("challenge_rating", "/challenge_rating"),
            ("creature_type_uuid", "/type"),
            ("size_uuid", "/size"),
        ],
        summary_fields: &["document_uuid"],
        include_data: false,
    },
    CategorySpec {
        table: "class",
        extras: &[("subclass_of", "/subclass_of")],
        summary_fields: &["document_uuid", "hit_dice", "caster_type"],
        include_data: false,
    },
    CategorySpec {
        table: "species",
        extras: &[("is_subspecies", "/is_subspecies")],
        summary_fields: &["document_uuid", "size", "speed", "subspecies_of"],
        include_data: false,
    },
    CategorySpec {
        table: "feat",
        extras: &[],
        summary_fields: &["document_uuid", "has_prerequisite", "prerequisite"],
        include_data: false,
    },
    CategorySpec {
        table: "background",
        extras: &[],
        summary_fields: &["document_uuid"],
        include_data: false,
    },
    CategorySpec {
        table: "weapon",
        extras: &[("is_simple", "/is_simple")],
        summary_fields: &["document_uuid", "damage_dice", "damage_type"],
        include_data: false,
    },
    CategorySpec {
        table: "armor",
        extras: &[("category", "/category")],
        summary_fields: &["document_uuid", "ac_display"],
        include_data: false,
    },
    CategorySpec {
        table: "item",
        extras: &[
            ("category_uuid", "/category_uuid"),
            ("rarity", "/rarity"),
            ("is_magic", "/is_magic"),
        ],
        summary_fields: &["document_uuid", "cost", "requires_attunement"],
        include_data: false,
    },
];

/// The categories surfaced as compendium tiles, in display order
/// (mirrors the mobile compendium).
pub const DISPLAY_CATEGORIES: &[&str] = &[
    "spell",
    "creature",
    "class",
    "species",
    "background",
    "feat",
    "item",
    "weapon",
    "armor",
    "condition",
    "language",
];

pub fn category_spec(table: &str) -> Option<&'static CategorySpec> {
    CATEGORIES.iter().find(|spec| spec.table == table)
}

/// Content tables that carry a materialized `summary` column, served
/// verbatim by the list endpoint. The summary shape is single-sourced by
/// each type's `summary()` (e.g. `Spell::summary` → `SpellSummary`).
/// Inc 3b extends this to the remaining display categories.
pub const SUMMARY_TABLES: &[&str] = &[
    "spell",
    "creature",
    "class",
    "species",
    "feat",
    "background",
    "weapon",
    "armor",
    "item",
];

pub fn has_summary(table: &str) -> bool {
    SUMMARY_TABLES.contains(&table)
}

/// How an import reacts to a bundle module whose slug already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlugConflict {
    /// Skip the module and its records (boot seeding: already-present
    /// modules were seeded by an earlier boot).
    Skip,
    /// Fail the whole import (admin installs: a silent partial install
    /// of a multi-module package would be confusing).
    Reject,
}

pub struct ImportOptions {
    pub origin: ModuleOrigin,
    pub on_slug_conflict: SlugConflict,
    /// Require every module to carry a license supported for bundling.
    /// Boot seeding enforces this; admin installs may carry
    /// `unlicensed` homebrew.
    pub require_bundling_license: bool,
}

#[derive(Debug)]
pub enum ImportError {
    UnsupportedSchema { version: u32 },
    UnsupportedLicense { slug: String },
    SlugConflict { slugs: Vec<String> },
    Db(anyhow::Error),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchema { version } => write!(
                f,
                "bundle schema v{version} is outside the supported window \
                 {MIN_SUPPORTED_SCHEMA_VERSION}..={SCHEMA_VERSION}"
            ),
            Self::UnsupportedLicense { slug } => {
                write!(
                    f,
                    "module '{slug}' carries a license unsupported for bundling"
                )
            }
            Self::SlugConflict { slugs } => {
                write!(f, "module slug(s) already installed: {}", slugs.join(", "))
            }
            Self::Db(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ImportError {}

impl From<sqlx::Error> for ImportError {
    fn from(value: sqlx::Error) -> Self {
        Self::Db(value.into())
    }
}

#[derive(Debug)]
pub struct ImportOutcome {
    pub installed: Vec<ContentModule>,
    pub skipped_slugs: Vec<String>,
    pub record_count: u64,
}

pub fn module_origin_to_str(origin: ModuleOrigin) -> &'static str {
    match origin {
        ModuleOrigin::Bundled => "bundled",
        ModuleOrigin::Uploaded => "uploaded",
        ModuleOrigin::Published => "published",
        ModuleOrigin::Local => "local",
    }
}

pub fn module_origin_from_str(s: &str) -> Option<ModuleOrigin> {
    match s {
        "bundled" => Some(ModuleOrigin::Bundled),
        "uploaded" => Some(ModuleOrigin::Uploaded),
        "published" => Some(ModuleOrigin::Published),
        "local" => Some(ModuleOrigin::Local),
        _ => None,
    }
}

/// Imports the shipped content bundle. Modules already present (by
/// slug) are skipped along with their records, so a server upgraded to
/// a bundle with additional source modules seeds only what's new. Runs
/// every boot; it never touches `is_active`, so an admin-disabled
/// bundled module stays disabled across restarts.
pub async fn seed_srd_content(db: &SqlitePool) -> Result<()> {
    let bundle: ContentBundle =
        serde_json::from_str(SRD_BUNDLE_JSON).context("decoding embedded content bundle")?;

    let outcome = import_bundle(
        db,
        &bundle,
        &ImportOptions {
            origin: ModuleOrigin::Bundled,
            on_slug_conflict: SlugConflict::Skip,
            require_bundling_license: true,
        },
    )
    .await
    .context("seeding embedded content bundle")?;

    // Idempotent provenance stamp: marks embedded-bundle modules
    // 'bundled' even when they were seeded before the origin column
    // existed (the migration can't know the embedded slugs).
    let placeholders = vec!["?"; bundle.modules.len()].join(", ");
    let sql = format!(
        "UPDATE content_module SET origin = 'bundled' \
         WHERE slug IN ({placeholders}) AND origin != 'bundled'"
    );
    let mut query = sqlx::query(&sql);
    for module in &bundle.modules {
        query = query.bind(&module.slug);
    }
    query.execute(db).await?;

    if !outcome.installed.is_empty() {
        tracing::info!(
            modules = outcome.installed.len(),
            records = outcome.record_count,
            "seeded content bundle"
        );
    }
    Ok(())
}

/// Imports a `ContentBundle` into the content tables in one
/// transaction, returning what was installed.
pub async fn import_bundle(
    db: &SqlitePool,
    bundle: &ContentBundle,
    opts: &ImportOptions,
) -> Result<ImportOutcome, ImportError> {
    if bundle.schema.version < MIN_SUPPORTED_SCHEMA_VERSION
        || bundle.schema.min_supported > SCHEMA_VERSION
    {
        return Err(ImportError::UnsupportedSchema {
            version: bundle.schema.version,
        });
    }
    if opts.require_bundling_license
        && let Some(module) = bundle
            .modules
            .iter()
            .find(|m| !m.license.is_supported_for_bundling())
    {
        return Err(ImportError::UnsupportedLicense {
            slug: module.slug.clone(),
        });
    }

    let existing_slugs: Vec<String> = sqlx::query_scalar("SELECT slug FROM content_module")
        .fetch_all(db)
        .await?;
    let (present, missing): (Vec<&ContentModule>, Vec<&ContentModule>) = bundle
        .modules
        .iter()
        .partition(|m| existing_slugs.contains(&m.slug));
    let skipped_slugs: Vec<String> = present.iter().map(|m| m.slug.clone()).collect();
    if !skipped_slugs.is_empty() && opts.on_slug_conflict == SlugConflict::Reject {
        return Err(ImportError::SlugConflict {
            slugs: skipped_slugs,
        });
    }
    if missing.is_empty() {
        return Ok(ImportOutcome {
            installed: Vec::new(),
            skipped_slugs,
            record_count: 0,
        });
    }
    // Records ride along only when their module is being installed.
    let allowed: std::collections::HashSet<String> =
        missing.iter().map(|m| m.uuid.to_string()).collect();

    let mut tx = db.begin().await.map_err(ImportError::from)?;
    for module in &missing {
        insert_module(&mut tx, module, opts.origin).await?;
    }
    let record_count = insert_bundle_records(&mut tx, &allowed, bundle)
        .await
        .map_err(ImportError::Db)?;
    tx.commit().await.map_err(ImportError::from)?;

    Ok(ImportOutcome {
        installed: missing.into_iter().cloned().collect(),
        skipped_slugs,
        record_count,
    })
}

/// Deletes a module row plus all its content records and module-scope
/// lore notes in one transaction. Callers gate on origin (bundled
/// modules must not be removed — the seeder would re-add them).
///
/// Content children delete before parents since foreign keys are
/// enforced; a cross-module reference into this module's rows surfaces
/// as `sqlx::Error::Database` with a foreign-key violation.
pub async fn remove_module(db: &SqlitePool, module_uuid: &str) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    // Within `class`, drop subclass rows before base rows: a single
    // DELETE can otherwise remove a parent before its child and trip
    // the self-FK check.
    sqlx::query("DELETE FROM class WHERE content_module_uuid = ? AND subclass_of IS NOT NULL")
        .bind(module_uuid)
        .execute(&mut *tx)
        .await?;
    for spec in CATEGORIES.iter().rev() {
        let sql = format!("DELETE FROM {} WHERE content_module_uuid = ?", spec.table);
        sqlx::query(&sql)
            .bind(module_uuid)
            .execute(&mut *tx)
            .await?;
    }

    // Module-scope notes are snapshot copies owned by the module; the
    // setting originals survive. Tag attachments cascade.
    sqlx::query("DELETE FROM lore_note WHERE scope_kind = 'module' AND scope_target_uuid = ?")
        .bind(module_uuid)
        .execute(&mut *tx)
        .await?;
    // Settings that published this module return to "unpublished" so
    // they can re-publish later. Tags merge into the global vocabulary;
    // only the provenance link is dropped. Version chains pointing at
    // the removed module break.
    sqlx::query(
        "UPDATE setting SET published_as_module_uuid = NULL, updated_at = datetime('now') \
         WHERE published_as_module_uuid = ?",
    )
    .bind(module_uuid)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE tag SET introduced_by_module_uuid = NULL WHERE introduced_by_module_uuid = ?",
    )
    .bind(module_uuid)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE content_module SET previous_version_uuid = NULL WHERE previous_version_uuid = ?",
    )
    .bind(module_uuid)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM content_module WHERE uuid = ?")
        .bind(module_uuid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await
}

/// Inserts every bundle table in import-dependency order; returns the
/// number of records inserted.
async fn insert_bundle_records(
    tx: &mut Transaction<'_, Sqlite>,
    allowed: &std::collections::HashSet<String>,
    bundle: &ContentBundle,
) -> Result<u64> {
    let mut n = 0;
    n += insert_records(tx, allowed, "license", &[], &bundle.licenses).await?;
    n += insert_records(tx, allowed, "publisher", &[], &bundle.publishers).await?;
    n += insert_records(tx, allowed, "document", &[], &bundle.documents).await?;
    n += insert_records(tx, allowed, "ability_score", &[], &bundle.ability_scores).await?;
    n += insert_records(tx, allowed, "skill", &[], &bundle.skills).await?;
    n += insert_records(tx, allowed, "alignment", &[], &bundle.alignments).await?;
    n += insert_records(tx, allowed, "damage_type", &[], &bundle.damage_types).await?;
    n += insert_records(tx, allowed, "condition", &[], &bundle.conditions).await?;
    n += insert_records(tx, allowed, "language", &[], &bundle.languages).await?;
    n += insert_records(tx, allowed, "size", &[], &bundle.sizes).await?;
    n += insert_records(tx, allowed, "environment", &[], &bundle.environments).await?;
    n += insert_records(tx, allowed, "spell_school", &[], &bundle.spell_schools).await?;
    n += insert_records(tx, allowed, "creature_type", &[], &bundle.creature_types).await?;
    n += insert_records(tx, allowed, "item_category", &[], &bundle.item_categories).await?;
    n += insert_records(
        tx,
        allowed,
        "weapon_property",
        &[],
        &bundle.weapon_properties,
    )
    .await?;
    n += insert_records_summarized(
        tx,
        allowed,
        "spell",
        spec_extras("spell"),
        &bundle.spells,
        |s: &Spell| s.summary(),
    )
    .await?;
    n += insert_records_summarized(
        tx,
        allowed,
        "creature",
        spec_extras("creature"),
        &bundle.creatures,
        |c: &Creature| c.summary(),
    )
    .await?;
    // Self-referential tables insert parents before children: bundle
    // order is key-sorted, and a subclass key can sort before its parent.
    let (base_classes, subclasses): (Vec<_>, Vec<_>) = bundle
        .classes
        .iter()
        .cloned()
        .partition(|c| c.subclass_of.is_none());
    n += insert_records_summarized(
        tx,
        allowed,
        "class",
        spec_extras("class"),
        &base_classes,
        |c: &Class| c.summary(),
    )
    .await?;
    n += insert_records_summarized(
        tx,
        allowed,
        "class",
        spec_extras("class"),
        &subclasses,
        |c: &Class| c.summary(),
    )
    .await?;
    let (base_species, subspecies): (Vec<_>, Vec<_>) = bundle
        .species
        .iter()
        .cloned()
        .partition(|s| s.subspecies_of.is_none());
    n += insert_records_summarized(
        tx,
        allowed,
        "species",
        spec_extras("species"),
        &base_species,
        |s: &Species| s.summary(),
    )
    .await?;
    n += insert_records_summarized(
        tx,
        allowed,
        "species",
        spec_extras("species"),
        &subspecies,
        |s: &Species| s.summary(),
    )
    .await?;
    n += insert_records_summarized(tx, allowed, "feat", &[], &bundle.feats, |f: &Feat| {
        f.summary()
    })
    .await?;
    n += insert_records_summarized(
        tx,
        allowed,
        "background",
        &[],
        &bundle.backgrounds,
        |b: &Background| b.summary(),
    )
    .await?;
    n += insert_records_summarized(
        tx,
        allowed,
        "weapon",
        spec_extras("weapon"),
        &bundle.weapons,
        |w: &Weapon| w.summary(),
    )
    .await?;
    n += insert_records_summarized(
        tx,
        allowed,
        "armor",
        spec_extras("armor"),
        &bundle.armors,
        |a: &Armor| a.summary(),
    )
    .await?;
    n += insert_records_summarized(
        tx,
        allowed,
        "item",
        spec_extras("item"),
        &bundle.items,
        |i: &Item| i.summary(),
    )
    .await?;
    Ok(n)
}

fn spec_extras(table: &str) -> &'static [(&'static str, &'static str)] {
    category_spec(table)
        .map(|spec| spec.extras)
        .unwrap_or_default()
}

async fn insert_module(
    tx: &mut Transaction<'_, Sqlite>,
    module: &ContentModule,
    origin: ModuleOrigin,
) -> Result<(), ImportError> {
    sqlx::query(
        "INSERT INTO content_module (uuid, name, slug, license, license_url, schema_version, \
         release_date, authors, publisher, description, website_url, is_active, ordering, \
         version_string, previous_version_uuid, published_at, origin, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(module.uuid.to_string())
    .bind(&module.name)
    .bind(&module.slug)
    .bind(module.license.wire_value())
    .bind(&module.license_url)
    .bind(module.schema_version)
    .bind(module.release_date.map(|d| d.to_string()))
    .bind(serde_json::to_string(&module.authors).map_err(|e| ImportError::Db(e.into()))?)
    .bind(&module.publisher)
    .bind(&module.description)
    .bind(&module.website_url)
    .bind(module.is_active)
    .bind(module.ordering)
    .bind(&module.version_string)
    .bind(module.previous_version_uuid.map(|u| u.to_string()))
    .bind(module.published_at.map(|t| t.to_rfc3339()))
    .bind(module_origin_to_str(origin))
    .bind(module.created_at.to_rfc3339())
    .bind(module.updated_at.to_rfc3339())
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Inserts one bundle table. `extras` maps additional indexed columns to
/// JSON pointers into the serialized record; the full record lands in
/// `data` verbatim. Returns the number of rows inserted.
async fn insert_records<T: Serialize>(
    tx: &mut Transaction<'_, Sqlite>,
    allowed_modules: &std::collections::HashSet<String>,
    table: &str,
    extras: &[(&str, &str)],
    records: &[T],
) -> Result<u64> {
    if records.is_empty() {
        return Ok(0);
    }
    let extra_cols = extras
        .iter()
        .map(|(col, _)| format!(", {col}"))
        .collect::<String>();
    let placeholders = ", ?".repeat(extras.len());
    let sql = format!(
        "INSERT INTO {table} (uuid, content_module_uuid, key, slug, name{extra_cols}, data) \
         VALUES (?, ?, ?, ?, ?{placeholders}, ?)"
    );

    let mut inserted = 0;
    for record in records {
        let value = serde_json::to_value(record)?;
        // Records belonging to an already-installed module are skipped.
        if !value
            .pointer("/content_module_uuid")
            .and_then(Value::as_str)
            .is_some_and(|uuid| allowed_modules.contains(uuid))
        {
            continue;
        }
        let data = serde_json::to_string(&value)?;
        let field = |ptr: &str| -> Result<&Value> {
            value
                .pointer(ptr)
                .ok_or_else(|| anyhow::anyhow!("{table} record missing field {ptr}"))
        };
        let text = |ptr: &str| -> Result<String> {
            Ok(field(ptr)?
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("{table} field {ptr} is not a string"))?
                .to_string())
        };

        let mut query = sqlx::query(&sql)
            .bind(text("/uuid")?)
            .bind(text("/content_module_uuid")?)
            .bind(text("/key")?)
            .bind(text("/slug")?)
            .bind(text("/name")?);
        for (col, ptr) in extras {
            query = match value.pointer(ptr) {
                None | Some(Value::Null) => query.bind(None::<String>),
                Some(Value::String(s)) => query.bind(s.clone()),
                Some(Value::Bool(b)) => query.bind(*b),
                Some(Value::Number(n)) if n.is_i64() || n.is_u64() => {
                    query.bind(n.as_i64().unwrap_or_default())
                }
                Some(Value::Number(n)) => query.bind(n.as_f64().unwrap_or_default()),
                Some(other) => bail!("{table} column {col}: unbindable JSON value {other}"),
            };
        }
        query.bind(data).execute(&mut **tx).await?;
        inserted += 1;
    }
    Ok(inserted)
}

/// Like [`insert_records`] but also writes a materialized `summary` column
/// (for tables in [`SUMMARY_TABLES`]). `summary_of` derives the list-row
/// summary from the typed record, so its shape is single-sourced in
/// lorewyld-types rather than assembled in SQL.
async fn insert_records_summarized<T, S, F>(
    tx: &mut Transaction<'_, Sqlite>,
    allowed_modules: &std::collections::HashSet<String>,
    table: &str,
    extras: &[(&str, &str)],
    records: &[T],
    summary_of: F,
) -> Result<u64>
where
    T: Serialize,
    S: Serialize,
    F: Fn(&T) -> S,
{
    if records.is_empty() {
        return Ok(0);
    }
    let extra_cols = extras
        .iter()
        .map(|(col, _)| format!(", {col}"))
        .collect::<String>();
    let placeholders = ", ?".repeat(extras.len());
    let sql = format!(
        "INSERT INTO {table} (uuid, content_module_uuid, key, slug, name{extra_cols}, summary, data) \
         VALUES (?, ?, ?, ?, ?{placeholders}, ?, ?)"
    );

    let mut inserted = 0;
    for record in records {
        let value = serde_json::to_value(record)?;
        if !value
            .pointer("/content_module_uuid")
            .and_then(Value::as_str)
            .is_some_and(|uuid| allowed_modules.contains(uuid))
        {
            continue;
        }
        let data = serde_json::to_string(&value)?;
        let summary = serde_json::to_string(&summary_of(record))?;
        let field = |ptr: &str| -> Result<&Value> {
            value
                .pointer(ptr)
                .ok_or_else(|| anyhow::anyhow!("{table} record missing field {ptr}"))
        };
        let text = |ptr: &str| -> Result<String> {
            Ok(field(ptr)?
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("{table} field {ptr} is not a string"))?
                .to_string())
        };

        let mut query = sqlx::query(&sql)
            .bind(text("/uuid")?)
            .bind(text("/content_module_uuid")?)
            .bind(text("/key")?)
            .bind(text("/slug")?)
            .bind(text("/name")?);
        for (col, ptr) in extras {
            query = match value.pointer(ptr) {
                None | Some(Value::Null) => query.bind(None::<String>),
                Some(Value::String(s)) => query.bind(s.clone()),
                Some(Value::Bool(b)) => query.bind(*b),
                Some(Value::Number(n)) if n.is_i64() || n.is_u64() => {
                    query.bind(n.as_i64().unwrap_or_default())
                }
                Some(Value::Number(n)) => query.bind(n.as_f64().unwrap_or_default()),
                Some(other) => bail!("{table} column {col}: unbindable JSON value {other}"),
            };
        }
        query.bind(summary).bind(data).execute(&mut **tx).await?;
        inserted += 1;
    }
    Ok(inserted)
}

// ─── Homebrew authoring ─────────────────────────────────────────────────

/// Outcome of persisting one user-authored content record.
#[derive(Debug)]
pub enum RecordError {
    /// The record failed structural validation (didn't deserialize into
    /// its typed struct) or named an unknown category — a 4xx.
    Invalid(String),
    /// An unexpected database failure — a 5xx.
    Db(anyhow::Error),
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(m) => write!(f, "{m}"),
            Self::Db(e) => write!(f, "{e}"),
        }
    }
}

fn db_err(e: impl Into<anyhow::Error>) -> RecordError {
    RecordError::Db(e.into())
}

/// The reserved-slug default homebrew module, plus its attribution
/// document, both created on first use.
pub struct HomebrewModule {
    pub module_uuid: String,
    pub document_uuid: String,
}

/// Returns the per-server homebrew module, creating it (and its
/// attribution `Document`) if it doesn't exist yet. Mirrors how the SRD
/// module is the pinned shared-vocabulary module: `homebrew` is the
/// pinned default authoring target.
pub async fn ensure_homebrew_module(db: &SqlitePool) -> Result<HomebrewModule> {
    if let Some((module_uuid,)) =
        sqlx::query_as::<_, (String,)>("SELECT uuid FROM content_module WHERE slug = ?")
            .bind(HOMEBREW_MODULE_SLUG)
            .fetch_optional(db)
            .await?
    {
        let document_uuid = ensure_module_document(db, &module_uuid, "Homebrew").await?;
        return Ok(HomebrewModule {
            module_uuid,
            document_uuid,
        });
    }

    let now = Utc::now();
    let module = ContentModule {
        uuid: Uuid::new_v4(),
        name: "Homebrew".to_string(),
        slug: HOMEBREW_MODULE_SLUG.to_string(),
        license: LicenseKind::Unlicensed,
        license_url: None,
        schema_version: SCHEMA_VERSION,
        release_date: None,
        authors: Vec::new(),
        publisher: None,
        description: Some("Content you author on this server.".to_string()),
        website_url: None,
        is_active: true,
        // Sorts after the bundled modules in management lists.
        ordering: 1000,
        version_string: "1.0.0".to_string(),
        previous_version_uuid: None,
        published_at: None,
        created_at: now,
        updated_at: now,
    };
    let mut tx = db.begin().await?;
    insert_module(&mut tx, &module, ModuleOrigin::Local)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    tx.commit().await?;

    let module_uuid = module.uuid.to_string();
    let document_uuid = ensure_module_document(db, &module_uuid, "Homebrew").await?;
    Ok(HomebrewModule {
        module_uuid,
        document_uuid,
    })
}

/// Returns the (single) attribution document uuid for a local module,
/// creating one if absent. Records authored under the module reference
/// it via `document_uuid` so the compendium renders a source label.
pub async fn ensure_module_document(
    db: &SqlitePool,
    module_uuid: &str,
    module_name: &str,
) -> Result<String> {
    if let Some((uuid,)) = sqlx::query_as::<_, (String,)>(
        "SELECT uuid FROM document WHERE content_module_uuid = ? LIMIT 1",
    )
    .bind(module_uuid)
    .fetch_optional(db)
    .await?
    {
        return Ok(uuid);
    }
    let now = Utc::now();
    let doc_uuid = Uuid::new_v4();
    let module_uuid_parsed = Uuid::parse_str(module_uuid)?;
    let document = Document {
        uuid: doc_uuid,
        content_module_uuid: module_uuid_parsed,
        name: module_name.to_string(),
        slug: slugify(module_name),
        // Document keys are globally UNIQUE; scope to the module.
        key: format!("doc-{module_uuid}"),
        desc: None,
        license_uuid: Uuid::nil(),
        publisher_uuid: Uuid::nil(),
        gamesystem_key: "5e-2014".to_string(),
        permalink: None,
        author: None,
        published_on: None,
        is_restricted: false,
        created_at: now,
        updated_at: now,
    };
    let data = serde_json::to_string(&document)?;
    sqlx::query(
        "INSERT INTO document (uuid, content_module_uuid, key, slug, name, data) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(doc_uuid.to_string())
    .bind(module_uuid)
    .bind(&document.key)
    .bind(&document.slug)
    .bind(&document.name)
    .bind(data)
    .execute(db)
    .await?;
    Ok(doc_uuid.to_string())
}

/// The provenance of a module by uuid, or `None` if no such module.
pub async fn module_origin(db: &SqlitePool, module_uuid: &str) -> Result<Option<ModuleOrigin>> {
    let row: Option<(String,)> = sqlx::query_as("SELECT origin FROM content_module WHERE uuid = ?")
        .bind(module_uuid)
        .fetch_optional(db)
        .await?;
    Ok(row.and_then(|(o,)| module_origin_from_str(&o)))
}

/// Fields needed to author a new custom (`local`) content module.
pub struct NewCustomModule {
    pub name: String,
    pub slug: String,
    pub license: LicenseKind,
    pub license_url: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub website_url: Option<String>,
    /// The creating user's uuid (gates later edit/delete).
    pub created_by: String,
}

/// Creates a user-authored `local` module (plus its attribution document)
/// that homebrew content can be assigned to. The slug is normalized and
/// must be unique.
pub async fn create_custom_module(
    db: &SqlitePool,
    req: NewCustomModule,
) -> Result<ContentModule, RecordError> {
    let slug = slugify(&req.slug);
    if slug.is_empty() {
        return Err(RecordError::Invalid("module slug cannot be empty".into()));
    }
    let clash: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM content_module WHERE slug = ?")
        .bind(&slug)
        .fetch_optional(db)
        .await
        .map_err(db_err)?;
    if clash.is_some() {
        return Err(RecordError::Invalid(format!(
            "a module with slug '{slug}' already exists"
        )));
    }

    let now = Utc::now();
    let module = ContentModule {
        uuid: Uuid::new_v4(),
        name: req.name,
        slug,
        license: req.license,
        license_url: req.license_url,
        schema_version: SCHEMA_VERSION,
        release_date: None,
        authors: req.authors,
        publisher: None,
        description: req.description,
        website_url: req.website_url,
        is_active: true,
        ordering: 1000,
        version_string: "1.0.0".to_string(),
        previous_version_uuid: None,
        published_at: None,
        created_at: now,
        updated_at: now,
    };
    let mut tx = db.begin().await.map_err(db_err)?;
    insert_module(&mut tx, &module, ModuleOrigin::Local)
        .await
        .map_err(|e| RecordError::Db(anyhow::anyhow!("{e}")))?;
    tx.commit().await.map_err(db_err)?;

    let module_uuid = module.uuid.to_string();
    sqlx::query("UPDATE content_module SET created_by_user_uuid = ? WHERE uuid = ?")
        .bind(&req.created_by)
        .bind(&module_uuid)
        .execute(db)
        .await
        .map_err(db_err)?;
    ensure_module_document(db, &module_uuid, &module.name)
        .await
        .map_err(db_err)?;
    Ok(module)
}

/// Loads every record of one table belonging to a module, decoded from the
/// stored `data` JSON into its typed form.
async fn load_module_table<T: DeserializeOwned>(
    db: &SqlitePool,
    table: &str,
    module_uuid: &str,
) -> Result<Vec<T>> {
    let rows: Vec<(String,)> = sqlx::query_as(&format!(
        "SELECT data FROM {table} WHERE content_module_uuid = ?"
    ))
    .bind(module_uuid)
    .fetch_all(db)
    .await?;
    rows.into_iter()
        .map(|(data,)| serde_json::from_str::<T>(&data).map_err(Into::into))
        .collect()
}

/// Assembles a self-contained [`ContentBundle`] for one module: the module
/// row plus every record it owns, decoded from storage. The inverse of
/// [`import_bundle`] — the same artifact re-imports on another instance or
/// re-seeds this one. Records load by table so the result is dependency-
/// ordered for re-import.
pub async fn export_module(db: &SqlitePool, module: ContentModule) -> Result<ContentBundle> {
    let u = module.uuid.to_string();
    Ok(ContentBundle {
        schema: SchemaVersion {
            version: SCHEMA_VERSION,
            min_supported: MIN_SUPPORTED_SCHEMA_VERSION,
        },
        modules: vec![module],
        licenses: load_module_table::<License>(db, "license", &u).await?,
        publishers: load_module_table::<Publisher>(db, "publisher", &u).await?,
        documents: load_module_table::<Document>(db, "document", &u).await?,
        ability_scores: load_module_table::<AbilityScoreEntry>(db, "ability_score", &u).await?,
        skills: load_module_table::<Skill>(db, "skill", &u).await?,
        alignments: load_module_table::<Alignment>(db, "alignment", &u).await?,
        damage_types: load_module_table::<DamageType>(db, "damage_type", &u).await?,
        conditions: load_module_table::<Condition>(db, "condition", &u).await?,
        languages: load_module_table::<Language>(db, "language", &u).await?,
        sizes: load_module_table::<Size>(db, "size", &u).await?,
        environments: load_module_table::<Environment>(db, "environment", &u).await?,
        spell_schools: load_module_table::<SpellSchool>(db, "spell_school", &u).await?,
        creature_types: load_module_table::<CreatureType>(db, "creature_type", &u).await?,
        item_categories: load_module_table::<ItemCategory>(db, "item_category", &u).await?,
        weapon_properties: load_module_table::<WeaponPropertyDef>(db, "weapon_property", &u)
            .await?,
        spells: load_module_table::<Spell>(db, "spell", &u).await?,
        creatures: load_module_table::<Creature>(db, "creature", &u).await?,
        classes: load_module_table::<Class>(db, "class", &u).await?,
        species: load_module_table::<Species>(db, "species", &u).await?,
        feats: load_module_table::<Feat>(db, "feat", &u).await?,
        backgrounds: load_module_table::<Background>(db, "background", &u).await?,
        weapons: load_module_table::<Weapon>(db, "weapon", &u).await?,
        armors: load_module_table::<Armor>(db, "armor", &u).await?,
        items: load_module_table::<Item>(db, "item", &u).await?,
    })
}

/// Inserts (or replaces, for the edit path) one user-authored content
/// record. Reuses the bundle insert path so the materialized `summary`
/// column and indexed filter columns stay single-sourced. The record's
/// `content_module_uuid` must already name a `local` module — callers
/// enforce the editable-module rule. `created_by_user_uuid` is set
/// separately by the caller, since it is a column rather than a record
/// field.
pub async fn upsert_content_record(
    db: &SqlitePool,
    category: &str,
    value: Value,
) -> Result<(), RecordError> {
    if category_spec(category).is_none() {
        return Err(RecordError::Invalid(format!("unknown category {category}")));
    }
    let uuid = json_str(&value, "uuid")?;
    let module_uuid = json_str(&value, "content_module_uuid")?;
    let allowed: HashSet<String> = std::iter::once(module_uuid).collect();

    let mut tx = db.begin().await.map_err(db_err)?;
    // Replace any prior row (the edit path rebuilds the whole record).
    sqlx::query(&format!("DELETE FROM {category} WHERE uuid = ?"))
        .bind(&uuid)
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;
    insert_authored_record(&mut tx, category, &allowed, value).await?;
    tx.commit().await.map_err(db_err)?;
    Ok(())
}

/// Deletes one content record. Returns whether a row was removed.
/// Callers gate on authorship + the editable-module rule.
pub async fn delete_content_record(
    db: &SqlitePool,
    category: &str,
    uuid: &str,
) -> Result<bool, RecordError> {
    if category_spec(category).is_none() {
        return Err(RecordError::Invalid(format!("unknown category {category}")));
    }
    let res = sqlx::query(&format!("DELETE FROM {category} WHERE uuid = ?"))
        .bind(uuid)
        .execute(db)
        .await
        .map_err(db_err)?;
    Ok(res.rows_affected() > 0)
}

/// Deserializes the record into its typed struct (the authoritative
/// structural guard) and inserts it via the shared bundle path.
async fn insert_authored_record(
    tx: &mut Transaction<'_, Sqlite>,
    category: &str,
    allowed: &HashSet<String>,
    value: Value,
) -> Result<(), RecordError> {
    /// Deserialize, then insert the single record with its materialized
    /// summary, mapping a serde failure to a 4xx.
    macro_rules! summarized {
        ($ty:ty) => {{
            let record: $ty = serde_json::from_value(value)
                .map_err(|e| RecordError::Invalid(format!("invalid {category}: {e}")))?;
            insert_records_summarized(
                tx,
                allowed,
                category,
                spec_extras(category),
                &[record],
                |r: &$ty| r.summary(),
            )
            .await
            .map_err(RecordError::Db)?;
        }};
    }
    macro_rules! lookup {
        ($ty:ty) => {{
            let record: $ty = serde_json::from_value(value)
                .map_err(|e| RecordError::Invalid(format!("invalid {category}: {e}")))?;
            insert_records(tx, allowed, category, &[], &[record])
                .await
                .map_err(RecordError::Db)?;
        }};
    }
    match category {
        "spell" => summarized!(Spell),
        "creature" => summarized!(Creature),
        "class" => summarized!(Class),
        "species" => summarized!(Species),
        "background" => summarized!(Background),
        "feat" => summarized!(Feat),
        "item" => summarized!(Item),
        "weapon" => summarized!(Weapon),
        "armor" => summarized!(Armor),
        "condition" => lookup!(Condition),
        "language" => lookup!(Language),
        other => {
            return Err(RecordError::Invalid(format!(
                "category {other} is not authorable"
            )));
        }
    }
    Ok(())
}

fn json_str(value: &Value, ptr: &str) -> Result<String, RecordError> {
    value
        .get(ptr)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| RecordError::Invalid(format!("record missing string field {ptr}")))
}

/// Lowercase, hyphen-joined slug of a display name (homebrew records and
/// modules need a slug but users only supply a name).
pub fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut prev_dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }
    slug.trim_end_matches('-').to_string()
}

#[cfg(test)]
mod compendium_contract {
    //! Drift guard for surface #2: the web (server/src/web/compendium.rs)
    //! and mobile (mobile/lib/compendium/) compendia read content records
    //! by stringly-typed field key. Those keys ARE the serde field names of
    //! the Rust content types. The clients are plain JS / dynamic Dart, so
    //! a Rust field rename can't be caught at compile time there — it would
    //! silently blank a fact row. This test pins every field the clients
    //! read to the live types: rename/remove one and the test fails loudly,
    //! pointing at the consumers to update in lockstep.
    //!
    //! Field presence is checked against the shipped bundle (real records),
    //! unioned across every record so `skip_serializing_if = None` optionals
    //! that are populated somewhere still count.

    use super::*;
    use std::collections::BTreeSet;

    fn keys_union<T: Serialize>(records: &[T]) -> BTreeSet<String> {
        let mut keys = BTreeSet::new();
        for record in records {
            if let Ok(Value::Object(map)) = serde_json::to_value(record) {
                keys.extend(map.into_iter().map(|(k, _)| k));
            }
        }
        keys
    }

    #[track_caller]
    fn assert_fields(category: &str, present: &BTreeSet<String>, consumed: &[&str]) {
        let missing: Vec<&str> = consumed
            .iter()
            .copied()
            .filter(|f| !present.contains(*f))
            .collect();
        assert!(
            missing.is_empty(),
            "content type `{category}` no longer serializes {missing:?}, which the \
             compendium clients read — update server/src/web/compendium.rs and \
             mobile/lib/compendium/ in lockstep with the type change",
        );
    }

    #[test]
    fn clients_consumed_fields_exist_on_current_types() {
        let bundle: ContentBundle =
            serde_json::from_str(SRD_BUNDLE_JSON).expect("embedded bundle decodes");

        assert_fields(
            "spell",
            &keys_union(&bundle.spells),
            &[
                "uuid",
                "key",
                "slug",
                "name",
                "level",
                "school",
                "concentration",
                "ritual",
                "verbal",
                "somatic",
                "material",
                "material_specified",
                "casting_time",
                "range_text",
                "duration",
                "description",
                "higher_level",
                "document_uuid",
            ],
        );
        assert_fields(
            "creature",
            &keys_union(&bundle.creatures),
            &[
                "uuid",
                "key",
                "slug",
                "name",
                "type",
                "size",
                "challenge_rating",
                "armor_class",
                "armor_detail",
                "hit_points",
                "hit_dice",
                "speed",
                "ability_scores",
                "experience_points",
                "languages",
                "actions",
                "document_uuid",
            ],
        );
        assert_fields(
            "class",
            &keys_union(&bundle.classes),
            &[
                "uuid",
                "key",
                "slug",
                "name",
                "subclass_of",
                "caster_type",
                "hit_dice",
                "prof_saving_throws",
                "prof_armor",
                "prof_weapons",
                "prof_skills",
                "features",
                "desc",
                "document_uuid",
            ],
        );
        assert_fields(
            "species",
            &keys_union(&bundle.species),
            &[
                "uuid",
                "key",
                "slug",
                "name",
                "is_subspecies",
                "subspecies_of",
                "size",
                "speed",
                "asi_desc",
                "traits",
                "desc",
                "document_uuid",
            ],
        );
        assert_fields(
            "feat",
            &keys_union(&bundle.feats),
            &[
                "uuid",
                "key",
                "slug",
                "name",
                "has_prerequisite",
                "prerequisite",
                "benefits",
                "desc",
                "document_uuid",
            ],
        );
        assert_fields(
            "background",
            &keys_union(&bundle.backgrounds),
            &[
                "uuid",
                "key",
                "slug",
                "name",
                "benefits",
                "desc",
                "document_uuid",
            ],
        );
        assert_fields(
            "item",
            &keys_union(&bundle.items),
            &[
                "uuid",
                "key",
                "slug",
                "name",
                "category_uuid",
                "cost",
                "weight",
                "is_magic",
                "rarity",
                "requires_attunement",
                "desc",
                "document_uuid",
            ],
        );
        assert_fields(
            "weapon",
            &keys_union(&bundle.weapons),
            &[
                "uuid",
                "key",
                "slug",
                "name",
                "is_simple",
                "damage_dice",
                "damage_type",
                "properties",
                "document_uuid",
            ],
        );
        assert_fields(
            "armor",
            &keys_union(&bundle.armors),
            &[
                "uuid",
                "key",
                "slug",
                "name",
                "category",
                "ac_display",
                "grants_stealth_disadvantage",
                "document_uuid",
            ],
        );
    }

    #[test]
    fn nested_named_list_sections_keep_name_and_desc() {
        // The entry view renders class.features / species.traits /
        // creature.actions / background.benefits as `{name, desc}` cards
        // (compendium.rs `namedList`). Guard those nested keys too. Feat
        // benefits are excluded: `FeatBenefit.name` is optional by design
        // (unnamed benefits are intentionally skipped by the client).
        let bundle: ContentBundle =
            serde_json::from_str(SRD_BUNDLE_JSON).expect("embedded bundle decodes");

        // Asserts the first non-empty list found under `field` has rows
        // carrying both `name` and `desc`.
        fn assert_name_desc<T: Serialize>(records: &[T], field: &str, label: &str) {
            let row = records
                .iter()
                .filter_map(|r| serde_json::to_value(r).ok())
                .find_map(|r| {
                    r.get(field)
                        .and_then(Value::as_array)
                        .and_then(|a| a.first())
                        .cloned()
                });
            let row = row
                .unwrap_or_else(|| panic!("{label}: no records carry a non-empty `{field}` list"));
            assert!(
                row.get("name").is_some() && row.get("desc").is_some(),
                "{label} rows lost `name`/`desc` — update compendium clients' namedList rendering",
            );
        }

        assert_name_desc(&bundle.classes, "features", "class.features");
        assert_name_desc(&bundle.species, "traits", "species.traits");
        assert_name_desc(&bundle.creatures, "actions", "creature.actions");
        assert_name_desc(&bundle.backgrounds, "benefits", "background.benefits");
    }
}
