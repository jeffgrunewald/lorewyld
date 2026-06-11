//! Admin-only endpoints: user management and server settings. Every
//! handler takes the [`AdminUser`] extractor, so non-admin callers are
//! rejected with `403 Forbidden` before any handler logic runs.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use lorewyld_types::{
    api_v1::{
        AdminCreateUserRequest, AdminUpdateUserRequest, ServerSettings,
        UpdateServerSettingsRequest, UserListResponse,
    },
    user::User,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::{
    ApiState,
    auth::{AdminUser, fetch_user, insert_user, validate_new_user},
    error::ApiError,
};

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub limit: Option<u32>,
}

/// `GET /api/admin/users?page&limit` — one page of registered users.
pub async fn list_users(
    State(state): State<ApiState>,
    _admin: AdminUser,
    Query(query): Query<UserListQuery>,
) -> Result<Json<UserListResponse>, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    let (total,): (u32,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?;

    let rows: Vec<(String, String, String, bool, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT id, username, email, admin, created_at FROM users \
         ORDER BY username LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let users = rows
        .into_iter()
        .map(|(id, username, email, admin, created_at)| {
            Ok(User {
                uuid: Uuid::parse_str(&id).map_err(|e| ApiError::Internal(e.into()))?,
                username,
                email,
                admin,
                created_at,
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;

    Ok(Json(UserListResponse {
        users,
        page,
        limit,
        total,
    }))
}

/// `POST /api/admin/users` — admin-driven account creation. Same shape
/// as registration minus the join code; admin access supersedes it.
pub async fn create_user(
    State(state): State<ApiState>,
    _admin: AdminUser,
    Json(req): Json<AdminCreateUserRequest>,
) -> Result<(StatusCode, Json<User>), ApiError> {
    validate_new_user(&req.username, &req.email, &req.password)?;
    let user_uuid = insert_user(&state.db, &req.username, &req.email, req.password).await?;
    let user = fetch_user(&state.db, &user_uuid).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// `DELETE /api/admin/users/:uuid` — remove an account. Sessions and
/// collaborator rows cascade; authored settings/notes orphan to a NULL
/// author. Self-deletion is blocked, which also guarantees at least one
/// admin always survives.
pub async fn delete_user(
    State(state): State<ApiState>,
    admin: AdminUser,
    Path(uuid): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    if uuid == admin.0.uuid {
        return Err(ApiError::BadRequest(
            "cannot delete your own account".into(),
        ));
    }

    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(uuid.to_string())
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// `PATCH /api/admin/users/:uuid` — toggle the admin flag. Removing
/// your own admin access is blocked so the server can't end up with no
/// admins.
pub async fn set_admin(
    State(state): State<ApiState>,
    admin: AdminUser,
    Path(uuid): Path<Uuid>,
    Json(req): Json<AdminUpdateUserRequest>,
) -> Result<Json<User>, ApiError> {
    if uuid == admin.0.uuid && !req.admin {
        return Err(ApiError::BadRequest(
            "cannot remove your own admin access".into(),
        ));
    }

    let result =
        sqlx::query("UPDATE users SET admin = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(req.admin)
            .bind(uuid.to_string())
            .execute(&state.db)
            .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    let user = fetch_user(&state.db, &uuid.to_string()).await?;
    Ok(Json(user))
}

/// `GET /api/admin/server` — server identity (editable name, read-only
/// join code) plus the read-only software version for display.
pub async fn get_server_settings(
    State(state): State<ApiState>,
    _admin: AdminUser,
) -> Result<Json<ServerSettings>, ApiError> {
    fetch_server_settings(&state.db).await.map(Json)
}

/// `PATCH /api/admin/server` — partial update of the server name. The
/// join code is never edited directly; see [`regenerate_join_code`].
pub async fn update_server_settings(
    State(state): State<ApiState>,
    _admin: AdminUser,
    Json(req): Json<UpdateServerSettingsRequest>,
) -> Result<Json<ServerSettings>, ApiError> {
    let next_name = req.name.as_deref().map(str::trim).map(str::to_string);
    // COALESCE only guards NULL — an explicit empty string would wipe
    // the field.
    if next_name.as_deref() == Some("") {
        return Err(ApiError::BadRequest("name cannot be empty".into()));
    }

    sqlx::query(
        "UPDATE game_server
            SET name       = COALESCE(?, name),
                updated_at = datetime('now')",
    )
    .bind(next_name)
    .execute(&state.db)
    .await?;

    fetch_server_settings(&state.db).await.map(Json)
}

/// `POST /api/admin/server/join-code` — replace the join code with a
/// freshly generated one. The old code stops gating registration
/// immediately. Generation reuses the boot-time generator, so the new
/// code satisfies the same format constraint by construction.
pub async fn regenerate_join_code(
    State(state): State<ApiState>,
    _admin: AdminUser,
) -> Result<Json<ServerSettings>, ApiError> {
    let code = crate::generate_join_code();
    debug_assert!(crate::is_valid_join_code(&code));

    sqlx::query("UPDATE game_server SET join_code = ?, updated_at = datetime('now')")
        .bind(&code)
        .execute(&state.db)
        .await?;

    fetch_server_settings(&state.db).await.map(Json)
}

async fn fetch_server_settings(db: &sqlx::SqlitePool) -> Result<ServerSettings, ApiError> {
    let (name, join_code, version): (String, String, String) =
        sqlx::query_as("SELECT name, join_code, version FROM game_server LIMIT 1")
            .fetch_one(db)
            .await?;
    Ok(ServerSettings {
        name,
        join_code,
        version,
    })
}
