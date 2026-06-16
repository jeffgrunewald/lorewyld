//! Character sheet CRUD. Sheets are readable by every authenticated
//! user (with owner attribution); writes are restricted to the owner
//! or an admin. The full sheet JSON lives in `character.data`; the
//! identity columns (uuid, owner, name, timestamps) are authoritative
//! duplicates, and owner attribution is never baked into the blob.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use lorewyld_types::CharacterSheet;
use uuid::Uuid;

use crate::api::{
    ApiState,
    auth::CurrentUser,
    error::ApiError,
    rows::{CHARACTER_SELECT_WITH_OWNER, CharacterRow},
};

/// `GET /api/characters` — every character on the server, name-sorted,
/// with owner attribution.
#[utoipa::path(
    get,
    path = "/api/characters",
    tag = "characters",
    security(("bearer" = [])),
    responses((status = 200, description = "Every character on the server, name-sorted", body = [CharacterSheet]))
)]
pub async fn list_characters(
    State(state): State<ApiState>,
    _user: CurrentUser,
) -> Result<Json<Vec<CharacterSheet>>, ApiError> {
    let sql = format!("{CHARACTER_SELECT_WITH_OWNER} ORDER BY c.name");
    let rows: Vec<CharacterRow> = sqlx::query_as(&sql).fetch_all(&state.db).await?;
    rows.into_iter()
        .map(CharacterRow::into_dto)
        .collect::<Result<Vec<_>, _>>()
        .map(Json)
}

/// `POST /api/characters` — create a sheet. The server owns identity:
/// client-supplied uuid, owner, and timestamps are overwritten.
#[utoipa::path(
    post,
    path = "/api/characters",
    tag = "characters",
    security(("bearer" = [])),
    request_body = CharacterSheet,
    responses((status = 201, description = "Created; server-assigned uuid/owner/timestamps", body = CharacterSheet))
)]
pub async fn create_character(
    State(state): State<ApiState>,
    user: CurrentUser,
    Json(mut sheet): Json<CharacterSheet>,
) -> Result<(StatusCode, Json<CharacterSheet>), ApiError> {
    if sheet.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name is required".to_string()));
    }
    sheet.name = sheet.name.trim().to_string();
    sheet.uuid = Uuid::new_v4();
    let now = chrono::Utc::now();
    sheet.created_at = now;
    sheet.updated_at = now;

    // Ownership lives in the identity columns, never in the blob.
    sheet.owner_user_uuid = None;
    sheet.owner_username = None;
    let data = serde_json::to_string(&sheet).map_err(|e| ApiError::Internal(e.into()))?;
    sqlx::query(
        "INSERT INTO character (uuid, owner_user_uuid, name, data, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(sheet.uuid.to_string())
    .bind(user.uuid.to_string())
    .bind(&sheet.name)
    .bind(&data)
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(&state.db)
    .await?;

    sheet.owner_user_uuid = Some(user.uuid);
    sheet.owner_username = Some(user.username.clone());
    Ok((StatusCode::CREATED, Json(sheet)))
}

/// `GET /api/characters/:uuid` — readable by any authenticated user.
#[utoipa::path(
    get,
    path = "/api/characters/{uuid}",
    tag = "characters",
    security(("bearer" = [])),
    params(("uuid" = String, Path, description = "Character UUID")),
    responses(
        (status = 200, description = "The character sheet", body = CharacterSheet),
        (status = 404, description = "No such character"),
    )
)]
pub async fn get_character(
    State(state): State<ApiState>,
    _user: CurrentUser,
    Path(uuid): Path<Uuid>,
) -> Result<Json<CharacterSheet>, ApiError> {
    fetch(&state, uuid).await?.into_dto().map(Json)
}

/// `PUT /api/characters/:uuid` — full-document replace (last write
/// wins, matching the mobile sheet-save model). Owner and `created_at`
/// are preserved — an admin editing keeps the original owner;
/// `updated_at` is bumped server-side.
#[utoipa::path(
    put,
    path = "/api/characters/{uuid}",
    tag = "characters",
    security(("bearer" = [])),
    params(("uuid" = String, Path, description = "Character UUID")),
    request_body = CharacterSheet,
    responses(
        (status = 200, description = "The updated sheet", body = CharacterSheet),
        (status = 403, description = "Not the owner or an admin"),
        (status = 404, description = "No such character"),
    )
)]
pub async fn replace_character(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path(uuid): Path<Uuid>,
    Json(mut sheet): Json<CharacterSheet>,
) -> Result<Json<CharacterSheet>, ApiError> {
    if sheet.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name is required".to_string()));
    }
    let existing = fetch_writable(&state, &user, uuid).await?;

    sheet.name = sheet.name.trim().to_string();
    sheet.uuid = uuid;
    sheet.created_at = existing.created_at;
    sheet.updated_at = chrono::Utc::now();

    sheet.owner_user_uuid = None;
    sheet.owner_username = None;
    let data = serde_json::to_string(&sheet).map_err(|e| ApiError::Internal(e.into()))?;
    sqlx::query("UPDATE character SET name = ?, data = ?, updated_at = ? WHERE uuid = ?")
        .bind(&sheet.name)
        .bind(&data)
        .bind(sheet.updated_at.to_rfc3339())
        .bind(uuid.to_string())
        .execute(&state.db)
        .await?;

    sheet.owner_user_uuid =
        Some(Uuid::parse_str(&existing.owner_user_uuid).map_err(|e| ApiError::Internal(e.into()))?);
    sheet.owner_username = existing.owner_username;
    Ok(Json(sheet))
}

/// `DELETE /api/characters/:uuid` — owner or admin.
#[utoipa::path(
    delete,
    path = "/api/characters/{uuid}",
    tag = "characters",
    security(("bearer" = [])),
    params(("uuid" = String, Path, description = "Character UUID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 403, description = "Not the owner or an admin"),
        (status = 404, description = "No such character"),
    )
)]
pub async fn delete_character(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path(uuid): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    fetch_writable(&state, &user, uuid).await?;
    sqlx::query("DELETE FROM character WHERE uuid = ?")
        .bind(uuid.to_string())
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Loads a character row by uuid, 404 on missing.
async fn fetch(state: &ApiState, uuid: Uuid) -> Result<CharacterRow, ApiError> {
    let sql = format!("{CHARACTER_SELECT_WITH_OWNER} WHERE c.uuid = ?");
    sqlx::query_as(&sql)
        .bind(uuid.to_string())
        .fetch_optional(&state.db)
        .await?
        .ok_or(ApiError::NotFound)
}

/// Loads a character row for mutation, distinguishing missing (404)
/// from not-writable (403). Writes are allowed for the owner or any
/// admin.
async fn fetch_writable(
    state: &ApiState,
    user: &CurrentUser,
    uuid: Uuid,
) -> Result<CharacterRow, ApiError> {
    let row = fetch(state, uuid).await?;
    if row.owner_user_uuid != user.uuid.to_string() && !user.admin {
        return Err(ApiError::Forbidden);
    }
    Ok(row)
}
