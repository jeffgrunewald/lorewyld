use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use lorewyld_types::{
    api_v1::{AddCollaboratorRequest, CreateSettingRequest, UpdateSettingRequest},
    setting::Setting,
};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api::{ApiState, auth::CurrentUser, error::ApiError};

#[derive(sqlx::FromRow)]
struct SettingRow {
    uuid: String,
    name: String,
    description_note_uuid: Option<String>,
    owner_user_uuid: Option<String>,
    published_as_module_uuid: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl SettingRow {
    fn into_dto(self) -> Result<Setting, ApiError> {
        Ok(Setting {
            uuid: Uuid::parse_str(&self.uuid).map_err(|e| ApiError::Internal(e.into()))?,
            name: self.name,
            description_note_uuid: self
                .description_note_uuid
                .as_deref()
                .map(|s| Uuid::parse_str(s).map_err(|e| ApiError::Internal(e.into())))
                .transpose()?,
            owner_user_uuid: self
                .owner_user_uuid
                .as_deref()
                .map(|s| Uuid::parse_str(s).map_err(|e| ApiError::Internal(e.into())))
                .transpose()?,
            published_as_module_uuid: self
                .published_as_module_uuid
                .as_deref()
                .map(|s| Uuid::parse_str(s).map_err(|e| ApiError::Internal(e.into())))
                .transpose()?,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

const SETTING_SELECT: &str = "SELECT uuid, name, description_note_uuid, owner_user_uuid, \
                              published_as_module_uuid, created_at, updated_at \
                              FROM setting";

pub async fn list_settings(
    State(state): State<ApiState>,
    user: CurrentUser,
) -> Result<Json<Vec<Setting>>, ApiError> {
    let sql = format!(
        "{SETTING_SELECT} WHERE owner_user_uuid = ?1 \
         OR uuid IN (SELECT setting_uuid FROM setting_collaborator WHERE user_uuid = ?1) \
         ORDER BY updated_at DESC"
    );
    let rows: Vec<SettingRow> = sqlx::query_as(&sql)
        .bind(user.uuid.to_string())
        .fetch_all(&state.db)
        .await?;
    rows.into_iter()
        .map(SettingRow::into_dto)
        .collect::<Result<_, _>>()
        .map(Json)
}

pub async fn get_setting(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path(uuid): Path<Uuid>,
) -> Result<Json<Setting>, ApiError> {
    // Same owner-or-collaborator access as list_settings; a setting the
    // caller can't list also shouldn't resolve by uuid (404, not 403,
    // so existence isn't leaked).
    let sql = format!(
        "{SETTING_SELECT} WHERE uuid = ?1 \
         AND (owner_user_uuid = ?2 \
              OR uuid IN (SELECT setting_uuid FROM setting_collaborator WHERE user_uuid = ?2))"
    );
    let row: Option<SettingRow> = sqlx::query_as(&sql)
        .bind(uuid.to_string())
        .bind(user.uuid.to_string())
        .fetch_optional(&state.db)
        .await?;
    let row = row.ok_or(ApiError::NotFound)?;
    Ok(Json(row.into_dto()?))
}

pub async fn create_setting(
    State(state): State<ApiState>,
    user: CurrentUser,
    Json(req): Json<CreateSettingRequest>,
) -> Result<(StatusCode, Json<Setting>), ApiError> {
    if req.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }
    let uuid = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO setting (uuid, name, description_note_uuid, owner_user_uuid)
         VALUES (?, ?, ?, ?)",
    )
    .bind(&uuid)
    .bind(req.name.trim())
    .bind(req.description_note_uuid.map(|u| u.to_string()))
    .bind(user.uuid.to_string())
    .execute(&state.db)
    .await?;

    let row: SettingRow = sqlx::query_as(&format!("{SETTING_SELECT} WHERE uuid = ?"))
        .bind(&uuid)
        .fetch_one(&state.db)
        .await?;
    Ok((StatusCode::CREATED, Json(row.into_dto()?)))
}

pub async fn update_setting(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path(uuid): Path<Uuid>,
    Json(req): Json<UpdateSettingRequest>,
) -> Result<Json<Setting>, ApiError> {
    require_owner(&state.db, &uuid, &user).await?;
    let next_name = req.name.as_deref().map(str::trim).map(str::to_string);
    // COALESCE only guards NULL — an explicit empty string would wipe
    // the name, which create_setting forbids.
    if next_name.as_deref() == Some("") {
        return Err(ApiError::BadRequest("name cannot be empty".into()));
    }
    sqlx::query(
        "UPDATE setting
            SET name                  = COALESCE(?, name),
                description_note_uuid = COALESCE(?, description_note_uuid),
                updated_at            = datetime('now')
          WHERE uuid = ?",
    )
    .bind(next_name)
    .bind(req.description_note_uuid.map(|u| u.to_string()))
    .bind(uuid.to_string())
    .execute(&state.db)
    .await?;

    let row: SettingRow = sqlx::query_as(&format!("{SETTING_SELECT} WHERE uuid = ?"))
        .bind(uuid.to_string())
        .fetch_one(&state.db)
        .await?;
    Ok(Json(row.into_dto()?))
}

pub async fn delete_setting(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path(uuid): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    require_owner(&state.db, &uuid, &user).await?;
    sqlx::query("DELETE FROM setting WHERE uuid = ?")
        .bind(uuid.to_string())
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_collaborator(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path(uuid): Path<Uuid>,
    Json(req): Json<AddCollaboratorRequest>,
) -> Result<impl IntoResponse, ApiError> {
    require_owner(&state.db, &uuid, &user).await?;
    let collaborator_exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM users WHERE id = ?")
        .bind(req.user_uuid.to_string())
        .fetch_optional(&state.db)
        .await?;
    if collaborator_exists.is_none() {
        return Err(ApiError::BadRequest("no such user".into()));
    }
    sqlx::query(
        "INSERT OR IGNORE INTO setting_collaborator (setting_uuid, user_uuid) VALUES (?, ?)",
    )
    .bind(uuid.to_string())
    .bind(req.user_uuid.to_string())
    .execute(&state.db)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_collaborator(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path((uuid, collab_uuid)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    require_owner(&state.db, &uuid, &user).await?;
    sqlx::query("DELETE FROM setting_collaborator WHERE setting_uuid = ? AND user_uuid = ?")
        .bind(uuid.to_string())
        .bind(collab_uuid.to_string())
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn require_owner(
    db: &SqlitePool,
    setting_uuid: &Uuid,
    user: &CurrentUser,
) -> Result<(), ApiError> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT owner_user_uuid FROM setting WHERE uuid = ?")
            .bind(setting_uuid.to_string())
            .fetch_optional(db)
            .await?;
    let (owner,) = row.ok_or(ApiError::NotFound)?;
    // NULL owner (deleted account) matches nobody — orphaned settings
    // are mutable by no one until admin tooling lands.
    if owner.as_deref() != Some(user.uuid.to_string().as_str()) {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}
