//! Custom (homebrew `local`) module lifecycle + bundle export/import:
//! creation + authorship gating, the pinned-module guard, and that an
//! exported module round-trips back through import on a fresh server.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use lorewyld::api::ApiState;
use lorewyld::api::auth::CurrentUser;
use lorewyld::api::authoring::create_content;
use lorewyld::api::error::ApiError;
use lorewyld::api::modules::{create_custom_module, delete_custom_module, update_custom_module};
use lorewyld::content::{self, ImportOptions, SlugConflict};
use lorewyld_types::api_v1::{CreateContentRequest, CreateModuleRequest, UpdateModuleRequest};
use lorewyld_types::{LicenseKind, ModuleOrigin};
use serde_json::json;
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

fn create_req(name: &str, slug: &str) -> CreateModuleRequest {
    CreateModuleRequest {
        name: name.to_string(),
        slug: slug.to_string(),
        license: LicenseKind::Unlicensed,
        license_url: None,
        description: Some("A homebrew set.".to_string()),
        authors: vec!["alice".to_string()],
        website_url: None,
    }
}

async fn make_module(state: &ApiState, user: &CurrentUser, name: &str, slug: &str) -> Uuid {
    let (_, Json(m)) = create_custom_module(
        State(state.clone()),
        user.clone(),
        Json(create_req(name, slug)),
    )
    .await
    .expect("creating custom module");
    m.uuid
}

#[tokio::test]
async fn create_custom_module_is_local_and_owned() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;

    let uuid = make_module(&state, &alice, "My World", "my-world").await;
    let (origin, created_by): (String, Option<String>) =
        sqlx::query_as("SELECT origin, created_by_user_uuid FROM content_module WHERE uuid = ?")
            .bind(uuid.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(origin, "local");
    assert_eq!(created_by.as_deref(), Some(alice.uuid.to_string().as_str()));
}

#[tokio::test]
async fn duplicate_slug_is_rejected() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;
    make_module(&state, &alice, "My World", "my-world").await;

    let again = create_custom_module(
        State(state.clone()),
        alice,
        Json(create_req("My World 2", "my-world")),
    )
    .await;
    assert!(matches!(again, Err(ApiError::BadRequest(_))));
}

#[tokio::test]
async fn content_can_be_authored_into_a_custom_module() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;
    let module = make_module(&state, &alice, "My World", "my-world").await;

    let (_, Json(feat)) = create_content(
        State(state.clone()),
        alice.clone(),
        Path("feat".to_string()),
        Json(CreateContentRequest {
            fields: json!({ "name": "Brawler", "desc": "Unarmed mastery." }),
            module_uuid: Some(module),
        }),
    )
    .await
    .expect("authoring into custom module");
    assert_eq!(feat["content_module_uuid"], json!(module.to_string()));
}

#[tokio::test]
async fn edit_and_delete_are_creator_or_admin_only() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;
    let bob = insert_user(&state.db, "bob", false).await;
    let admin = insert_user(&state.db, "carol", true).await;
    let module = make_module(&state, &alice, "My World", "my-world").await;

    let patch = |user: CurrentUser| {
        update_custom_module(
            State(state.clone()),
            user,
            Path(module),
            Json(UpdateModuleRequest {
                name: Some("Renamed".to_string()),
                ..Default::default()
            }),
        )
    };
    assert!(matches!(patch(bob.clone()).await, Err(ApiError::Forbidden)));
    let Json(updated) = patch(admin.clone()).await.expect("admin edits");
    assert_eq!(updated.name, "Renamed");
    let Json(_) = patch(alice.clone()).await.expect("creator edits");

    // Bob can't delete; the creator can.
    assert!(matches!(
        delete_custom_module(State(state.clone()), bob, Path(module)).await,
        Err(ApiError::Forbidden)
    ));
    let status = delete_custom_module(State(state.clone()), alice, Path(module))
        .await
        .expect("creator deletes");
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn homebrew_module_cannot_be_deleted() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;

    // Authoring without a module lazily creates the pinned Homebrew module.
    let _ = create_content(
        State(state.clone()),
        alice.clone(),
        Path("feat".to_string()),
        Json(CreateContentRequest {
            fields: json!({ "name": "X", "desc": "y" }),
            module_uuid: None,
        }),
    )
    .await
    .unwrap();
    let (hb,): (String,) =
        sqlx::query_as("SELECT uuid FROM content_module WHERE slug = 'homebrew'")
            .fetch_one(&state.db)
            .await
            .unwrap();
    let res = delete_custom_module(State(state.clone()), alice, Path(hb.parse().unwrap())).await;
    assert!(matches!(res, Err(ApiError::BadRequest(_))));
}

#[tokio::test]
async fn exported_module_round_trips_through_import() {
    let pool = fresh_pool().await;
    let state = ApiState { db: pool };
    let alice = insert_user(&state.db, "alice", false).await;

    // Build a module with one authored record, then export it.
    let (_, Json(module)) = create_custom_module(
        State(state.clone()),
        alice.clone(),
        Json(create_req("My World", "my-world")),
    )
    .await
    .unwrap();
    let _ = create_content(
        State(state.clone()),
        alice.clone(),
        Path("feat".to_string()),
        Json(CreateContentRequest {
            fields: json!({ "name": "Brawler", "desc": "Unarmed mastery." }),
            module_uuid: Some(module.uuid),
        }),
    )
    .await
    .unwrap();

    let bundle = content::export_module(&state.db, module.clone())
        .await
        .expect("exporting module");
    assert_eq!(bundle.modules.len(), 1);
    assert_eq!(bundle.feats.len(), 1);
    assert_eq!(bundle.feats[0].name, "Brawler");

    // Import the exact bundle into a fresh server: it installs as uploaded.
    let other = ApiState {
        db: fresh_pool().await,
    };
    let outcome = content::import_bundle(
        &other.db,
        &bundle,
        &ImportOptions {
            origin: ModuleOrigin::Uploaded,
            on_slug_conflict: SlugConflict::Reject,
            require_bundling_license: false,
        },
    )
    .await
    .expect("importing exported bundle");
    assert_eq!(outcome.installed.len(), 1);

    let (origin,): (String,) =
        sqlx::query_as("SELECT origin FROM content_module WHERE slug = 'my-world'")
            .fetch_one(&other.db)
            .await
            .unwrap();
    assert_eq!(origin, "uploaded");
    let feat_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM feat")
        .fetch_one(&other.db)
        .await
        .unwrap();
    assert_eq!(feat_count, 1);
}
