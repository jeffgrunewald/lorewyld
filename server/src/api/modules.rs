use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use lorewyld_types::{
    ModuleOrigin,
    api_v1::{
        CreateModuleRequest, InstallModuleResponse, LoreNoteWithTags, PublishModuleRequest,
        PublishModuleResponse, UpdateModuleRequest,
    },
    content_module::ContentModule,
};
use uuid::Uuid;

use crate::{
    api::{
        ApiState,
        auth::CurrentUser,
        error::{ApiError, is_unique_violation},
        rows::{
            ContentModuleRow, LORE_NOTE_SELECT, LoreNoteRow, MODULE_SELECT_ACTIVE,
            MODULE_SELECT_ONE,
        },
        tags::load_tags_for_notes,
    },
    content::{
        self, HOMEBREW_MODULE_SLUG, ImportError, ImportOptions, NewCustomModule,
        PINNED_MODULE_SLUG, RecordError, SlugConflict, import_bundle,
    },
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

/// `GET /api/modules/editable` — the `local` (homebrew) modules content
/// can be authored into or reassigned between. Any authenticated member
/// sees them (local content is server-shared); the compendium uses this
/// to decide which records show Edit/Delete and to populate the
/// "move to module" picker.
#[utoipa::path(
    get,
    path = "/api/modules/editable",
    tag = "modules",
    operation_id = "list_editable_modules",
    security(("bearer" = [])),
    responses((status = 200, description = "Active homebrew (local) modules", body = [ContentModule]))
)]
pub async fn list_editable_modules(
    State(state): State<ApiState>,
    _user: CurrentUser,
) -> Result<Json<Vec<ContentModule>>, ApiError> {
    let rows: Vec<ContentModuleRow> = sqlx::query_as(MODULE_SELECT_ACTIVE)
        .fetch_all(&state.db)
        .await?;
    rows.into_iter()
        .filter(|r| r.origin_kind() == ModuleOrigin::Local)
        .map(ContentModuleRow::into_dto)
        .collect::<Result<_, _>>()
        .map(Json)
}

/// `POST /api/modules/custom` — create a homebrew (`local`) module any
/// authenticated member can author content into.
#[utoipa::path(
    post,
    path = "/api/modules/custom",
    tag = "modules",
    operation_id = "create_custom_module",
    security(("bearer" = [])),
    request_body = CreateModuleRequest,
    responses(
        (status = 201, description = "The created local module", body = ContentModule),
        (status = 400, description = "Empty/duplicate slug"),
    )
)]
pub async fn create_custom_module(
    State(state): State<ApiState>,
    user: CurrentUser,
    Json(req): Json<CreateModuleRequest>,
) -> Result<(StatusCode, Json<ContentModule>), ApiError> {
    if req.name.trim().is_empty() {
        return Err(ApiError::BadRequest("module name is required".into()));
    }
    let module = content::create_custom_module(
        &state.db,
        NewCustomModule {
            name: req.name.trim().to_string(),
            slug: req.slug,
            license: req.license,
            license_url: req.license_url,
            description: req.description,
            authors: req.authors,
            website_url: req.website_url,
            created_by: user.uuid.to_string(),
        },
    )
    .await
    .map_err(record_error)?;
    Ok((StatusCode::CREATED, Json(module)))
}

/// `PATCH /api/modules/{uuid}` — edit a `local` module's metadata or
/// toggle its active state. Creator-or-admin only; only `local` modules
/// are user-editable (bundled/uploaded/published metadata is fixed; use
/// the admin status endpoint to disable those).
#[utoipa::path(
    patch,
    path = "/api/modules/{uuid}",
    tag = "modules",
    operation_id = "update_custom_module",
    security(("bearer" = [])),
    params(("uuid" = String, Path, description = "Module UUID")),
    request_body = UpdateModuleRequest,
    responses(
        (status = 200, description = "Updated module", body = ContentModule),
        (status = 403, description = "Not the creator/admin, or not a local module"),
        (status = 404, description = "No such module"),
    )
)]
pub async fn update_custom_module(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path(uuid): Path<Uuid>,
    Json(req): Json<UpdateModuleRequest>,
) -> Result<Json<ContentModule>, ApiError> {
    let meta = load_module_meta(&state.db, &uuid.to_string())
        .await?
        .ok_or(ApiError::NotFound)?;
    if meta.origin != ModuleOrigin::Local {
        return Err(ApiError::Forbidden);
    }
    if !meta.is_creator(&user) {
        return Err(ApiError::Forbidden);
    }

    let authors_json = match &req.authors {
        Some(a) => Some(serde_json::to_string(a).map_err(|e| ApiError::Internal(e.into()))?),
        None => None,
    };
    sqlx::query(
        "UPDATE content_module SET \
            name        = COALESCE(?, name), \
            license     = COALESCE(?, license), \
            license_url = COALESCE(?, license_url), \
            description = COALESCE(?, description), \
            authors     = COALESCE(?, authors), \
            website_url = COALESCE(?, website_url), \
            is_active   = COALESCE(?, is_active), \
            updated_at  = datetime('now') \
          WHERE uuid = ?",
    )
    .bind(req.name.as_deref().map(str::trim))
    .bind(req.license.map(|l| l.wire_value()))
    .bind(req.license_url.as_deref())
    .bind(req.description.as_deref())
    .bind(authors_json)
    .bind(req.website_url.as_deref())
    .bind(req.is_active)
    .bind(uuid.to_string())
    .execute(&state.db)
    .await?;

    let row: ContentModuleRow = sqlx::query_as(MODULE_SELECT_ONE)
        .bind(uuid.to_string())
        .fetch_one(&state.db)
        .await?;
    row.into_dto().map(Json)
}

/// `DELETE /api/modules/{uuid}` — uninstall a `local` module (creator or
/// admin) or an `uploaded`/`published` module (admin). Bundled modules and
/// the pinned SRD/Homebrew modules cannot be deleted.
#[utoipa::path(
    delete,
    path = "/api/modules/{uuid}",
    tag = "modules",
    operation_id = "delete_custom_module",
    security(("bearer" = [])),
    params(("uuid" = String, Path, description = "Module UUID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 403, description = "Not permitted (or a protected module)"),
        (status = 404, description = "No such module"),
    )
)]
pub async fn delete_custom_module(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path(uuid): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let uuid_str = uuid.to_string();
    let meta = load_module_meta(&state.db, &uuid_str)
        .await?
        .ok_or(ApiError::NotFound)?;
    if meta.slug == PINNED_MODULE_SLUG || meta.slug == HOMEBREW_MODULE_SLUG {
        return Err(ApiError::BadRequest(
            "this module is pinned and cannot be uninstalled".into(),
        ));
    }
    let permitted = match meta.origin {
        ModuleOrigin::Local => meta.is_creator(&user),
        ModuleOrigin::Uploaded | ModuleOrigin::Published => user.admin,
        ModuleOrigin::Bundled => {
            return Err(ApiError::BadRequest(
                "bundled modules cannot be uninstalled; disable the module instead".into(),
            ));
        }
    };
    if !permitted {
        return Err(ApiError::Forbidden);
    }
    content::remove_module(&state.db, &uuid_str)
        .await
        .map_err(map_fk_conflict)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/modules/{uuid}/export` — download a module as a self-contained
/// `.lorebundle` (a `ContentBundle` JSON) that re-imports on any instance.
#[utoipa::path(
    get,
    path = "/api/modules/{uuid}/export",
    tag = "modules",
    operation_id = "export_module",
    security(("bearer" = [])),
    params(("uuid" = String, Path, description = "Module UUID")),
    responses(
        (status = 200, description = "A ContentBundle JSON download"),
        (status = 404, description = "No such module"),
    )
)]
pub async fn export_module(
    State(state): State<ApiState>,
    _user: CurrentUser,
    Path(uuid): Path<Uuid>,
) -> Result<Response, ApiError> {
    let row: Option<ContentModuleRow> = sqlx::query_as(MODULE_SELECT_ONE)
        .bind(uuid.to_string())
        .fetch_optional(&state.db)
        .await?;
    let module = row.ok_or(ApiError::NotFound)?.into_dto()?;
    let slug = module.slug.clone();
    let bundle = content::export_module(&state.db, module).await?;
    let body = serde_json::to_string(&bundle).map_err(|e| ApiError::Internal(e.into()))?;
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/json".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{slug}.lorebundle\""),
            ),
        ],
        body,
    )
        .into_response())
}

/// `POST /api/modules/import` — install a `.lorebundle` (ContentBundle) as
/// an `uploaded` module. Available to any authenticated member; slug
/// collisions reject the whole package.
#[utoipa::path(
    post,
    path = "/api/modules/import",
    tag = "modules",
    operation_id = "import_module",
    security(("bearer" = [])),
    request_body(content = String, description = "A complete ContentBundle JSON package"),
    responses(
        (status = 201, description = "Modules installed", body = InstallModuleResponse),
        (status = 400, description = "Schema/slug/license problem"),
    )
)]
pub async fn import_module(
    State(state): State<ApiState>,
    _user: CurrentUser,
    Json(bundle): Json<lorewyld_types::ContentBundle>,
) -> Result<(StatusCode, Json<InstallModuleResponse>), ApiError> {
    if bundle.modules.is_empty() {
        return Err(ApiError::BadRequest("bundle contains no modules".into()));
    }
    let outcome = import_bundle(
        &state.db,
        &bundle,
        &ImportOptions {
            origin: ModuleOrigin::Uploaded,
            on_slug_conflict: SlugConflict::Reject,
            require_bundling_license: false,
        },
    )
    .await
    .map_err(|e| match e {
        ImportError::Db(err) => ApiError::Internal(err),
        other => ApiError::BadRequest(other.to_string()),
    })?;
    Ok((
        StatusCode::CREATED,
        Json(InstallModuleResponse {
            installed: outcome.installed,
            record_count: outcome.record_count as u32,
        }),
    ))
}

/// Gate-relevant module columns.
struct ModuleMeta {
    origin: ModuleOrigin,
    created_by: Option<String>,
    slug: String,
}

impl ModuleMeta {
    fn is_creator(&self, user: &CurrentUser) -> bool {
        user.admin || self.created_by.as_deref() == Some(user.uuid.to_string().as_str())
    }
}

async fn load_module_meta(
    db: &sqlx::SqlitePool,
    uuid: &str,
) -> Result<Option<ModuleMeta>, ApiError> {
    let row: Option<(String, Option<String>, String)> = sqlx::query_as(
        "SELECT origin, created_by_user_uuid, slug FROM content_module WHERE uuid = ?",
    )
    .bind(uuid)
    .fetch_optional(db)
    .await?;
    Ok(row.map(|(origin, created_by, slug)| ModuleMeta {
        origin: crate::content::module_origin_from_str(&origin).unwrap_or(ModuleOrigin::Uploaded),
        created_by,
        slug,
    }))
}

/// Cross-module references surface as FK violations on uninstall — a
/// client problem, not a server bug.
fn map_fk_conflict(err: sqlx::Error) -> ApiError {
    if matches!(&err, sqlx::Error::Database(db) if db.is_foreign_key_violation()) {
        ApiError::BadRequest(
            "module content is referenced by other installed modules; uninstall those first".into(),
        )
    } else {
        err.into()
    }
}

fn record_error(e: RecordError) -> ApiError {
    match e {
        RecordError::Invalid(m) => ApiError::BadRequest(m),
        RecordError::Db(err) => ApiError::Internal(err),
    }
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
