use axum::{
    Json,
    extract::{FromRef, FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts},
};
use chrono::{DateTime, Utc};
use lorewyld_types::{
    api_v1::{AuthResponse, LoginRequest, RegisterRequest},
    app_user::AppUser,
};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api::{ApiState, error::ApiError};

/// Authenticated caller resolved from a `Authorization: Bearer <token>`
/// header. Handlers that require auth accept `CurrentUser` as an
/// extractor; the extractor returns `401 Unauthorized` if the header
/// is missing or the token is unknown.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub uuid: Uuid,
    pub display_name: String,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
    ApiState: axum::extract::FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(ApiError::Unauthorized)?;

        let api_state = ApiState::from_ref(state);
        resolve_session(&api_state.db, token).await
    }
}

async fn resolve_session(db: &SqlitePool, token: &str) -> Result<CurrentUser, ApiError> {
    let row = sqlx::query_as::<_, (String, String, Option<DateTime<Utc>>)>(
        "SELECT u.uuid, u.display_name, s.expires_at
           FROM user_session s
           JOIN app_user u ON u.uuid = s.user_uuid
          WHERE s.token = ?",
    )
    .bind(token)
    .fetch_optional(db)
    .await?;

    let (uuid, display_name, expires_at) = row.ok_or(ApiError::Unauthorized)?;

    if let Some(expiry) = expires_at
        && expiry < Utc::now() {
        return Err(ApiError::Unauthorized);
    }

    Ok(CurrentUser {
        uuid: Uuid::parse_str(&uuid).map_err(|e| ApiError::Internal(e.into()))?,
        display_name,
    })
}

pub async fn register(
    State(state): State<ApiState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let server: (String, String) =
        sqlx::query_as("SELECT id, join_code FROM game_server LIMIT 1")
            .fetch_one(&state.db)
            .await?;
    let (server_id, join_code) = server;

    if req.join_code != join_code {
        return Err(ApiError::InvalidJoinCode);
    }

    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT uuid FROM app_user WHERE server_uuid = ? AND display_name = ?",
    )
    .bind(&server_id)
    .bind(&req.display_name)
    .fetch_optional(&state.db)
    .await?;
    if existing.is_some() {
        return Err(ApiError::DisplayNameTaken);
    }

    let user_uuid = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO app_user (uuid, server_uuid, display_name) VALUES (?, ?, ?)")
        .bind(&user_uuid)
        .bind(&server_id)
        .bind(&req.display_name)
        .execute(&state.db)
        .await?;

    let response = build_auth_response(&state.db, &user_uuid).await?;
    Ok(Json(response))
}

pub async fn login(
    State(state): State<ApiState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let user_row: Option<(String,)> = sqlx::query_as(
        "SELECT uuid FROM app_user
          WHERE server_uuid = (SELECT id FROM game_server LIMIT 1)
            AND display_name = ?",
    )
    .bind(&req.display_name)
    .fetch_optional(&state.db)
    .await?;

    let (user_uuid,) = user_row.ok_or(ApiError::DisplayNameNotFound)?;
    let response = build_auth_response(&state.db, &user_uuid).await?;
    Ok(Json(response))
}

async fn build_auth_response(db: &SqlitePool, user_uuid: &str) -> Result<AuthResponse, ApiError> {
    let token = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO user_session (token, user_uuid) VALUES (?, ?)")
        .bind(&token)
        .bind(user_uuid)
        .execute(db)
        .await?;

    let user_row: (String, String, String, DateTime<Utc>) = sqlx::query_as(
        "SELECT uuid, server_uuid, display_name, created_at FROM app_user WHERE uuid = ?",
    )
    .bind(user_uuid)
    .fetch_one(db)
    .await?;

    let user = AppUser {
        uuid: Uuid::parse_str(&user_row.0).map_err(|e| ApiError::Internal(e.into()))?,
        server_uuid: Uuid::parse_str(&user_row.1).map_err(|e| ApiError::Internal(e.into()))?,
        display_name: user_row.2,
        created_at: user_row.3,
    };

    Ok(AuthResponse {
        user,
        session_token: token,
    })
}
