//! Homebrew content authoring invariants: records land in the auto-created
//! Homebrew (`local`) module, validation rejects bad input, edit/delete is
//! creator-or-admin, and the editable-module rule blocks authoring into or
//! out of read-only (non-`local`) modules.
//!
//! Uses `feat` as the probe type: it needs no lookup-table foreign keys
//! beyond `content_module`, so a freshly-migrated (unseeded) DB suffices.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use lorewyld::api::ApiState;
use lorewyld::api::auth::CurrentUser;
use lorewyld::api::authoring::{create_content, delete_content, update_content};
use lorewyld::api::error::ApiError;
use lorewyld_types::api_v1::{CreateContentRequest, UpdateContentRequest};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use uuid::Uuid;

async fn fresh_pool() -> SqlitePool {
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

async fn insert_user(pool: &SqlitePool, username: &str, admin: bool) -> CurrentUser {
    let uuid = Uuid::new_v4();
    let email = format!("{username}@example.com");
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, admin) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(uuid.to_string())
    .bind(username)
    .bind(&email)
    .bind("not-a-real-hash")
    .bind(admin)
    .execute(pool)
    .await
    .expect("inserting test user");
    CurrentUser {
        uuid,
        username: username.to_string(),
        email,
        admin,
    }
}

/// Inserts a bare module row with the given slug + origin.
async fn insert_module(pool: &SqlitePool, slug: &str, origin: &str) -> Uuid {
    let uuid = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO content_module (uuid, name, slug, license, origin) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(uuid.to_string())
    .bind(slug)
    .bind(slug)
    .bind("unlicensed")
    .bind(origin)
    .execute(pool)
    .await
    .expect("inserting module");
    uuid
}

fn feat_fields(name: &str) -> Value {
    json!({ "name": name, "desc": "A heroic knack." })
}

async fn create_feat(
    state: &ApiState,
    user: &CurrentUser,
    name: &str,
    module: Option<Uuid>,
) -> Result<Value, ApiError> {
    create_content(
        State(state.clone()),
        user.clone(),
        Path("feat".to_string()),
        Json(CreateContentRequest {
            fields: feat_fields(name),
            module_uuid: module,
        }),
    )
    .await
    .map(|(_, Json(v))| v)
}

#[tokio::test]
async fn create_lands_in_homebrew_and_sets_author() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;

    let created = create_feat(&state, &alice, "Tough", None)
        .await
        .expect("creating feat");
    let uuid = created["uuid"].as_str().unwrap().to_string();

    // The Homebrew module was auto-created as a `local` module.
    let (slug, origin): (String, String) = sqlx::query_as(
        "SELECT m.slug, m.origin FROM content_module m \
         JOIN feat f ON f.content_module_uuid = m.uuid WHERE f.uuid = ?",
    )
    .bind(&uuid)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(slug, "homebrew");
    assert_eq!(origin, "local");

    // Authorship column is stamped; the slug derives from the name.
    let author: Option<String> =
        sqlx::query_scalar("SELECT created_by_user_uuid FROM feat WHERE uuid = ?")
            .bind(&uuid)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(author.as_deref(), Some(alice.uuid.to_string().as_str()));
    assert_eq!(created["slug"], json!("tough"));
}

#[tokio::test]
async fn missing_required_name_is_rejected() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;

    let result = create_content(
        State(state.clone()),
        alice,
        Path("feat".to_string()),
        Json(CreateContentRequest {
            fields: json!({ "desc": "no name" }),
            module_uuid: None,
        }),
    )
    .await;
    assert!(matches!(result, Err(ApiError::Validation(_))));
}

#[tokio::test]
async fn unknown_category_is_not_found() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;

    let result = create_content(
        State(state.clone()),
        alice,
        Path("dragon".to_string()),
        Json(CreateContentRequest {
            fields: json!({ "name": "x" }),
            module_uuid: None,
        }),
    )
    .await;
    assert!(matches!(result, Err(ApiError::NotFound)));
}

#[tokio::test]
async fn edit_is_creator_or_admin_only() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;
    let bob = insert_user(&state.db, "bob", false).await;
    let carol = insert_user(&state.db, "carol", true).await;

    let created = create_feat(&state, &alice, "Tough", None).await.unwrap();
    let uuid: Uuid = created["uuid"].as_str().unwrap().parse().unwrap();

    let patch = |user: CurrentUser| {
        update_content(
            State(state.clone()),
            user,
            Path(("feat".to_string(), uuid)),
            Json(UpdateContentRequest {
                fields: Some(json!({ "name": "Tougher", "desc": "More HP." })),
                module_uuid: None,
            }),
        )
    };

    assert!(matches!(patch(bob).await, Err(ApiError::Forbidden)));

    let Json(by_admin) = patch(carol).await.expect("admin may edit");
    assert_eq!(by_admin["name"], json!("Tougher"));

    let Json(by_owner) = patch(alice).await.expect("creator may edit");
    assert_eq!(by_owner["name"], json!("Tougher"));
}

#[tokio::test]
async fn cannot_author_into_or_out_of_non_local_module() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;
    let uploaded = insert_module(&state.db, "third-party", "uploaded").await;

    // Create directly into a read-only module is refused.
    let create = create_feat(&state, &alice, "Tough", Some(uploaded)).await;
    assert!(matches!(create, Err(ApiError::Forbidden)));

    // Create in homebrew, then try to reassign into the read-only module.
    let created = create_feat(&state, &alice, "Tough", None).await.unwrap();
    let uuid: Uuid = created["uuid"].as_str().unwrap().parse().unwrap();
    let reassign = update_content(
        State(state.clone()),
        alice,
        Path(("feat".to_string(), uuid)),
        Json(UpdateContentRequest {
            fields: None,
            module_uuid: Some(uploaded),
        }),
    )
    .await;
    assert!(matches!(reassign, Err(ApiError::Forbidden)));
}

#[tokio::test]
async fn reassign_between_local_modules() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;
    let custom = insert_module(&state.db, "my-world", "local").await;

    let created = create_feat(&state, &alice, "Tough", None).await.unwrap();
    let uuid: Uuid = created["uuid"].as_str().unwrap().parse().unwrap();

    let Json(moved) = update_content(
        State(state.clone()),
        alice,
        Path(("feat".to_string(), uuid)),
        Json(UpdateContentRequest {
            fields: None,
            module_uuid: Some(custom),
        }),
    )
    .await
    .expect("reassigning to a local module");
    assert_eq!(moved["content_module_uuid"], json!(custom.to_string()));

    let stored_module: String =
        sqlx::query_scalar("SELECT content_module_uuid FROM feat WHERE uuid = ?")
            .bind(uuid.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(stored_module, custom.to_string());
}

#[tokio::test]
async fn forged_uuid_in_fields_cannot_clobber_another_record() {
    // IDOR guard: editing one's own record while sending a different
    // record's uuid in `fields` must NOT delete/overwrite the victim.
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;
    let bob = insert_user(&state.db, "bob", false).await;

    let victim = create_feat(&state, &bob, "Bob's Feat", None).await.unwrap();
    let victim_uuid = victim["uuid"].as_str().unwrap().to_string();

    let attacker = create_feat(&state, &alice, "Alice's Feat", None)
        .await
        .unwrap();
    let attacker_uuid: Uuid = attacker["uuid"].as_str().unwrap().parse().unwrap();

    // Alice edits her own feat but tries to smuggle the victim's uuid.
    let Json(_) = update_content(
        State(state.clone()),
        alice,
        Path(("feat".to_string(), attacker_uuid)),
        Json(UpdateContentRequest {
            fields: Some(json!({
                "name": "Pwned",
                "desc": "evil",
                "uuid": victim_uuid,
            })),
            module_uuid: None,
        }),
    )
    .await
    .expect("editing own record");

    // Victim survives untouched; the edit landed on Alice's own record.
    let victim_name: Option<String> = sqlx::query_scalar("SELECT name FROM feat WHERE uuid = ?")
        .bind(&victim_uuid)
        .fetch_optional(&state.db)
        .await
        .unwrap();
    assert_eq!(victim_name.as_deref(), Some("Bob's Feat"));

    let attacker_name: String = sqlx::query_scalar("SELECT name FROM feat WHERE uuid = ?")
        .bind(attacker_uuid.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(attacker_name, "Pwned");
}

#[tokio::test]
async fn delete_roundtrip() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;

    let created = create_feat(&state, &alice, "Tough", None).await.unwrap();
    let uuid: Uuid = created["uuid"].as_str().unwrap().parse().unwrap();

    let status = delete_content(
        State(state.clone()),
        alice.clone(),
        Path(("feat".to_string(), uuid)),
    )
    .await
    .expect("deleting feat");
    assert_eq!(status, StatusCode::NO_CONTENT);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM feat WHERE uuid = ?")
        .bind(uuid.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
