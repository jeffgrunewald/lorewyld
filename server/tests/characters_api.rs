//! Character API authorization invariants: every authenticated user
//! can read every sheet, writes are owner-or-admin, and owner
//! attribution lives in the identity columns rather than the stored
//! JSON blob.

use axum::Json;
use axum::extract::{Path, State};
use lorewyld::api::ApiState;
use lorewyld::api::auth::CurrentUser;
use lorewyld::api::characters::{
    create_character, delete_character, get_character, list_characters, replace_character,
};
use lorewyld::api::error::ApiError;
use lorewyld_types::CharacterSheet;
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

/// A minimal sheet via serde defaults — only the name is required.
fn sheet_named(name: &str) -> CharacterSheet {
    serde_json::from_value(serde_json::json!({ "name": name })).expect("sheet from defaults")
}

async fn create(state: &ApiState, user: &CurrentUser, name: &str) -> CharacterSheet {
    let (status, Json(created)) =
        create_character(State(state.clone()), user.clone(), Json(sheet_named(name)))
            .await
            .expect("creating character");
    assert_eq!(status, axum::http::StatusCode::CREATED);
    created
}

#[tokio::test]
async fn owner_crud_roundtrip() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;

    let created = create(&state, &alice, "Thistle").await;
    assert_eq!(created.owner_user_uuid, Some(alice.uuid));
    assert_eq!(created.owner_username.as_deref(), Some("alice"));

    let Json(listed) = list_characters(State(state.clone()), alice.clone())
        .await
        .expect("listing characters");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].owner_username.as_deref(), Some("alice"));

    let Json(fetched) = get_character(State(state.clone()), alice.clone(), Path(created.uuid))
        .await
        .expect("fetching character");
    assert_eq!(fetched.name, "Thistle");

    let mut update = sheet_named("Thistle Quickfoot");
    update.level = 3;
    let Json(replaced) = replace_character(
        State(state.clone()),
        alice.clone(),
        Path(created.uuid),
        Json(update),
    )
    .await
    .expect("replacing character");
    assert_eq!(replaced.name, "Thistle Quickfoot");
    assert_eq!(replaced.level, 3);
    assert_eq!(replaced.owner_user_uuid, Some(alice.uuid));

    let status = delete_character(State(state.clone()), alice, Path(created.uuid))
        .await
        .expect("deleting character");
    assert_eq!(status, axum::http::StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn non_owner_can_read_but_not_write() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;
    let bob = insert_user(&state.db, "bob", false).await;

    let created = create(&state, &alice, "Thistle").await;

    let Json(listed) = list_characters(State(state.clone()), bob.clone())
        .await
        .expect("listing as non-owner");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].owner_username.as_deref(), Some("alice"));

    let Json(fetched) = get_character(State(state.clone()), bob.clone(), Path(created.uuid))
        .await
        .expect("reading as non-owner");
    assert_eq!(fetched.owner_user_uuid, Some(alice.uuid));

    let replace = replace_character(
        State(state.clone()),
        bob.clone(),
        Path(created.uuid),
        Json(sheet_named("Stolen")),
    )
    .await;
    assert!(matches!(replace, Err(ApiError::Forbidden)));

    let delete = delete_character(State(state.clone()), bob, Path(created.uuid)).await;
    assert!(matches!(delete, Err(ApiError::Forbidden)));
}

#[tokio::test]
async fn admin_can_write_others_characters() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;
    let carol = insert_user(&state.db, "carol", true).await;

    let created = create(&state, &alice, "Thistle").await;

    let Json(replaced) = replace_character(
        State(state.clone()),
        carol.clone(),
        Path(created.uuid),
        Json(sheet_named("Thistle, revised")),
    )
    .await
    .expect("admin replacing another user's character");
    // Admin edits never steal ownership.
    assert_eq!(replaced.owner_user_uuid, Some(alice.uuid));
    assert_eq!(replaced.owner_username.as_deref(), Some("alice"));

    let status = delete_character(State(state.clone()), carol, Path(created.uuid))
        .await
        .expect("admin deleting another user's character");
    assert_eq!(status, axum::http::StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn stored_blob_has_no_owner_fields() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;

    let created = create(&state, &alice, "Thistle").await;

    let data: String = sqlx::query_scalar("SELECT data FROM character WHERE uuid = ?")
        .bind(created.uuid.to_string())
        .fetch_one(&state.db)
        .await
        .expect("reading stored blob");
    let blob: serde_json::Value = serde_json::from_str(&data).expect("parsing stored blob");
    assert!(blob.get("owner_user_uuid").is_none());
    assert!(blob.get("owner_username").is_none());
}
