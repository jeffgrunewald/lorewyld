use axum::{
    Json,
    extract::{Query, State},
};
use chrono::{DateTime, Utc};
use lorewyld_types::{
    api_v1::CreateTagRequest,
    tag::Tag,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api::{ApiState, auth::CurrentUser, error::ApiError};

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

    rows.into_iter().map(TagRow::into_dto).collect::<Result<_, _>>().map(Json)
}

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

    let existing: Option<(String,)> = sqlx::query_as("SELECT uuid FROM tag WHERE slug = ?")
        .bind(&slug)
        .fetch_optional(&state.db)
        .await?;
    if existing.is_some() {
        return Err(ApiError::BadRequest(format!("tag slug '{slug}' already exists")));
    }

    let uuid = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO tag (uuid, slug, display_name, is_system) VALUES (?, ?, ?, 0)",
    )
    .bind(&uuid)
    .bind(&slug)
    .bind(&display_name)
    .execute(&state.db)
    .await?;

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
    let mut uuids = Vec::with_capacity(slugs.len());
    for raw_slug in slugs {
        let slug = raw_slug.trim().to_lowercase();
        if slug.is_empty() {
            continue;
        }
        let existing: Option<(String,)> = sqlx::query_as("SELECT uuid FROM tag WHERE slug = ?")
            .bind(&slug)
            .fetch_optional(db)
            .await?;
        let uuid = if let Some((uuid,)) = existing {
            uuid
        } else {
            let new_uuid = Uuid::new_v4().to_string();
            let display_name = slug.replace('-', " ");
            sqlx::query(
                "INSERT INTO tag (uuid, slug, display_name, is_system) VALUES (?, ?, ?, 0)",
            )
            .bind(&new_uuid)
            .bind(&slug)
            .bind(&display_name)
            .execute(db)
            .await?;
            new_uuid
        };
        if !uuids.contains(&uuid) {
            uuids.push(uuid);
        }
    }
    Ok(uuids)
}

/// Load all tags attached to a given lore note.
pub async fn load_tags_for_note(
    db: &SqlitePool,
    note_uuid: &str,
) -> Result<Vec<Tag>, ApiError> {
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
