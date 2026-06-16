//! Admin module lifecycle: install (upload), disable/reinstall, and
//! uninstall. Module state is server-wide — disabling hides a module's
//! content from every user — so everything here is [`AdminUser`]-gated.
//!
//! Bundled modules (seeded from the embedded content bundle) can only
//! be disabled: the boot seeder re-adds missing bundle slugs, so a
//! deleted bundled module would silently resurrect on restart.

use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use lorewyld_types::{
    ContentBundle, ModuleOrigin,
    api_v1::{AdminModuleSummary, CategoryCount, InstallModuleResponse, UpdateModuleStatusRequest},
};
use uuid::Uuid;

use crate::{
    api::{
        ApiState,
        auth::AdminUser,
        error::ApiError,
        rows::{ContentModuleRow, MODULE_SELECT_ALL, MODULE_SELECT_ONE},
    },
    content::{CATEGORIES, ImportError, ImportOptions, SlugConflict, import_bundle},
};

/// `GET /api/admin/modules` — every module on the server (active or
/// not) with provenance and record counts for the management UI.
#[utoipa::path(
    get,
    path = "/api/admin/modules",
    tag = "admin",
    operation_id = "admin_list_modules",
    security(("bearer" = [])),
    responses((status = 200, description = "Every module (active or not) with provenance + record counts (admin only)", body = [AdminModuleSummary]))
)]
pub async fn list_modules(
    State(state): State<ApiState>,
    _admin: AdminUser,
) -> Result<Json<Vec<AdminModuleSummary>>, ApiError> {
    let rows: Vec<ContentModuleRow> = sqlx::query_as(MODULE_SELECT_ALL)
        .fetch_all(&state.db)
        .await?;

    // One UNION-ALL pass over the content tables, grouped per module.
    let count_sql = CATEGORIES
        .iter()
        .map(|spec| {
            format!(
                "SELECT '{t}' AS category, content_module_uuid, COUNT(*) AS n \
                 FROM {t} GROUP BY content_module_uuid",
                t = spec.table
            )
        })
        .collect::<Vec<_>>()
        .join(" UNION ALL ");
    let count_rows: Vec<(String, String, u32)> =
        sqlx::query_as(&count_sql).fetch_all(&state.db).await?;
    let mut counts_by_module: HashMap<String, Vec<CategoryCount>> = HashMap::new();
    for (category, module_uuid, n) in count_rows {
        counts_by_module
            .entry(module_uuid)
            .or_default()
            .push(CategoryCount { category, count: n });
    }

    let note_rows: Vec<(String, u32)> = sqlx::query_as(
        "SELECT scope_target_uuid, COUNT(*) FROM lore_note \
         WHERE scope_kind = 'module' GROUP BY scope_target_uuid",
    )
    .fetch_all(&state.db)
    .await?;
    let notes_by_module: HashMap<String, u32> = note_rows.into_iter().collect();

    rows.into_iter()
        .map(|row| {
            let origin = row.origin_kind();
            let record_counts = counts_by_module.remove(&row.uuid).unwrap_or_default();
            let lore_note_count = notes_by_module.get(&row.uuid).copied().unwrap_or(0);
            Ok(AdminModuleSummary {
                module: row.into_dto()?,
                origin,
                record_counts,
                lore_note_count,
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()
        .map(Json)
}

/// `POST /api/admin/modules/install` — install a `ContentBundle`
/// package. The request body is the bundle file itself; slug
/// collisions with installed modules reject the whole package.
#[utoipa::path(
    post,
    path = "/api/admin/modules/install",
    tag = "admin",
    security(("bearer" = [])),
    request_body(content = String, description = "A complete ContentBundle JSON package (schema omitted here — it embeds the full content type graph)"),
    responses((status = 201, description = "Modules installed (admin only)", body = InstallModuleResponse))
)]
pub async fn install_module(
    State(state): State<ApiState>,
    _admin: AdminUser,
    Json(bundle): Json<ContentBundle>,
) -> Result<(StatusCode, Json<InstallModuleResponse>), ApiError> {
    if bundle.modules.is_empty() {
        return Err(ApiError::BadRequest(
            "bundle contains no modules".to_string(),
        ));
    }
    let outcome = import_bundle(
        &state.db,
        &bundle,
        &ImportOptions {
            origin: ModuleOrigin::Uploaded,
            on_slug_conflict: SlugConflict::Reject,
            // Homebrew uploads may be unlicensed; only the embedded
            // bundle is held to the bundling-license bar.
            require_bundling_license: false,
        },
    )
    .await
    .map_err(|e| match e {
        ImportError::Db(err) => ApiError::Internal(err),
        other => ApiError::BadRequest(other.to_string()),
    })?;

    tracing::info!(
        modules = outcome.installed.len(),
        records = outcome.record_count,
        "installed uploaded content bundle"
    );
    Ok((
        StatusCode::CREATED,
        Json(InstallModuleResponse {
            installed: outcome.installed,
            record_count: outcome.record_count as u32,
        }),
    ))
}

/// `PATCH /api/admin/modules/:uuid` — disable (`is_active: false`) or
/// reinstall/activate (`true`). Disabled content stays in the database
/// but is excluded from every content read. The pinned SRD module can
/// never be disabled — every other module references its shared rules
/// vocabulary.
#[utoipa::path(
    patch,
    path = "/api/admin/modules/{uuid}",
    tag = "admin",
    security(("bearer" = [])),
    params(("uuid" = String, Path, description = "Module UUID")),
    request_body = UpdateModuleStatusRequest,
    responses((status = 200, description = "Updated module (admin only)", body = lorewyld_types::ContentModule))
)]
pub async fn update_module_status(
    State(state): State<ApiState>,
    _admin: AdminUser,
    Path(uuid): Path<Uuid>,
    Json(req): Json<UpdateModuleStatusRequest>,
) -> Result<Json<lorewyld_types::ContentModule>, ApiError> {
    if !req.is_active {
        let slug: Option<(String,)> =
            sqlx::query_as("SELECT slug FROM content_module WHERE uuid = ?")
                .bind(uuid.to_string())
                .fetch_optional(&state.db)
                .await?;
        let (slug,) = slug.ok_or(ApiError::NotFound)?;
        if slug == crate::content::PINNED_MODULE_SLUG {
            return Err(ApiError::BadRequest(
                "the SRD module provides the shared rules vocabulary every other module \
                 references and cannot be disabled"
                    .to_string(),
            ));
        }
    }

    let updated = sqlx::query(
        "UPDATE content_module \
            SET is_active = ?, updated_at = datetime('now') \
          WHERE uuid = ?",
    )
    .bind(req.is_active)
    .bind(uuid.to_string())
    .execute(&state.db)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    let row: ContentModuleRow = sqlx::query_as(MODULE_SELECT_ONE)
        .bind(uuid.to_string())
        .fetch_one(&state.db)
        .await?;
    row.into_dto().map(Json)
}

/// `DELETE /api/admin/modules/:uuid` — full uninstall: removes the
/// module row, all its content records, and its module-scope lore
/// notes. Bundled modules are rejected with 400 (disable instead).
#[utoipa::path(
    delete,
    path = "/api/admin/modules/{uuid}",
    tag = "admin",
    security(("bearer" = [])),
    params(("uuid" = String, Path, description = "Module UUID")),
    responses((status = 204, description = "Module uninstalled; bundled modules can only be disabled (admin only)"))
)]
pub async fn uninstall_module(
    State(state): State<ApiState>,
    _admin: AdminUser,
    Path(uuid): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let uuid_str = uuid.to_string();
    let row: Option<(String,)> = sqlx::query_as("SELECT origin FROM content_module WHERE uuid = ?")
        .bind(&uuid_str)
        .fetch_optional(&state.db)
        .await?;
    let (origin,) = row.ok_or(ApiError::NotFound)?;
    if origin == "bundled" {
        return Err(ApiError::BadRequest(
            "bundled modules cannot be uninstalled; disable the module instead".to_string(),
        ));
    }

    crate::content::remove_module(&state.db, &uuid_str)
        .await
        .map_err(map_fk_conflict)?;
    tracing::info!(module = %uuid_str, "uninstalled content module");
    Ok(StatusCode::NO_CONTENT)
}

/// Cross-module references (another installed module's records point
/// at this module's lookup rows) surface as FK violations — a client
/// problem, not a server bug.
fn map_fk_conflict(err: sqlx::Error) -> ApiError {
    if matches!(&err, sqlx::Error::Database(db) if db.is_foreign_key_violation()) {
        ApiError::BadRequest(
            "module content is referenced by other installed modules; uninstall those first"
                .to_string(),
        )
    } else {
        err.into()
    }
}
