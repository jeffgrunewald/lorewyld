use std::collections::{HashMap, HashSet};

use axum::{
    Json,
    extract::{Query, State},
};
use chrono::{DateTime, Utc};
use lorewyld_types::{api_v1::CreateTagRequest, tag::Tag};
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api::{
    ApiState,
    auth::CurrentUser,
    error::{ApiError, is_unique_violation},
};

#[derive(Debug, Deserialize)]
pub struct TagListQuery {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct TagRow {
    uuid: String,
    slug: String,
    display_name: String,
    is_system: i64,
    introduced_by_module_uuid: Option<String>,
    created_at: DateTime<Utc>,
}

impl TagRow {
    fn into_dto(self) -> Result<Tag, ApiError> {
        Ok(Tag {
            uuid: Uuid::parse_str(&self.uuid).map_err(|e| ApiError::Internal(e.into()))?,
            slug: self.slug,
            display_name: self.display_name,
            is_system: self.is_system != 0,
            introduced_by_module_uuid: self
                .introduced_by_module_uuid
                .as_deref()
                .map(|s| Uuid::parse_str(s).map_err(|e| ApiError::Internal(e.into())))
                .transpose()?,
            created_at: self.created_at,
        })
    }
}

#[utoipa::path(
    get,
    path = "/api/tags",
    tag = "tags",
    params(("q" = Option<String>, Query, description = "Slug/name prefix filter")),
    responses((status = 200, description = "Tags, optionally filtered", body = [Tag]))
)]
pub async fn list_tags(
    State(state): State<ApiState>,
    Query(query): Query<TagListQuery>,
) -> Result<Json<Vec<Tag>>, ApiError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let rows: Vec<TagRow> = if let Some(q) = query.q.as_deref().filter(|s| !s.is_empty()) {
        let pattern = format!("%{}%", q.to_lowercase());
        sqlx::query_as(
            "SELECT uuid, slug, display_name, is_system, introduced_by_module_uuid, created_at
               FROM tag
              WHERE lower(slug) LIKE ?1 OR lower(display_name) LIKE ?1
              ORDER BY is_system DESC, slug
              LIMIT ?2",
        )
        .bind(pattern)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            "SELECT uuid, slug, display_name, is_system, introduced_by_module_uuid, created_at
               FROM tag
              ORDER BY is_system DESC, slug
              LIMIT ?1",
        )
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    };

    rows.into_iter()
        .map(TagRow::into_dto)
        .collect::<Result<_, _>>()
        .map(Json)
}

#[utoipa::path(
    post,
    path = "/api/tags",
    tag = "tags",
    security(("bearer" = [])),
    request_body = CreateTagRequest,
    responses(
        (status = 200, description = "The created tag", body = Tag),
        (status = 409, description = "Slug already exists"),
    )
)]
pub async fn create_tag(
    State(state): State<ApiState>,
    _user: CurrentUser,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<Tag>, ApiError> {
    let slug = req.slug.trim().to_lowercase();
    if slug.is_empty() {
        return Err(ApiError::BadRequest("slug cannot be empty".into()));
    }
    let display_name = req.display_name.trim().to_string();
    if display_name.is_empty() {
        return Err(ApiError::BadRequest("display_name cannot be empty".into()));
    }

    // No pre-check: the UNIQUE(slug) constraint is the real guard. A
    // check-then-insert would race a concurrent create and 500 on the
    // constraint instead of returning 400.
    let uuid = Uuid::new_v4().to_string();
    let inserted =
        sqlx::query("INSERT INTO tag (uuid, slug, display_name, is_system) VALUES (?, ?, ?, 0)")
            .bind(&uuid)
            .bind(&slug)
            .bind(&display_name)
            .execute(&state.db)
            .await;
    match inserted {
        Ok(_) => {}
        Err(e) if is_unique_violation(&e) => {
            return Err(ApiError::BadRequest(format!(
                "tag slug '{slug}' already exists"
            )));
        }
        Err(e) => return Err(e.into()),
    }

    let row: TagRow = sqlx::query_as(
        "SELECT uuid, slug, display_name, is_system, introduced_by_module_uuid, created_at
           FROM tag WHERE uuid = ?",
    )
    .bind(&uuid)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(row.into_dto()?))
}

/// Resolve a list of tag slugs to tag UUIDs, creating any missing user
/// tags on the fly. Used during note create/update to absorb new tags
/// the user typed in the chip input without a separate roundtrip.
pub async fn resolve_or_create_tags(
    db: &SqlitePool,
    slugs: &[String],
) -> Result<Vec<String>, ApiError> {
    // Normalize + dedup, preserving the caller's order.
    let mut seen = HashSet::new();
    let cleaned: Vec<String> = slugs
        .iter()
        .map(|raw| raw.trim().to_lowercase())
        .filter(|slug| !slug.is_empty() && seen.insert(slug.clone()))
        .collect();
    if cleaned.is_empty() {
        return Ok(Vec::new());
    }

    // One query resolves every already-known slug.
    let placeholders = vec!["?"; cleaned.len()].join(", ");
    let sql = format!("SELECT slug, uuid FROM tag WHERE slug IN ({placeholders})");
    let mut query = sqlx::query_as::<_, (String, String)>(&sql);
    for slug in &cleaned {
        query = query.bind(slug);
    }
    let mut by_slug: HashMap<String, String> = query.fetch_all(db).await?.into_iter().collect();

    // Missing slugs: ON CONFLICT DO NOTHING + re-select, so a concurrent
    // request creating the same slug can't turn UNIQUE(slug) into a 500.
    let missing: Vec<String> = cleaned
        .iter()
        .filter(|s| !by_slug.contains_key(*s))
        .cloned()
        .collect();
    for slug in &missing {
        sqlx::query(
            "INSERT INTO tag (uuid, slug, display_name, is_system)
             VALUES (?, ?, ?, 0)
             ON CONFLICT(slug) DO NOTHING",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(slug)
        .bind(slug.replace('-', " "))
        .execute(db)
        .await?;
        let (uuid,): (String,) = sqlx::query_as("SELECT uuid FROM tag WHERE slug = ?")
            .bind(slug)
            .fetch_one(db)
            .await?;
        by_slug.insert(slug.clone(), uuid);
    }

    cleaned
        .iter()
        .map(|slug| {
            by_slug.get(slug).cloned().ok_or_else(|| {
                ApiError::Internal(anyhow::anyhow!("tag '{slug}' vanished during resolution"))
            })
        })
        .collect()
}

/// Load all tags attached to a given lore note.
pub async fn load_tags_for_note(db: &SqlitePool, note_uuid: &str) -> Result<Vec<Tag>, ApiError> {
    let rows: Vec<TagRow> = sqlx::query_as(
        "SELECT t.uuid, t.slug, t.display_name, t.is_system, t.introduced_by_module_uuid, t.created_at
           FROM tag t
           JOIN tag_attachment_lore_note tan ON tan.tag_uuid = t.uuid
          WHERE tan.lore_note_uuid = ?
          ORDER BY t.is_system DESC, t.slug",
    )
    .bind(note_uuid)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(TagRow::into_dto).collect()
}

#[derive(sqlx::FromRow)]
struct AttachedTagRow {
    lore_note_uuid: String,
    #[sqlx(flatten)]
    tag: TagRow,
}

/// Batch-load tags for many notes in one query (instead of one query per
/// note). Returns a map keyed by note uuid; notes without tags are absent.
pub async fn load_tags_for_notes(
    db: &SqlitePool,
    note_uuids: &[String],
) -> Result<HashMap<String, Vec<Tag>>, ApiError> {
    if note_uuids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = vec!["?"; note_uuids.len()].join(", ");
    let sql = format!(
        "SELECT tan.lore_note_uuid, t.uuid, t.slug, t.display_name, t.is_system,
                t.introduced_by_module_uuid, t.created_at
           FROM tag t
           JOIN tag_attachment_lore_note tan ON tan.tag_uuid = t.uuid
          WHERE tan.lore_note_uuid IN ({placeholders})
          ORDER BY t.is_system DESC, t.slug"
    );
    let mut query = sqlx::query_as::<_, AttachedTagRow>(&sql);
    for uuid in note_uuids {
        query = query.bind(uuid);
    }
    let rows = query.fetch_all(db).await?;

    let mut by_note: HashMap<String, Vec<Tag>> = HashMap::new();
    for row in rows {
        by_note
            .entry(row.lore_note_uuid)
            .or_default()
            .push(row.tag.into_dto()?);
    }
    Ok(by_note)
}
