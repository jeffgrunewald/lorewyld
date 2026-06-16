use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use lorewyld_types::{
    api_v1::{LoreNoteWithTags, PublishModuleRequest, PublishModuleResponse},
    content_module::ContentModule,
};
use uuid::Uuid;

use crate::api::{
    ApiState,
    auth::CurrentUser,
    error::{ApiError, is_unique_violation},
    rows::{
        ContentModuleRow, LORE_NOTE_SELECT, LoreNoteRow, MODULE_SELECT_ACTIVE, MODULE_SELECT_ONE,
    },
    tags::load_tags_for_notes,
};

/// Filter loose notes against viewer's visibility expectations. For
/// public/unauthenticated module-detail rendering we only show notes
/// with `Visible` visibility — `AuthorOnly`/`GamemasterOnly` notes that
/// somehow ended up in a Module scope (an authoring mistake the
/// Promote-to-Module wizard's review step should prevent) stay hidden.
fn note_is_publicly_visible(visibility: &str) -> bool {
    visibility == "visible"
}

/// `GET /api/modules` — list installed (active) content modules.
#[utoipa::path(
    get,
    path = "/api/modules",
    tag = "modules",
    responses((status = 200, description = "Active content modules on the server", body = [ContentModule]))
)]
pub async fn list_modules(
    State(state): State<ApiState>,
) -> Result<Json<Vec<ContentModule>>, ApiError> {
    let rows: Vec<ContentModuleRow> = sqlx::query_as(MODULE_SELECT_ACTIVE)
        .fetch_all(&state.db)
        .await?;
    rows.into_iter()
        .map(ContentModuleRow::into_dto)
        .collect::<Result<_, _>>()
        .map(Json)
}

/// `GET /api/modules/:uuid` — read a single module with its notes.
/// Public: published modules are by definition meant to be shareable.
#[utoipa::path(
    get,
    path = "/api/modules/{uuid}",
    tag = "modules",
    params(("uuid" = String, Path, description = "Module UUID")),
    responses(
        (status = 200, description = "The module plus its lore notes", body = ModuleWithNotes),
        (status = 404, description = "No such module"),
    )
)]
pub async fn get_module(
    State(state): State<ApiState>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<ModuleWithNotes>, ApiError> {
    let row: Option<ContentModuleRow> = sqlx::query_as(MODULE_SELECT_ONE)
        .bind(uuid.to_string())
        .fetch_optional(&state.db)
        .await?;
    let module = row.ok_or(ApiError::NotFound)?.into_dto()?;

    let sql = format!(
        "{LORE_NOTE_SELECT} WHERE scope_kind = 'module' AND scope_target_uuid = ? \
         ORDER BY updated_at DESC"
    );
    let note_rows: Vec<LoreNoteRow> = sqlx::query_as(&sql)
        .bind(uuid.to_string())
        .fetch_all(&state.db)
        .await?;

    let visible: Vec<LoreNoteRow> = note_rows
        .into_iter()
        .filter(|row| note_is_publicly_visible(&row.visibility))
        .collect();
    let note_uuids: Vec<String> = visible.iter().map(|r| r.uuid.clone()).collect();
    let mut tags_by_note = load_tags_for_notes(&state.db, &note_uuids).await?;
    let notes = visible
        .into_iter()
        .map(|row| {
            let note = row.into_dto()?;
            let tags = tags_by_note
                .remove(&note.uuid.to_string())
                .unwrap_or_default();
            Ok(LoreNoteWithTags { note, tags })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;
    Ok(Json(ModuleWithNotes { module, notes }))
}

/// `POST /api/modules` — Promote-to-Module commit. Snapshot-publishes
/// a `Setting`'s selected notes into a new `ContentModule` row.
#[utoipa::path(
    post,
    path = "/api/modules",
    tag = "modules",
    security(("bearer" = [])),
    request_body = PublishModuleRequest,
    responses((status = 201, description = "The newly published module", body = PublishModuleResponse))
)]
pub async fn publish_module(
    State(state): State<ApiState>,
    user: CurrentUser,
    Json(req): Json<PublishModuleRequest>,
) -> Result<(StatusCode, Json<PublishModuleResponse>), ApiError> {
    if req.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }
    let slug = req.slug.trim().to_lowercase();
    if slug.is_empty() {
        return Err(ApiError::BadRequest("slug is required".into()));
    }
    if req.version_string.trim().is_empty() {
        return Err(ApiError::BadRequest("version_string is required".into()));
    }

    // Verify ownership of source setting. A NULL owner (deleted account)
    // matches nobody — orphaned settings are unpublishable.
    let owner_row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT owner_user_uuid, published_as_module_uuid FROM setting WHERE uuid = ?",
    )
    .bind(req.source_setting_uuid.to_string())
    .fetch_optional(&state.db)
    .await?;
    let (owner_uuid, prev_published_uuid) = owner_row.ok_or(ApiError::NotFound)?;
    if owner_uuid.as_deref() != Some(user.uuid.to_string().as_str()) {
        return Err(ApiError::Forbidden);
    }

    // Slug-collision check.
    let slug_taken: Option<(String,)> =
        sqlx::query_as("SELECT uuid FROM content_module WHERE slug = ?")
            .bind(&slug)
            .fetch_optional(&state.db)
            .await?;
    if slug_taken.is_some() {
        return Err(ApiError::BadRequest(format!(
            "module slug '{slug}' is already taken"
        )));
    }

    // Pull selected notes (validate they belong to the source setting).
    let placeholders = vec!["?"; req.selected_note_uuids.len()].join(", ");
    let setting_uuid_str = req.source_setting_uuid.to_string();
    let selected_notes: Vec<SelectedNoteRow> = if req.selected_note_uuids.is_empty() {
        Vec::new()
    } else {
        let select_sql = format!(
            "SELECT uuid, title, body_markdown, visibility
               FROM lore_note
              WHERE scope_kind = 'setting'
                AND scope_target_uuid = ?
                AND uuid IN ({placeholders})"
        );
        let mut q = sqlx::query_as::<_, SelectedNoteRow>(&select_sql);
        q = q.bind(&setting_uuid_str);
        for u in &req.selected_note_uuids {
            q = q.bind(u.to_string());
        }
        q.fetch_all(&state.db).await?
    };
    if selected_notes.len() != req.selected_note_uuids.len() {
        return Err(ApiError::BadRequest(
            "one or more selected_note_uuids do not belong to the source setting".into(),
        ));
    }

    // Pre-resolve every source note's tag attachments in one query so we
    // can copy them inside the publish transaction without contending
    // for SQLite's write lock.
    let mut tag_uuids_per_note: HashMap<String, Vec<String>> = HashMap::new();
    if !selected_notes.is_empty() {
        let sql = format!(
            "SELECT lore_note_uuid, tag_uuid FROM tag_attachment_lore_note \
             WHERE lore_note_uuid IN ({placeholders})"
        );
        let mut q = sqlx::query_as::<_, (String, String)>(&sql);
        for note in &selected_notes {
            q = q.bind(&note.uuid);
        }
        for (note_uuid, tag_uuid) in q.fetch_all(&state.db).await? {
            tag_uuids_per_note
                .entry(note_uuid)
                .or_default()
                .push(tag_uuid);
        }
    }

    let module_uuid = Uuid::new_v4().to_string();
    // Module authorship is attributed by email, decoupled from server
    // user rows — published content survives account deletion. The
    // publisher is always credited.
    let authors = if req.authors.contains(&user.email) {
        req.authors.clone()
    } else {
        req.authors
            .iter()
            .cloned()
            .chain(std::iter::once(user.email.clone()))
            .collect()
    };
    let authors_json = serde_json::to_string(&authors).map_err(|e| ApiError::Internal(e.into()))?;

    let mut tx = state.db.begin().await?;

    // The earlier slug check gives the friendly message; UNIQUE(slug) is
    // the real guard, so a concurrent publish racing past the check maps
    // back to 400 here rather than surfacing as a 500.
    let inserted = sqlx::query(
        "INSERT INTO content_module (
            uuid, name, slug, license, license_url, schema_version,
            authors, description, version_string, previous_version_uuid,
            published_at, is_active, ordering, origin
         ) VALUES (?, ?, ?, ?, ?, 1, ?, ?, ?, ?, datetime('now'), 1, 0, 'published')",
    )
    .bind(&module_uuid)
    .bind(req.name.trim())
    .bind(&slug)
    .bind(req.license.wire_value())
    .bind(&req.license_url)
    .bind(&authors_json)
    .bind(&req.description)
    .bind(&req.version_string)
    .bind(prev_published_uuid.clone())
    .execute(&mut *tx)
    .await;
    match inserted {
        Ok(_) => {}
        Err(e) if is_unique_violation(&e) => {
            return Err(ApiError::BadRequest(format!(
                "module slug '{slug}' is already taken"
            )));
        }
        Err(e) => return Err(e.into()),
    }

    // Snapshot-copy each note into Module scope with derived_from linkage.
    for note in &selected_notes {
        let tag_uuids = tag_uuids_per_note
            .get(&note.uuid)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let new_note_uuid = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO lore_note (
                uuid, title, body_markdown, scope_kind, scope_target_uuid,
                visibility, derived_from_setting_note_uuid, created_by_user_uuid
             ) VALUES (?, ?, ?, 'module', ?, ?, ?, ?)",
        )
        .bind(&new_note_uuid)
        .bind(&note.title)
        .bind(&note.body_markdown)
        .bind(&module_uuid)
        .bind(&note.visibility)
        .bind(&note.uuid)
        .bind(user.uuid.to_string())
        .execute(&mut *tx)
        .await?;

        for tag_uuid in tag_uuids {
            sqlx::query(
                "INSERT INTO tag_attachment_lore_note (tag_uuid, lore_note_uuid)
                 VALUES (?, ?)",
            )
            .bind(tag_uuid)
            .bind(&new_note_uuid)
            .execute(&mut *tx)
            .await?;
        }
    }

    // Link source setting to the published module.
    sqlx::query(
        "UPDATE setting
            SET published_as_module_uuid = ?, updated_at = datetime('now')
          WHERE uuid = ?",
    )
    .bind(&module_uuid)
    .bind(req.source_setting_uuid.to_string())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let module_row: ContentModuleRow = sqlx::query_as(MODULE_SELECT_ONE)
        .bind(&module_uuid)
        .fetch_one(&state.db)
        .await?;
    let module = module_row.into_dto()?;

    Ok((
        StatusCode::CREATED,
        Json(PublishModuleResponse {
            module,
            note_count: selected_notes.len() as u32,
        }),
    ))
}

#[derive(sqlx::FromRow)]
struct SelectedNoteRow {
    uuid: String,
    title: String,
    body_markdown: String,
    visibility: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ModuleWithNotes {
    pub module: ContentModule,
    pub notes: Vec<LoreNoteWithTags>,
}
