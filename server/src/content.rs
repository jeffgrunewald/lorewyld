//! Seeds the embedded SRD content bundle into SQLite on first boot.
//!
//! Content tables use the doc-style hybrid layout (identity + indexed
//! filter columns + full record JSON in `data`), so the importer is a
//! thin spec-driven loop rather than per-type column mapping.

use anyhow::{Context, Result, bail};
use lorewyld_types::{ContentBundle, ContentModule, MIN_SUPPORTED_SCHEMA_VERSION, SCHEMA_VERSION};
use serde::Serialize;
use serde_json::Value;
use sqlx::{Sqlite, SqlitePool, Transaction};

const SRD_BUNDLE_JSON: &str = include_str!("../../content/srd-bundle.json");

/// Imports the shipped content bundle. Modules already present (by
/// slug) are skipped along with their records, so a server upgraded to
/// a bundle with additional source modules seeds only what's new.
pub async fn seed_srd_content(db: &SqlitePool) -> Result<()> {
    let bundle: ContentBundle =
        serde_json::from_str(SRD_BUNDLE_JSON).context("decoding embedded content bundle")?;
    if bundle.schema.version < MIN_SUPPORTED_SCHEMA_VERSION
        || bundle.schema.min_supported > SCHEMA_VERSION
    {
        bail!(
            "embedded content bundle schema v{} is outside the supported window {}..={}",
            bundle.schema.version,
            MIN_SUPPORTED_SCHEMA_VERSION,
            SCHEMA_VERSION
        );
    }
    // Shipped content must carry a supported license; `unlicensed` is
    // reserved for user-published homebrew.
    for module in &bundle.modules {
        if !module.license.is_supported_for_bundling() {
            bail!(
                "bundled module '{}' carries unsupported license {:?}",
                module.slug,
                module.license
            );
        }
    }

    let existing_slugs: Vec<String> = sqlx::query_scalar("SELECT slug FROM content_module")
        .fetch_all(db)
        .await?;
    let missing: Vec<&ContentModule> = bundle
        .modules
        .iter()
        .filter(|m| !existing_slugs.contains(&m.slug))
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    // Records ride along only when their module is being seeded.
    let allowed: std::collections::HashSet<String> =
        missing.iter().map(|m| m.uuid.to_string()).collect();

    let mut tx = db.begin().await?;
    for module in &missing {
        insert_module(&mut tx, module).await?;
    }

    // Insertion follows the bundle's import-dependency order.
    insert_records(&mut tx, &allowed, "license", &[], &bundle.licenses).await?;
    insert_records(&mut tx, &allowed, "publisher", &[], &bundle.publishers).await?;
    insert_records(&mut tx, &allowed, "document", &[], &bundle.documents).await?;
    insert_records(&mut tx, &allowed, "ability_score", &[], &bundle.ability_scores).await?;
    insert_records(&mut tx, &allowed, "skill", &[], &bundle.skills).await?;
    insert_records(&mut tx, &allowed, "alignment", &[], &bundle.alignments).await?;
    insert_records(&mut tx, &allowed, "damage_type", &[], &bundle.damage_types).await?;
    insert_records(&mut tx, &allowed, "condition", &[], &bundle.conditions).await?;
    insert_records(&mut tx, &allowed, "language", &[], &bundle.languages).await?;
    insert_records(&mut tx, &allowed, "size", &[], &bundle.sizes).await?;
    insert_records(&mut tx, &allowed, "environment", &[], &bundle.environments).await?;
    insert_records(&mut tx, &allowed, "spell_school", &[], &bundle.spell_schools).await?;
    insert_records(&mut tx, &allowed, "creature_type", &[], &bundle.creature_types).await?;
    insert_records(&mut tx, &allowed, "item_category", &[], &bundle.item_categories).await?;
    insert_records(&mut tx, &allowed, "weapon_property", &[], &bundle.weapon_properties).await?;
    insert_records(&mut tx, &allowed,
        "spell",
        &[
            ("level", "/level"),
            ("school_uuid", "/school"),
            ("concentration", "/concentration"),
            ("ritual", "/ritual"),
        ],
        &bundle.spells,
    )
    .await?;
    insert_records(&mut tx, &allowed,
        "creature",
        &[
            ("challenge_rating", "/challenge_rating"),
            ("creature_type_uuid", "/type"),
            ("size_uuid", "/size"),
        ],
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
    insert_records(&mut tx, &allowed, "class", &[("subclass_of", "/subclass_of")], &base_classes).await?;
    insert_records(&mut tx, &allowed, "class", &[("subclass_of", "/subclass_of")], &subclasses).await?;
    let (base_species, subspecies): (Vec<_>, Vec<_>) = bundle
        .species
        .iter()
        .cloned()
        .partition(|s| s.subspecies_of.is_none());
    let species_extras = [("is_subspecies", "/is_subspecies")];
    insert_records(&mut tx, &allowed, "species", &species_extras, &base_species).await?;
    insert_records(&mut tx, &allowed, "species", &species_extras, &subspecies).await?;
    insert_records(&mut tx, &allowed, "feat", &[], &bundle.feats).await?;
    insert_records(&mut tx, &allowed, "background", &[], &bundle.backgrounds).await?;
    insert_records(&mut tx, &allowed, "weapon", &[("is_simple", "/is_simple")], &bundle.weapons).await?;
    insert_records(&mut tx, &allowed, "armor", &[("category", "/category")], &bundle.armors).await?;
    insert_records(&mut tx, &allowed,
        "item",
        &[
            ("category_uuid", "/category_uuid"),
            ("rarity", "/rarity"),
            ("is_magic", "/is_magic"),
        ],
        &bundle.items,
    )
    .await?;
    tx.commit().await?;

    tracing::info!(
        spells = bundle.spells.len(),
        creatures = bundle.creatures.len(),
        classes = bundle.classes.len(),
        species = bundle.species.len(),
        items = bundle.items.len(),
        weapons = bundle.weapons.len(),
        armors = bundle.armors.len(),
        feats = bundle.feats.len(),
        backgrounds = bundle.backgrounds.len(),
        "seeded content bundle"
    );
    Ok(())
}

async fn insert_module(tx: &mut Transaction<'_, Sqlite>, module: &ContentModule) -> Result<()> {
    sqlx::query(
        "INSERT INTO content_module (uuid, name, slug, license, license_url, schema_version, \
         release_date, authors, publisher, description, website_url, is_active, ordering, \
         version_string, previous_version_uuid, published_at, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(module.uuid.to_string())
    .bind(&module.name)
    .bind(&module.slug)
    .bind(module.license.wire_value())
    .bind(&module.license_url)
    .bind(module.schema_version)
    .bind(module.release_date.map(|d| d.to_string()))
    .bind(serde_json::to_string(&module.authors)?)
    .bind(&module.publisher)
    .bind(&module.description)
    .bind(&module.website_url)
    .bind(module.is_active)
    .bind(module.ordering)
    .bind(&module.version_string)
    .bind(module.previous_version_uuid.map(|u| u.to_string()))
    .bind(module.published_at.map(|t| t.to_rfc3339()))
    .bind(module.created_at.to_rfc3339())
    .bind(module.updated_at.to_rfc3339())
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Inserts one bundle table. `extras` maps additional indexed columns to
/// JSON pointers into the serialized record; the full record lands in
/// `data` verbatim.
async fn insert_records<T: Serialize>(
    tx: &mut Transaction<'_, Sqlite>,
    allowed_modules: &std::collections::HashSet<String>,
    table: &str,
    extras: &[(&str, &str)],
    records: &[T],
) -> Result<()> {
    if records.is_empty() {
        return Ok(());
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

    for record in records {
        let value = serde_json::to_value(record)?;
        // Records belonging to an already-seeded module are skipped.
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
    }
    Ok(())
}
