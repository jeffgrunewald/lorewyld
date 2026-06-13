//! Module lifecycle invariants: bundled-origin stamping, disable
//! surviving reseed, uploaded-bundle install/conflict semantics, and
//! full uninstall leaving no trace.

use chrono::Utc;
use lorewyld::content::{
    ImportError, ImportOptions, SlugConflict, import_bundle, remove_module, seed_srd_content,
};
use lorewyld_types::{
    Condition, ConditionName, ContentBundle, ContentModule, LicenseKind, ModuleOrigin,
    SchemaVersion, content_uuid,
};
use sqlx::SqlitePool;
use uuid::Uuid;

async fn fresh_pool() -> SqlitePool {
    // One connection: every connection to `:memory:` is its own
    // database, so a pool of two would see different schemas.
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("opening in-memory sqlite");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("running migrations");
    pool
}

async fn count(pool: &SqlitePool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.expect(sql)
}

/// A minimal one-module bundle carrying two conditions.
fn test_bundle(slug: &str) -> ContentBundle {
    let now = Utc::now();
    let module_uuid = Uuid::new_v4();
    let condition = |name: ConditionName, key: &str| Condition {
        uuid: content_uuid("condition", key),
        content_module_uuid: module_uuid,
        name,
        slug: key.to_string(),
        key: key.to_string(),
        desc: "test condition".to_string(),
        is_restricted: false,
        created_at: now,
        updated_at: now,
    };
    ContentBundle {
        schema: SchemaVersion::current(),
        modules: vec![ContentModule {
            uuid: module_uuid,
            name: format!("Test pack {slug}"),
            slug: slug.to_string(),
            license: LicenseKind::Unlicensed,
            license_url: None,
            schema_version: 2,
            release_date: None,
            authors: vec!["tester@example.com".to_string()],
            publisher: None,
            description: Some("synthetic test bundle".to_string()),
            website_url: None,
            is_active: true,
            ordering: 99,
            version_string: "1.0.0".to_string(),
            previous_version_uuid: None,
            published_at: None,
            created_at: now,
            updated_at: now,
        }],
        conditions: vec![
            condition(ConditionName::Blinded, &format!("{slug}_blinded")),
            condition(ConditionName::Charmed, &format!("{slug}_charmed")),
        ],
        ..ContentBundle::default()
    }
}

const UPLOAD_OPTS: ImportOptions = ImportOptions {
    origin: ModuleOrigin::Uploaded,
    on_slug_conflict: SlugConflict::Reject,
    require_bundling_license: false,
};

#[tokio::test]
async fn seeder_stamps_bundled_origin_and_preserves_disable() {
    let pool = fresh_pool().await;
    seed_srd_content(&pool).await.expect("seed");

    let non_bundled = count(
        &pool,
        "SELECT COUNT(*) FROM content_module WHERE origin != 'bundled'",
    )
    .await;
    assert_eq!(non_bundled, 0, "every seeded module must be origin=bundled");

    // Simulate a pre-origin database row: blank the stamp and reseed.
    sqlx::query("UPDATE content_module SET origin = 'uploaded' WHERE slug = 'srd'")
        .execute(&pool)
        .await
        .expect("unstamp");
    seed_srd_content(&pool).await.expect("reseed");
    let srd_origin: String =
        sqlx::query_scalar("SELECT origin FROM content_module WHERE slug = 'srd'")
            .fetch_one(&pool)
            .await
            .expect("srd origin");
    assert_eq!(srd_origin, "bundled", "stamp must self-heal older rows");

    // Disable a bundled module; reseeding must neither reactivate nor
    // duplicate it.
    let modules = count(&pool, "SELECT COUNT(*) FROM content_module").await;
    sqlx::query("UPDATE content_module SET is_active = 0 WHERE slug = 'srd'")
        .execute(&pool)
        .await
        .expect("disable");
    seed_srd_content(&pool).await.expect("reseed after disable");
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM content_module").await,
        modules
    );
    let still_disabled: i64 =
        sqlx::query_scalar("SELECT is_active FROM content_module WHERE slug = 'srd'")
            .fetch_one(&pool)
            .await
            .expect("is_active");
    assert_eq!(still_disabled, 0, "disable must survive reseed");
}

#[tokio::test]
async fn upload_installs_then_rejects_slug_conflict() {
    let pool = fresh_pool().await;
    seed_srd_content(&pool).await.expect("seed");

    let bundle = test_bundle("testpack");
    let outcome = import_bundle(&pool, &bundle, &UPLOAD_OPTS)
        .await
        .expect("install");
    assert_eq!(outcome.installed.len(), 1);
    assert_eq!(outcome.record_count, 2);

    let origin: String =
        sqlx::query_scalar("SELECT origin FROM content_module WHERE slug = 'testpack'")
            .fetch_one(&pool)
            .await
            .expect("origin");
    assert_eq!(origin, "uploaded");

    // Reinstalling the same slug rejects the whole package.
    let err = import_bundle(&pool, &test_bundle("testpack"), &UPLOAD_OPTS)
        .await
        .expect_err("conflict");
    assert!(
        matches!(&err, ImportError::SlugConflict { slugs } if slugs == &vec!["testpack".to_string()]),
        "expected slug conflict, got {err:?}"
    );
}

#[tokio::test]
async fn uninstall_removes_all_module_rows() {
    let pool = fresh_pool().await;
    seed_srd_content(&pool).await.expect("seed");

    let conditions_before = count(&pool, "SELECT COUNT(*) FROM condition").await;
    let bundle = test_bundle("removeme");
    import_bundle(&pool, &bundle, &UPLOAD_OPTS)
        .await
        .expect("install");
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM condition").await,
        conditions_before + 2
    );

    let module_uuid = bundle.modules[0].uuid.to_string();
    remove_module(&pool, &module_uuid).await.expect("uninstall");

    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM condition").await,
        conditions_before
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM content_module WHERE slug = 'removeme'"
        )
        .await,
        0
    );

    // A fresh install of the same slug works again after uninstall.
    import_bundle(&pool, &test_bundle("removeme"), &UPLOAD_OPTS)
        .await
        .expect("reinstall after uninstall");
}
