use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use lorewyld_types::{
    api_v1::{LoreNoteWithTags, SearchRequest, SearchResponse},
    lore_note::{LoreNote, NoteScope, NoteScopeKind, NoteVisibility},
};
use uuid::Uuid;

use crate::api::{ApiState, auth::CurrentUser, error::ApiError, tags::load_tags_for_note};

const DEFAULT_LIMIT: u32 = 50;
const MAX_LIMIT: u32 = 200;

/// Unified search across `LoreNote`s. Composes:
///   - FTS5 free-text over `title + body_markdown` (if `q` set)
///   - scope kind + scope target filters (if set)
///   - tag-slug filter with AND semantics (note must carry every listed slug)
///   - visibility filter (Visible always; AuthorOnly/GamemasterOnly only
///     for the caller's own notes)
pub async fn search(
    State(state): State<ApiState>,
    user: CurrentUser,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, ApiError> {
    let limit = req
        .limit
        .map(|n| n.clamp(1, MAX_LIMIT))
        .unwrap_or(DEFAULT_LIMIT) as i64;

    let mut sql = String::from(
        "SELECT n.uuid, n.title, n.body_markdown, n.scope_kind, n.scope_target_uuid,
                n.visibility, n.derived_from_setting_note_uuid, n.created_by_user_uuid,
                n.created_at, n.updated_at
           FROM lore_note n",
    );
    let mut binds: Vec<String> = Vec::new();
    let mut conds: Vec<String> = Vec::new();

    if let Some(q) = req.q.as_deref().filter(|s| !s.is_empty()) {
        sql.push_str(" JOIN lore_note_fts fts ON fts.rowid = n.rowid");
        conds.push("lore_note_fts MATCH ?".into());
        binds.push(q.to_string());
    }

    if let Some(kind) = req.scope_kind {
        conds.push("n.scope_kind = ?".into());
        binds.push(scope_kind_to_str(kind).to_string());
    }
    if let Some(target) = req.scope_target_uuid {
        conds.push("n.scope_target_uuid = ?".into());
        binds.push(target.to_string());
    }

    let tag_slug_count = req.tag_slugs.len();
    if tag_slug_count > 0 {
        let placeholders = vec!["?"; tag_slug_count].join(", ");
        // Inline the count literal because SQLite's HAVING compares
        // INTEGER COUNT(...) against bound values via type affinity —
        // a string-bound '1' won't match INTEGER 1. The count is derived
        // from the request body's `tag_slugs.len()` and carries no
        // injection risk.
        conds.push(format!(
            "n.uuid IN (
                SELECT tan.lore_note_uuid
                  FROM tag_attachment_lore_note tan
                  JOIN tag t ON t.uuid = tan.tag_uuid
                 WHERE t.slug IN ({placeholders})
                 GROUP BY tan.lore_note_uuid
                HAVING COUNT(DISTINCT t.uuid) = {tag_slug_count}
            )"
        ));
        for slug in &req.tag_slugs {
            binds.push(slug.to_lowercase());
        }
    }

    // Visibility filter applies last (always present).
    conds.push("(n.visibility = 'visible' OR n.created_by_user_uuid = ?)".into());
    binds.push(user.uuid.to_string());

    if !conds.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conds.join(" AND "));
    }
    sql.push_str(" ORDER BY n.updated_at DESC LIMIT ?");

    let mut q = sqlx::query_as::<_, NoteRow>(&sql);
    for b in &binds {
        q = q.bind(b);
    }
    q = q.bind(limit);
    let rows = q.fetch_all(&state.db).await?;

    let mut notes = Vec::with_capacity(rows.len());
    for row in rows {
        let note = row.into_dto()?;
        let tags = load_tags_for_note(&state.db, &note.uuid.to_string()).await?;
        notes.push(LoreNoteWithTags { note, tags });
    }
    Ok(Json(SearchResponse { notes }))
}

#[derive(sqlx::FromRow)]
struct NoteRow {
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

impl NoteRow {
    fn into_dto(self) -> Result<LoreNote, ApiError> {
        Ok(LoreNote {
            uuid: Uuid::parse_str(&self.uuid).map_err(|e| ApiError::Internal(e.into()))?,
            title: self.title,
            body_markdown: self.body_markdown,
            scope: NoteScope {
                kind: match self.scope_kind.as_str() {
                    "module" => NoteScopeKind::Module,
                    "setting" => NoteScopeKind::Setting,
                    "campaign" => NoteScopeKind::Campaign,
                    "character" => NoteScopeKind::Character,
                    other => {
                        return Err(ApiError::Internal(anyhow::anyhow!(
                            "unknown scope_kind: {other}"
                        )));
                    }
                },
                target_uuid: Uuid::parse_str(&self.scope_target_uuid)
                    .map_err(|e| ApiError::Internal(e.into()))?,
            },
            visibility: match self.visibility.as_str() {
                "visible" => NoteVisibility::Visible,
                "author_only" => NoteVisibility::AuthorOnly,
                "gamemaster_only" => NoteVisibility::GamemasterOnly,
                other => {
                    return Err(ApiError::Internal(anyhow::anyhow!(
                        "unknown visibility: {other}"
                    )));
                }
            },
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
