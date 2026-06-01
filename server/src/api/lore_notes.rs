use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use lorewyld_types::{
    api_v1::{CreateLoreNoteRequest, LoreNoteWithTags, UpdateLoreNoteRequest},
    lore_note::{LoreNote, NoteScope, NoteScopeKind, NoteVisibility},
};
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api::{
    ApiState,
    auth::CurrentUser,
    error::ApiError,
    tags::{load_tags_for_note, resolve_or_create_tags},
};

#[derive(Debug, Deserialize)]
pub struct LoreNoteListQuery {
    #[serde(default)]
    pub scope_kind: Option<String>,
    #[serde(default)]
    pub scope_target: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

pub async fn list_lore_notes(
    State(state): State<ApiState>,
    user: CurrentUser,
    Query(query): Query<LoreNoteListQuery>,
) -> Result<Json<Vec<LoreNoteWithTags>>, ApiError> {
    let limit = query.limit.unwrap_or(100).clamp(1, 500);

    let mut sql = String::from(
        "SELECT n.uuid, n.title, n.body_markdown, n.scope_kind, n.scope_target_uuid,
                n.visibility, n.derived_from_setting_note_uuid, n.created_by_user_uuid,
                n.created_at, n.updated_at
           FROM lore_note n",
    );
    let mut joins = String::new();
    let mut conds = Vec::<String>::new();
    let mut binds = Vec::<String>::new();

    if let Some(scope_kind) = query.scope_kind.as_deref() {
        validate_scope_kind(scope_kind)?;
        conds.push("n.scope_kind = ?".into());
        binds.push(scope_kind.to_string());
    }
    if let Some(scope_target) = query.scope_target.as_deref() {
        conds.push("n.scope_target_uuid = ?".into());
        binds.push(scope_target.to_string());
    }
    if let Some(tag_slug) = query.tag.as_deref().filter(|s| !s.is_empty()) {
        joins.push_str(
            " JOIN tag_attachment_lore_note tan ON tan.lore_note_uuid = n.uuid \
              JOIN tag t ON t.uuid = tan.tag_uuid",
        );
        conds.push("t.slug = ?".into());
        binds.push(tag_slug.to_lowercase());
    }

    // Visibility filter: caller sees Visible always, plus their own
    // AuthorOnly/GamemasterOnly notes. (Campaign-DM-based visibility for
    // GamemasterOnly arrives in v1.5 with the Campaign entity.)
    conds.push("(n.visibility = 'visible' OR n.created_by_user_uuid = ?)".into());
    binds.push(user.uuid.to_string());

    if !joins.is_empty() {
        sql.push_str(&joins);
    }
    if !conds.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conds.join(" AND "));
    }
    sql.push_str(" ORDER BY n.updated_at DESC LIMIT ?");

    let mut query_builder = sqlx::query_as::<_, LoreNoteRow>(&sql);
    for b in &binds {
        query_builder = query_builder.bind(b);
    }
    query_builder = query_builder.bind(limit);

    let rows = query_builder.fetch_all(&state.db).await?;
    let mut notes = Vec::with_capacity(rows.len());
    for row in rows {
        let note = row.into_dto()?;
        let tags = load_tags_for_note(&state.db, &note.uuid.to_string()).await?;
        notes.push(LoreNoteWithTags { note, tags });
    }
    Ok(Json(notes))
}

pub async fn get_lore_note(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path(uuid): Path<Uuid>,
) -> Result<Json<LoreNoteWithTags>, ApiError> {
    let row: Option<LoreNoteRow> = sqlx::query_as(LORE_NOTE_SELECT)
        .bind(uuid.to_string())
        .fetch_optional(&state.db)
        .await?;
    let row = row.ok_or(ApiError::NotFound)?;
    let note = row.into_dto()?;
    if !can_view(&note, &user) {
        return Err(ApiError::NotFound);
    }
    let tags = load_tags_for_note(&state.db, &note.uuid.to_string()).await?;
    Ok(Json(LoreNoteWithTags { note, tags }))
}

pub async fn create_lore_note(
    State(state): State<ApiState>,
    user: CurrentUser,
    Json(req): Json<CreateLoreNoteRequest>,
) -> Result<(StatusCode, Json<LoreNoteWithTags>), ApiError> {
    if req.title.trim().is_empty() {
        return Err(ApiError::BadRequest("title is required".into()));
    }
    validate_scope_authorization(&state.db, &req.scope, &user).await?;

    // Resolve tags first — SQLite serializes writes, so we can't open
    // a transaction here and then use the pool inside it (two connections
    // racing for the same write lock deadlocks).
    let tag_uuids = resolve_or_create_tags(&state.db, &req.tag_slugs).await?;

    let uuid = Uuid::new_v4().to_string();
    let scope_kind = scope_kind_to_str(req.scope.kind);
    let visibility = visibility_to_str(req.visibility);

    let mut tx = state.db.begin().await?;
    sqlx::query(
        "INSERT INTO lore_note (
            uuid, title, body_markdown, scope_kind, scope_target_uuid,
            visibility, created_by_user_uuid
         ) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&uuid)
    .bind(req.title.trim())
    .bind(&req.body_markdown)
    .bind(scope_kind)
    .bind(req.scope.target_uuid.to_string())
    .bind(visibility)
    .bind(user.uuid.to_string())
    .execute(&mut *tx)
    .await?;

    for tag_uuid in &tag_uuids {
        sqlx::query(
            "INSERT INTO tag_attachment_lore_note (tag_uuid, lore_note_uuid) VALUES (?, ?)",
        )
        .bind(tag_uuid)
        .bind(&uuid)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    let row: LoreNoteRow = sqlx::query_as(LORE_NOTE_SELECT)
        .bind(&uuid)
        .fetch_one(&state.db)
        .await?;
    let note = row.into_dto()?;
    let tags = load_tags_for_note(&state.db, &uuid).await?;
    Ok((StatusCode::CREATED, Json(LoreNoteWithTags { note, tags })))
}

pub async fn update_lore_note(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path(uuid): Path<Uuid>,
    Json(req): Json<UpdateLoreNoteRequest>,
) -> Result<Json<LoreNoteWithTags>, ApiError> {
    let row: Option<LoreNoteRow> = sqlx::query_as(LORE_NOTE_SELECT)
        .bind(uuid.to_string())
        .fetch_optional(&state.db)
        .await?;
    let row = row.ok_or(ApiError::NotFound)?;
    let existing = row.into_dto()?;
    if existing.created_by_user_uuid != user.uuid {
        return Err(ApiError::Unauthorized);
    }

    // Resolve tags before the transaction (see create_lore_note for why).
    let resolved_tag_uuids = if let Some(new_slugs) = req.tag_slugs.as_ref() {
        Some(resolve_or_create_tags(&state.db, new_slugs).await?)
    } else {
        None
    };

    let mut tx = state.db.begin().await?;

    let next_title = req.title.as_deref().map(str::trim).map(str::to_string);
    let next_body = req.body_markdown.clone();
    let next_visibility = req.visibility.map(visibility_to_str);

    sqlx::query(
        "UPDATE lore_note
            SET title         = COALESCE(?, title),
                body_markdown = COALESCE(?, body_markdown),
                visibility    = COALESCE(?, visibility),
                updated_at    = datetime('now')
          WHERE uuid = ?",
    )
    .bind(next_title)
    .bind(next_body)
    .bind(next_visibility)
    .bind(uuid.to_string())
    .execute(&mut *tx)
    .await?;

    if let Some(tag_uuids) = resolved_tag_uuids {
        sqlx::query("DELETE FROM tag_attachment_lore_note WHERE lore_note_uuid = ?")
            .bind(uuid.to_string())
            .execute(&mut *tx)
            .await?;
        for tag_uuid in &tag_uuids {
            sqlx::query(
                "INSERT INTO tag_attachment_lore_note (tag_uuid, lore_note_uuid) VALUES (?, ?)",
            )
            .bind(tag_uuid)
            .bind(uuid.to_string())
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    let row: LoreNoteRow = sqlx::query_as(LORE_NOTE_SELECT)
        .bind(uuid.to_string())
        .fetch_one(&state.db)
        .await?;
    let note = row.into_dto()?;
    let tags = load_tags_for_note(&state.db, &uuid.to_string()).await?;
    Ok(Json(LoreNoteWithTags { note, tags }))
}

pub async fn delete_lore_note(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path(uuid): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT created_by_user_uuid FROM lore_note WHERE uuid = ?")
            .bind(uuid.to_string())
            .fetch_optional(&state.db)
            .await?;
    let (creator,) = row.ok_or(ApiError::NotFound)?;
    if creator != user.uuid.to_string() {
        return Err(ApiError::Unauthorized);
    }
    sqlx::query("DELETE FROM lore_note WHERE uuid = ?")
        .bind(uuid.to_string())
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ─── helpers ────────────────────────────────────────────────────────────

const LORE_NOTE_SELECT: &str = "SELECT uuid, title, body_markdown, scope_kind, \
                                scope_target_uuid, visibility, \
                                derived_from_setting_note_uuid, \
                                created_by_user_uuid, created_at, updated_at \
                                FROM lore_note WHERE uuid = ?";

#[derive(sqlx::FromRow)]
struct LoreNoteRow {
    uuid: String,
    title: String,
    body_markdown: String,
    scope_kind: String,
    scope_target_uuid: String,
    visibility: String,
    derived_from_setting_note_uuid: Option<String>,
    created_by_user_uuid: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl LoreNoteRow {
    fn into_dto(self) -> Result<LoreNote, ApiError> {
        Ok(LoreNote {
            uuid: Uuid::parse_str(&self.uuid).map_err(|e| ApiError::Internal(e.into()))?,
            title: self.title,
            body_markdown: self.body_markdown,
            scope: NoteScope {
                kind: scope_kind_from_str(&self.scope_kind)?,
                target_uuid: Uuid::parse_str(&self.scope_target_uuid)
                    .map_err(|e| ApiError::Internal(e.into()))?,
            },
            visibility: visibility_from_str(&self.visibility)?,
            derived_from_setting_note_uuid: self
                .derived_from_setting_note_uuid
                .as_deref()
                .map(|s| Uuid::parse_str(s).map_err(|e| ApiError::Internal(e.into())))
                .transpose()?,
            created_by_user_uuid: Uuid::parse_str(&self.created_by_user_uuid)
                .map_err(|e| ApiError::Internal(e.into()))?,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

fn scope_kind_to_str(kind: NoteScopeKind) -> &'static str {
    match kind {
        NoteScopeKind::Module => "module",
        NoteScopeKind::Setting => "setting",
        NoteScopeKind::Campaign => "campaign",
        NoteScopeKind::Character => "character",
    }
}

fn scope_kind_from_str(s: &str) -> Result<NoteScopeKind, ApiError> {
    match s {
        "module" => Ok(NoteScopeKind::Module),
        "setting" => Ok(NoteScopeKind::Setting),
        "campaign" => Ok(NoteScopeKind::Campaign),
        "character" => Ok(NoteScopeKind::Character),
        other => Err(ApiError::Internal(anyhow::anyhow!(
            "unknown scope_kind in database: {other}"
        ))),
    }
}

fn validate_scope_kind(s: &str) -> Result<(), ApiError> {
    scope_kind_from_str(s).map(|_| ())
}

fn visibility_to_str(v: NoteVisibility) -> &'static str {
    match v {
        NoteVisibility::Visible => "visible",
        NoteVisibility::AuthorOnly => "author_only",
        NoteVisibility::GamemasterOnly => "gamemaster_only",
    }
}

fn visibility_from_str(s: &str) -> Result<NoteVisibility, ApiError> {
    match s {
        "visible" => Ok(NoteVisibility::Visible),
        "author_only" => Ok(NoteVisibility::AuthorOnly),
        "gamemaster_only" => Ok(NoteVisibility::GamemasterOnly),
        other => Err(ApiError::Internal(anyhow::anyhow!(
            "unknown visibility in database: {other}"
        ))),
    }
}

/// Can the given user view the given note? V1 rules:
/// - `Visible` → anyone authenticated.
/// - `AuthorOnly` → only the creator.
/// - `GamemasterOnly` → only the creator (extended to campaign DMs in v1.5 when Campaign lands).
fn can_view(note: &LoreNote, user: &CurrentUser) -> bool {
    match note.visibility {
        NoteVisibility::Visible => true,
        NoteVisibility::AuthorOnly | NoteVisibility::GamemasterOnly => {
            note.created_by_user_uuid == user.uuid
        }
    }
}

/// Verify the caller is allowed to author a note in the requested scope.
/// V1 only validates `Setting` scope ownership (the only authoring-target
/// entity that exists yet). `Module` scope is locked down to the
/// Promote-to-Module wizard's commit endpoint and shouldn't appear here.
/// `Campaign` and `Character` scope land in v1.5.
async fn validate_scope_authorization(
    db: &SqlitePool,
    scope: &NoteScope,
    user: &CurrentUser,
) -> Result<(), ApiError> {
    match scope.kind {
        NoteScopeKind::Setting => {
            let row: Option<(String,)> = sqlx::query_as(
                "SELECT owner_user_uuid FROM setting WHERE uuid = ?",
            )
            .bind(scope.target_uuid.to_string())
            .fetch_optional(db)
            .await?;
            let (owner,) = row.ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "no setting with uuid {} exists",
                    scope.target_uuid
                ))
            })?;
            if owner == user.uuid.to_string() {
                return Ok(());
            }
            let is_collaborator: Option<(i64,)> = sqlx::query_as(
                "SELECT 1 FROM setting_collaborator
                  WHERE setting_uuid = ? AND user_uuid = ?",
            )
            .bind(scope.target_uuid.to_string())
            .bind(user.uuid.to_string())
            .fetch_optional(db)
            .await?;
            if is_collaborator.is_some() {
                Ok(())
            } else {
                Err(ApiError::Unauthorized)
            }
        }
        NoteScopeKind::Module => Err(ApiError::BadRequest(
            "module-scope notes are created only via the Promote-to-Module endpoint".into(),
        )),
        NoteScopeKind::Campaign | NoteScopeKind::Character => Err(ApiError::BadRequest(
            "campaign- and character-scope notes are not yet supported".into(),
        )),
    }
}
