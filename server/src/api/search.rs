use axum::{Json, extract::State};
use lorewyld_types::api_v1::{LoreNoteWithTags, SearchRequest, SearchResponse};

use crate::api::{
    ApiState,
    auth::CurrentUser,
    error::ApiError,
    rows::{LORE_NOTE_SELECT_N, LoreNoteRow, VISIBILITY_PREDICATE, scope_kind_to_str},
    tags::load_tags_for_notes,
};

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

    let mut sql = String::from(LORE_NOTE_SELECT_N);
    let mut binds: Vec<String> = Vec::new();
    let mut conds: Vec<String> = Vec::new();

    if let Some(q) = req.q.as_deref().filter(|s| !s.trim().is_empty()) {
        sql.push_str(" JOIN lore_note_fts fts ON fts.rowid = n.rowid");
        conds.push("lore_note_fts MATCH ?".into());
        binds.push(fts5_quote(q));
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
    conds.push(VISIBILITY_PREDICATE.into());
    binds.push(user.uuid.to_string());

    if !conds.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conds.join(" AND "));
    }
    sql.push_str(" ORDER BY n.updated_at DESC LIMIT ?");

    let mut q = sqlx::query_as::<_, LoreNoteRow>(&sql);
    for b in &binds {
        q = q.bind(b);
    }
    q = q.bind(limit);
    let rows = q.fetch_all(&state.db).await?;

    let note_uuids: Vec<String> = rows.iter().map(|r| r.uuid.clone()).collect();
    let mut tags_by_note = load_tags_for_notes(&state.db, &note_uuids).await?;
    let notes = rows
        .into_iter()
        .map(|row| {
            let note = row.into_dto()?;
            let tags = tags_by_note.remove(&note.uuid.to_string()).unwrap_or_default();
            Ok(LoreNoteWithTags { note, tags })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;
    Ok(Json(SearchResponse { notes }))
}

/// Quote each whitespace-separated term as an FTS5 string literal
/// (implicit AND between terms). Raw user input fed to MATCH otherwise
/// hits FTS5's query grammar — an unbalanced `"` or stray `NEAR(`
/// becomes a SQLite syntax error and surfaces as a 500.
fn fts5_quote(q: &str) -> String {
    q.split_whitespace()
        .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}
