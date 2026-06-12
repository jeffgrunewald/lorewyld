//! The embedded bundle seeds into a fresh database, reseeding is a
//! no-op, and a module wiped from the database is restored on the next
//! boot without touching the others (the incremental-upgrade path).

use sqlx::SqlitePool;

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

#[tokio::test]
async fn seeds_multi_module_bundle_idempotently_and_incrementally() {
    let pool = fresh_pool().await;

    lorewyld::content::seed_srd_content(&pool).await.expect("first seed");
    let modules = count(&pool, "SELECT COUNT(*) FROM content_module").await;
    let creatures = count(&pool, "SELECT COUNT(*) FROM creature").await;
    let spells = count(&pool, "SELECT COUNT(*) FROM spell").await;
    assert!(modules > 1, "expected one module per source, got {modules}");
    assert!(creatures > 3000, "expected the full creature set, got {creatures}");
    assert!(spells > 1000, "expected the full spell set, got {spells}");

    // Reseeding an up-to-date database changes nothing.
    lorewyld::content::seed_srd_content(&pool).await.expect("reseed");
    assert_eq!(count(&pool, "SELECT COUNT(*) FROM content_module").await, modules);
    assert_eq!(count(&pool, "SELECT COUNT(*) FROM creature").await, creatures);

    // Simulate an install that predates one module: drop it and its
    // records, then reseed — only the missing module comes back.
    for sql in [
        "DELETE FROM creature WHERE content_module_uuid = \
         (SELECT uuid FROM content_module WHERE slug = 'tob')",
        "DELETE FROM document WHERE content_module_uuid = \
         (SELECT uuid FROM content_module WHERE slug = 'tob')",
        "DELETE FROM content_module WHERE slug = 'tob'",
    ] {
        sqlx::query(sql).execute(&pool).await.expect(sql);
    }
    let creatures_without_tob = count(&pool, "SELECT COUNT(*) FROM creature").await;
    assert!(creatures_without_tob < creatures);

    lorewyld::content::seed_srd_content(&pool).await.expect("incremental seed");
    assert_eq!(count(&pool, "SELECT COUNT(*) FROM content_module").await, modules);
    assert_eq!(count(&pool, "SELECT COUNT(*) FROM creature").await, creatures);
}
