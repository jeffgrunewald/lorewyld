//! Homebrew content authoring: create / edit / delete / reassign
//! user-authored content records.
//!
//! Validation is single-sourced in `lorewyld-domain` (the same
//! [`validate_value`](lorewyld_domain::validate_value) the web and mobile
//! clients run pre-submit) and the record is then assembled from the
//! category's [`default_record`](lorewyld_domain::default_record) skeleton
//! plus the authorable input, with the server stamping identity,
//! provenance, and timestamps. Records may be created in, and reassigned
//! between, `local` (homebrew) modules only — never into or out of the
//! read-only bundled/uploaded/published modules.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use lorewyld_types::{
    ModuleOrigin,
    api_v1::{CreateContentRequest, UpdateContentRequest},
};
use serde_json::Value;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    api::{ApiState, auth::CurrentUser, error::ApiError},
    content::{self, RecordError, slugify},
};

/// `POST /api/content/{category}` — author a homebrew content record.
#[utoipa::path(
    post,
    path = "/api/content/{category}",
    tag = "content",
    operation_id = "create_content",
    security(("bearer" = [])),
    params(("category" = String, Path, description = "Content category, e.g. spell, creature, item")),
    request_body = CreateContentRequest,
    responses(
        (status = 201, description = "The created record's full stored JSON"),
        (status = 404, description = "Category is not user-authorable"),
        (status = 422, description = "Field validation failed"),
    )
)]
pub async fn create_content(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path(category): Path<String>,
    Json(req): Json<CreateContentRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let schema = lorewyld_domain::field_schema(&category).ok_or(ApiError::NotFound)?;
    lorewyld_domain::validate_value(&schema, &req.fields).map_err(ApiError::Validation)?;

    // Resolve the target local module + its attribution document.
    let (module_uuid, document_uuid) = match req.module_uuid {
        Some(target) => resolve_editable_module(&state.db, &target.to_string()).await?,
        None => {
            let hb = content::ensure_homebrew_module(&state.db).await?;
            (hb.module_uuid, hb.document_uuid)
        }
    };

    let record = assemble_record(&category, &req.fields, &module_uuid, &document_uuid, None)?;
    let uuid = json_str(&record, "uuid")?;

    content::upsert_content_record(&state.db, &category, record)
        .await
        .map_err(record_error)?;
    set_author(&state.db, &category, &uuid, &user.uuid.to_string()).await?;

    let stored = fetch_record(&state.db, &category, &uuid)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok((StatusCode::CREATED, Json(stored)))
}

/// `PATCH /api/content/{category}/{uuid}` — edit fields and/or reassign
/// the record's module. Creator-or-admin only; the record (and any
/// reassignment target) must be in a `local` module.
#[utoipa::path(
    patch,
    path = "/api/content/{category}/{uuid}",
    tag = "content",
    operation_id = "update_content",
    security(("bearer" = [])),
    params(
        ("category" = String, Path, description = "Content category"),
        ("uuid" = String, Path, description = "Record UUID"),
    ),
    request_body = UpdateContentRequest,
    responses(
        (status = 200, description = "The updated record's full stored JSON"),
        (status = 403, description = "Not the creator/admin, or the record is read-only reference content"),
        (status = 404, description = "No such record"),
        (status = 422, description = "Field validation failed"),
    )
)]
pub async fn update_content(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path((category, uuid)): Path<(String, Uuid)>,
    Json(req): Json<UpdateContentRequest>,
) -> Result<Json<Value>, ApiError> {
    let schema = lorewyld_domain::field_schema(&category).ok_or(ApiError::NotFound)?;
    let uuid = uuid.to_string();

    let existing = load_authored_row(&state.db, &category, &uuid)
        .await?
        .ok_or(ApiError::NotFound)?;
    authorize_edit(&state.db, &user, &existing).await?;

    // Where the record will live after this edit (reassignment honours
    // the editable-source-AND-target rule).
    let (module_uuid, document_uuid) = match &req.module_uuid {
        Some(target) => resolve_editable_module(&state.db, &target.to_string()).await?,
        None => (
            existing.module_uuid.clone(),
            record_document(&existing.data),
        ),
    };

    let mut record: Value =
        serde_json::from_str(&existing.data).map_err(|e| ApiError::Internal(e.into()))?;
    // The record's immutable identity comes from the loaded row, never the
    // client: `merge_fields` ignores server-managed keys, and we re-pin the
    // path uuid + original key here so a forged `fields.uuid` can never
    // redirect the upsert's DELETE at another record (IDOR).
    let original_key = record
        .get("key")
        .and_then(Value::as_str)
        .map(str::to_string);
    if let Some(fields) = &req.fields {
        merge_fields(&mut record, fields)?;
    }
    set_managed(
        &mut record,
        &module_uuid,
        &document_uuid,
        Some(&uuid),
        original_key.as_deref(),
    );
    // Validate the authorable subset of the merged record.
    lorewyld_domain::validate_value(&schema, &record).map_err(ApiError::Validation)?;

    content::upsert_content_record(&state.db, &category, record)
        .await
        .map_err(record_error)?;
    // The upsert rebuilt the row, so restore the original authorship.
    if let Some(creator) = &existing.created_by {
        set_author(&state.db, &category, &uuid, creator).await?;
    }

    let stored = fetch_record(&state.db, &category, &uuid)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(stored))
}

/// `DELETE /api/content/{category}/{uuid}` — remove a homebrew record.
/// Creator-or-admin only; the record must be in a `local` module.
#[utoipa::path(
    delete,
    path = "/api/content/{category}/{uuid}",
    tag = "content",
    operation_id = "delete_content",
    security(("bearer" = [])),
    params(
        ("category" = String, Path, description = "Content category"),
        ("uuid" = String, Path, description = "Record UUID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 403, description = "Not the creator/admin, or read-only reference content"),
        (status = 404, description = "No such record"),
    )
)]
pub async fn delete_content(
    State(state): State<ApiState>,
    user: CurrentUser,
    Path((category, uuid)): Path<(String, Uuid)>,
) -> Result<StatusCode, ApiError> {
    lorewyld_domain::field_schema(&category).ok_or(ApiError::NotFound)?;
    let uuid = uuid.to_string();
    let existing = load_authored_row(&state.db, &category, &uuid)
        .await?
        .ok_or(ApiError::NotFound)?;
    authorize_edit(&state.db, &user, &existing).await?;

    let removed = content::delete_content_record(&state.db, &category, &uuid)
        .await
        .map_err(record_error)?;
    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

// ─── helpers ────────────────────────────────────────────────────────────

/// An authored row's gate-relevant columns.
struct AuthoredRow {
    data: String,
    module_uuid: String,
    created_by: Option<String>,
}

async fn load_authored_row(
    db: &SqlitePool,
    category: &str,
    uuid: &str,
) -> Result<Option<AuthoredRow>, ApiError> {
    // `category` is validated by the caller via field_schema before this
    // runs, so it names a real authorable table.
    let sql = format!(
        "SELECT data, content_module_uuid, created_by_user_uuid FROM {category} WHERE uuid = ?"
    );
    let row: Option<(String, String, Option<String>)> =
        sqlx::query_as(&sql).bind(uuid).fetch_optional(db).await?;
    Ok(row.map(|(data, module_uuid, created_by)| AuthoredRow {
        data,
        module_uuid,
        created_by,
    }))
}

/// Confirms the caller may edit/delete: they created the record (or are
/// an admin) AND the record lives in an editable `local` module.
async fn authorize_edit(
    db: &SqlitePool,
    user: &CurrentUser,
    row: &AuthoredRow,
) -> Result<(), ApiError> {
    let is_creator = row.created_by.as_deref() == Some(user.uuid.to_string().as_str());
    if !is_creator && !user.admin {
        return Err(ApiError::Forbidden);
    }
    match content::module_origin(db, &row.module_uuid).await? {
        Some(ModuleOrigin::Local) => Ok(()),
        _ => Err(ApiError::Forbidden),
    }
}

/// Resolves a target module uuid for authoring, returning its uuid + its
/// attribution-document uuid. Errors unless the module exists and is
/// editable (`local`).
async fn resolve_editable_module(
    db: &SqlitePool,
    module_uuid: &str,
) -> Result<(String, String), ApiError> {
    match content::module_origin(db, module_uuid).await? {
        Some(ModuleOrigin::Local) => {}
        Some(_) => {
            return Err(ApiError::Forbidden);
        }
        None => return Err(ApiError::NotFound),
    }
    let name: (String,) = sqlx::query_as("SELECT name FROM content_module WHERE uuid = ?")
        .bind(module_uuid)
        .fetch_one(db)
        .await?;
    let document_uuid = content::ensure_module_document(db, module_uuid, &name.0).await?;
    Ok((module_uuid.to_string(), document_uuid))
}

/// Builds a full record from the category skeleton + authorable input,
/// then stamps server-managed fields. `existing_uuid`/`existing_key` are
/// `None` on create (a fresh uuid+key are minted) — the edit path keeps
/// them via [`set_managed`].
fn assemble_record(
    category: &str,
    fields: &Value,
    module_uuid: &str,
    document_uuid: &str,
    existing_uuid: Option<&str>,
) -> Result<Value, ApiError> {
    let mut record = lorewyld_domain::default_record(category).ok_or(ApiError::NotFound)?;
    merge_fields(&mut record, fields)?;

    let uuid = existing_uuid
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let key = format!("{category}-{uuid}");
    set_managed(
        &mut record,
        module_uuid,
        document_uuid,
        Some(&uuid),
        Some(&key),
    );
    // Stamp a real creation time (the skeleton carries an epoch
    // placeholder, and `merge_fields` strips any client-sent created_at).
    if let Some(obj) = record.as_object_mut() {
        obj.insert(
            "created_at".into(),
            Value::String(chrono::Utc::now().to_rfc3339()),
        );
    }
    Ok(record)
}

/// Server-managed fields the client may never set through `fields`:
/// identity, provenance, slug (derived), and audit columns. Skipping them
/// in [`merge_fields`] is what prevents a forged `uuid`/`key`/module from
/// riding in on otherwise-legitimate authoring input.
const MANAGED_KEYS: &[&str] = &[
    "uuid",
    "key",
    "slug",
    "content_module_uuid",
    "document_uuid",
    "created_at",
    "updated_at",
    "created_by_user_uuid",
];

/// Overlays the authorable `fields` object onto a record, ignoring any
/// server-managed keys the client tried to set. Rejects a non-object
/// payload.
fn merge_fields(record: &mut Value, fields: &Value) -> Result<(), ApiError> {
    let (Some(obj), Some(incoming)) = (record.as_object_mut(), fields.as_object()) else {
        return Err(ApiError::BadRequest("fields must be a JSON object".into()));
    };
    for (k, v) in incoming {
        if MANAGED_KEYS.contains(&k.as_str()) {
            continue;
        }
        obj.insert(k.clone(), v.clone());
    }
    Ok(())
}

/// Stamps the server-managed fields. `document_uuid` is only set when the
/// record type carries one (lookups like condition/language don't).
/// `uuid`/`key` are set only when provided (create path).
fn set_managed(
    record: &mut Value,
    module_uuid: &str,
    document_uuid: &str,
    uuid: Option<&str>,
    key: Option<&str>,
) {
    let Some(obj) = record.as_object_mut() else {
        return;
    };
    obj.insert(
        "content_module_uuid".into(),
        Value::String(module_uuid.to_string()),
    );
    if obj.contains_key("document_uuid") {
        obj.insert(
            "document_uuid".into(),
            Value::String(document_uuid.to_string()),
        );
    }
    if let Some(uuid) = uuid {
        obj.insert("uuid".into(), Value::String(uuid.to_string()));
    }
    if let Some(key) = key {
        obj.insert("key".into(), Value::String(key.to_string()));
    }
    // Derive the slug from the (possibly just-merged) name.
    if let Some(name) = obj.get("name").and_then(Value::as_str) {
        let slug = slugify(name);
        let slug = if slug.is_empty() {
            obj.get("uuid")
                .and_then(Value::as_str)
                .unwrap_or("item")
                .to_string()
        } else {
            slug
        };
        obj.insert("slug".into(), Value::String(slug));
    }
    // Bump the edit timestamp (the LWW basis for content sync).
    let now = chrono::Utc::now().to_rfc3339();
    obj.insert("updated_at".into(), Value::String(now));
}

/// The record's current `document_uuid` (used when an edit doesn't
/// reassign the module), defaulting to nil if absent.
fn record_document(data: &str) -> String {
    serde_json::from_str::<Value>(data)
        .ok()
        .and_then(|v| {
            v.get("document_uuid")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| Uuid::nil().to_string())
}

async fn set_author(
    db: &SqlitePool,
    category: &str,
    uuid: &str,
    user_uuid: &str,
) -> Result<(), ApiError> {
    let sql = format!("UPDATE {category} SET created_by_user_uuid = ? WHERE uuid = ?");
    sqlx::query(&sql)
        .bind(user_uuid)
        .bind(uuid)
        .execute(db)
        .await?;
    Ok(())
}

async fn fetch_record(
    db: &SqlitePool,
    category: &str,
    uuid: &str,
) -> Result<Option<Value>, ApiError> {
    let sql = format!("SELECT data FROM {category} WHERE uuid = ?");
    let data: Option<String> = sqlx::query_scalar(&sql)
        .bind(uuid)
        .fetch_optional(db)
        .await?;
    data.map(|d| serde_json::from_str(&d).map_err(|e| ApiError::Internal(e.into())))
        .transpose()
}

fn json_str(value: &Value, key: &str) -> Result<String, ApiError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("assembled record missing {key}")))
}

fn record_error(e: RecordError) -> ApiError {
    match e {
        RecordError::Invalid(m) => ApiError::BadRequest(m),
        RecordError::Db(err) => ApiError::Internal(err),
    }
}
