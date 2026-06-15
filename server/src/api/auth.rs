use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::{FromRef, FromRequestParts, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION, request::Parts},
};
use chrono::{DateTime, Utc};
use lorewyld_types::{
    api_v1::{AuthResponse, ChangePasswordRequest, LoginRequest, RegisterRequest},
    user::User,
};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api::{
    ApiState,
    error::{ApiError, is_unique_violation},
};

/// Authenticated caller resolved from a `Authorization: Bearer <token>`
/// header. Handlers that require auth accept `CurrentUser` as an
/// extractor; the extractor returns `401 Unauthorized` if the header
/// is missing or the token is unknown.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub uuid: Uuid,
    pub username: String,
    pub email: String,
    pub admin: bool,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
    ApiState: axum::extract::FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = bearer_token(&parts.headers).ok_or(ApiError::Unauthorized)?;
        let api_state = ApiState::from_ref(state);
        resolve_session(&api_state.db, &token).await
    }
}

/// Admin-gated extractor: resolves the session like `CurrentUser`, then
/// rejects non-admin callers with `403 Forbidden`. All `/api/admin/*`
/// handlers take this instead of `CurrentUser`.
#[derive(Debug, Clone)]
pub struct AdminUser(pub CurrentUser);

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
    ApiState: axum::extract::FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = CurrentUser::from_request_parts(parts, state).await?;
        user.admin
            .then_some(AdminUser(user))
            .ok_or(ApiError::Forbidden)
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_owned)
}

async fn resolve_session(db: &SqlitePool, token: &str) -> Result<CurrentUser, ApiError> {
    let row = sqlx::query_as::<_, (String, String, String, bool, Option<DateTime<Utc>>)>(
        "SELECT u.id, u.username, u.email, u.admin, s.expires_at
           FROM user_session s
           JOIN users u ON u.id = s.user_uuid
          WHERE s.token = ?",
    )
    .bind(token)
    .fetch_optional(db)
    .await?;

    let (uuid, username, email, admin, expires_at) = row.ok_or(ApiError::Unauthorized)?;

    if let Some(expiry) = expires_at
        && expiry < Utc::now()
    {
        return Err(ApiError::Unauthorized);
    }

    Ok(CurrentUser {
        uuid: Uuid::parse_str(&uuid).map_err(|e| ApiError::Internal(e.into()))?,
        username,
        email,
        admin,
    })
}

/// Argon2 hashing with default params costs tens of milliseconds, so
/// both helpers run on the blocking pool to keep the async workers free.
pub async fn hash_password(password: String) -> Result<String, ApiError> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("password hashing failed: {e}")))
    })
    .await
    .map_err(|e| ApiError::Internal(e.into()))?
}

async fn verify_password(password: String, phc_hash: String) -> Result<bool, ApiError> {
    tokio::task::spawn_blocking(move || {
        PasswordHash::new(&phc_hash)
            .map(|parsed| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &parsed)
                    .is_ok()
            })
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("stored password hash invalid: {e}")))
    })
    .await
    .map_err(|e| ApiError::Internal(e.into()))?
}

/// Password policy shared by registration, admin creation, and
/// self-service change.
pub fn validate_password(password: &str) -> Result<(), ApiError> {
    if password.len() < 8 {
        return Err(ApiError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }
    Ok(())
}

/// Shared validation for registration and admin user creation.
pub fn validate_new_user(username: &str, email: &str, password: &str) -> Result<(), ApiError> {
    if username.trim().is_empty() {
        return Err(ApiError::BadRequest("username must not be empty".into()));
    }
    if !email.contains('@') {
        return Err(ApiError::BadRequest("email must be a valid address".into()));
    }
    validate_password(password)
}

/// Maps a UNIQUE-constraint violation on `users` to the right 409. The
/// SQLite message names the failing column ("users.username" /
/// "users.email"), which is the only signal available through sqlx.
pub fn map_user_unique_violation(err: sqlx::Error) -> ApiError {
    let message = match &err {
        sqlx::Error::Database(db) => db.message().to_owned(),
        _ => String::new(),
    };
    if message.contains("users.email") {
        ApiError::EmailTaken
    } else if message.contains("users.username") {
        ApiError::UsernameTaken
    } else {
        ApiError::Internal(err.into())
    }
}

/// Inserts a user row with a freshly hashed password and returns its id.
pub async fn insert_user(
    db: &SqlitePool,
    username: &str,
    email: &str,
    password: String,
) -> Result<String, ApiError> {
    let password_hash = hash_password(password).await?;

    // The UNIQUE constraints are the real uniqueness guard; a SELECT
    // pre-check would race a concurrent registration and surface the
    // constraint violation as a 500 instead of 409.
    let user_uuid = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO users (id, username, email, password_hash) VALUES (?, ?, ?, ?)")
        .bind(&user_uuid)
        .bind(username)
        .bind(email)
        .bind(&password_hash)
        .execute(db)
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                map_user_unique_violation(e)
            } else {
                e.into()
            }
        })?;

    Ok(user_uuid)
}

#[utoipa::path(
    post,
    path = "/api/users/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Account created; returns the user and a session token", body = AuthResponse),
        (status = 403, description = "Invalid join code"),
        (status = 409, description = "Username or email already taken"),
    )
)]
pub async fn register(
    State(state): State<ApiState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let (join_code,): (String,) = sqlx::query_as("SELECT join_code FROM game_server LIMIT 1")
        .fetch_one(&state.db)
        .await?;

    if req.join_code != join_code {
        return Err(ApiError::InvalidJoinCode);
    }

    validate_new_user(&req.username, &req.email, &req.password)?;
    let user_uuid = insert_user(&state.db, &req.username, &req.email, req.password).await?;

    let response = build_auth_response(&state.db, &user_uuid).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/users/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Authenticated; returns the user and a session token", body = AuthResponse),
        (status = 401, description = "Invalid username or password"),
    )
)]
pub async fn login(
    State(state): State<ApiState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let user_row: Option<(String, String)> =
        sqlx::query_as("SELECT id, password_hash FROM users WHERE username = ?")
            .bind(&req.username)
            .fetch_optional(&state.db)
            .await?;

    // Same error for unknown username and wrong password — responses
    // must not reveal which usernames exist.
    let (user_uuid, password_hash) = user_row.ok_or(ApiError::InvalidCredentials)?;
    if !verify_password(req.password, password_hash).await? {
        return Err(ApiError::InvalidCredentials);
    }

    let response = build_auth_response(&state.db, &user_uuid).await?;
    Ok(Json(response))
}

/// Revokes the presented session token. Idempotent: deleting an already
/// revoked token still returns `204`.
#[utoipa::path(
    post,
    path = "/api/users/logout",
    tag = "auth",
    security(("bearer" = [])),
    responses(
        (status = 204, description = "Session token revoked (idempotent)"),
        (status = 401, description = "Missing or malformed bearer token"),
    )
)]
pub async fn logout(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let token = bearer_token(&headers).ok_or(ApiError::Unauthorized)?;

    sqlx::query("DELETE FROM user_session WHERE token = ?")
        .bind(&token)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "auth",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "The authenticated user", body = User),
        (status = 401, description = "Missing or invalid session"),
    )
)]
pub async fn me(State(state): State<ApiState>, user: CurrentUser) -> Result<Json<User>, ApiError> {
    let user = fetch_user(&state.db, &user.uuid.to_string()).await?;
    Ok(Json(user))
}

/// `POST /api/users/password` — self-service password change. Verifies
/// the current password, applies the registration password policy to
/// the new one, then revokes every *other* session so a stolen token
/// can't outlive a password rotation; the presenting session stays
/// valid.
pub async fn change_password(
    State(state): State<ApiState>,
    headers: HeaderMap,
    user: CurrentUser,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<StatusCode, ApiError> {
    let token = bearer_token(&headers).ok_or(ApiError::Unauthorized)?;
    validate_password(&req.new_password)?;

    let user_uuid = user.uuid.to_string();
    let (current_hash,): (String,) = sqlx::query_as("SELECT password_hash FROM users WHERE id = ?")
        .bind(&user_uuid)
        .fetch_one(&state.db)
        .await?;
    if !verify_password(req.current_password, current_hash).await? {
        return Err(ApiError::BadRequest("current password is incorrect".into()));
    }

    let new_hash = hash_password(req.new_password).await?;
    sqlx::query("UPDATE users SET password_hash = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(&new_hash)
        .bind(&user_uuid)
        .execute(&state.db)
        .await?;

    sqlx::query("DELETE FROM user_session WHERE user_uuid = ? AND token != ?")
        .bind(&user_uuid)
        .bind(&token)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Loads the full wire-type `User` row by id.
pub async fn fetch_user(db: &SqlitePool, user_uuid: &str) -> Result<User, ApiError> {
    let (id, username, email, admin, created_at): (String, String, String, bool, DateTime<Utc>) =
        sqlx::query_as("SELECT id, username, email, admin, created_at FROM users WHERE id = ?")
            .bind(user_uuid)
            .fetch_one(db)
            .await?;

    Ok(User {
        uuid: Uuid::parse_str(&id).map_err(|e| ApiError::Internal(e.into()))?,
        username,
        email,
        admin,
        created_at,
    })
}

async fn build_auth_response(db: &SqlitePool, user_uuid: &str) -> Result<AuthResponse, ApiError> {
    let token = Uuid::new_v4().to_string();
    // v1 sessions are permanent: expires_at stays NULL, so the expiry
    // check in resolve_session never fires. v1.5 adds an expiry policy
    // by stamping expires_at here.
    sqlx::query("INSERT INTO user_session (token, user_uuid) VALUES (?, ?)")
        .bind(&token)
        .bind(user_uuid)
        .execute(db)
        .await?;

    let user = fetch_user(db, user_uuid).await?;

    Ok(AuthResponse {
        user,
        session_token: token,
    })
}
