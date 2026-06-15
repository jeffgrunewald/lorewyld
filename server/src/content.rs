//! Content-bundle import shared by boot seeding and admin installs.
//!
//! Content tables use the doc-style hybrid layout (identity + indexed
//! filter columns + full record JSON in `data`), so the importer is a
//! thin spec-driven loop rather than per-type column mapping.
//! [`CATEGORIES`] is the single source of truth for the table set:
//! import order, uninstall order (reversed), compendium projections,
//! and record counts all derive from it.

use anyhow::{Context, Result, bail};
use lorewyld_types::{
    ContentBundle, ContentModule, MIN_SUPPORTED_SCHEMA_VERSION, ModuleOrigin, SCHEMA_VERSION, Spell,
};
use serde::Serialize;
use serde_json::Value;
use sqlx::{Sqlite, SqlitePool, Transaction};

const SRD_BUNDLE_JSON: &str = include_str!("../../content/srd-bundle.json");

/// The SRD module hosts the shared rules vocabulary (licenses,
/// publishers, schools, sizes, conditions, …) every other module
/// references — it can never be disabled or removed. Mirrors the
/// mobile app's `ContentStore.pinnedModuleSlug`.
pub const PINNED_MODULE_SLUG: &str = "srd";

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
pub const SUMMARY_TABLES: &[&str] = &["spell"];

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
    }
}

pub fn module_origin_from_str(s: &str) -> Option<ModuleOrigin> {
    match s {
        "bundled" => Some(ModuleOrigin::Bundled),
        "uploaded" => Some(ModuleOrigin::Uploaded),
        "published" => Some(ModuleOrigin::Published),
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
    n += insert_records(
        tx,
        allowed,
        "creature",
        spec_extras("creature"),
        &bundle.creatures,
    )
    .await?;
    // Self-referential tables insert parents before children: bundle
    // order is key-sorted, and a subclass key can sort before its parent.
    let (base_classes, subclasses): (Vec<_>, Vec<_>) = bundle
        .classes
        .iter()
        .cloned()
        .partition(|c| c.subclass_of.is_none());
    n += insert_records(tx, allowed, "class", spec_extras("class"), &base_classes).await?;
    n += insert_records(tx, allowed, "class", spec_extras("class"), &subclasses).await?;
    let (base_species, subspecies): (Vec<_>, Vec<_>) = bundle
        .species
        .iter()
        .cloned()
        .partition(|s| s.subspecies_of.is_none());
    n += insert_records(
        tx,
        allowed,
        "species",
        spec_extras("species"),
        &base_species,
    )
    .await?;
    n += insert_records(tx, allowed, "species", spec_extras("species"), &subspecies).await?;
    n += insert_records(tx, allowed, "feat", &[], &bundle.feats).await?;
    n += insert_records(tx, allowed, "background", &[], &bundle.backgrounds).await?;
    n += insert_records(
        tx,
        allowed,
        "weapon",
        spec_extras("weapon"),
        &bundle.weapons,
    )
    .await?;
    n += insert_records(tx, allowed, "armor", spec_extras("armor"), &bundle.armors).await?;
    n += insert_records(tx, allowed, "item", spec_extras("item"), &bundle.items).await?;
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

/// Backfills `summary` columns for content rows seeded before the column
/// existed. Idempotent: only NULL-summary rows are touched, so it is a
/// no-op once every row has a summary (new seeds write one at ingest).
pub async fn backfill_summaries(db: &SqlitePool) -> Result<()> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT uuid, data FROM spell WHERE summary IS NULL")
            .fetch_all(db)
            .await?;
    if rows.is_empty() {
        return Ok(());
    }
    let mut tx = db.begin().await?;
    for (uuid, data) in &rows {
        let spell: Spell = serde_json::from_str(data)
            .with_context(|| format!("decoding spell {uuid} for summary backfill"))?;
        let summary = serde_json::to_string(&spell.summary())?;
        sqlx::query("UPDATE spell SET summary = ? WHERE uuid = ?")
            .bind(summary)
            .bind(uuid)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    tracing::info!(spells = rows.len(), "backfilled spell summaries");
    Ok(())
}
