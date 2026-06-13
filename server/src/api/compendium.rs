//! Read-only compendium endpoints over the seeded content tables.
//!
//! Every query joins `content_module` on `is_active = 1`, so disabling
//! a module hides its content here (and 404s entry reads) without
//! touching the rows themselves.
//!
//! List responses are slim projections — identity + the indexed filter
//! columns + a few `json_extract`ed fields ([`CategorySpec::summary_fields`])
//! — because full records would be megabytes for the big tables.
//! Clients fetch the verbatim record JSON per entry instead. Small
//! lookup tables return full records, since consumers want fields like
//! `rank` wholesale and the payloads are tiny.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use lorewyld_types::api_v1::{CategoryCount, ContentCountsResponse};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    api::{ApiState, auth::CurrentUser, error::ApiError},
    content::{CategorySpec, DISPLAY_CATEGORIES, category_spec},
};

const ACTIVE_JOIN: &str = "JOIN content_module m ON m.uuid = t.content_module_uuid \
                           AND m.is_active = 1";

/// `GET /api/content/counts` — entry counts per compendium category
/// across active modules, for the landing-grid tiles.
pub async fn content_counts(
    State(state): State<ApiState>,
    _user: CurrentUser,
) -> Result<Json<ContentCountsResponse>, ApiError> {
    let sql = DISPLAY_CATEGORIES
        .iter()
        .map(|table| {
            format!("SELECT '{table}' AS category, COUNT(*) AS n FROM {table} t {ACTIVE_JOIN}")
        })
        .collect::<Vec<_>>()
        .join(" UNION ALL ");
    let rows: Vec<(String, u32)> = sqlx::query_as(&sql).fetch_all(&state.db).await?;
    Ok(Json(ContentCountsResponse {
        counts: rows
            .into_iter()
            .map(|(category, count)| CategoryCount { category, count })
            .collect(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct ContentListQuery {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}

/// `GET /api/content/{category}?q&limit` — list a category's entries
/// from active modules, name-sorted. `category` must name a known
/// content table; it is resolved through the static spec map, never
/// interpolated from user input.
pub async fn list_category(
    State(state): State<ApiState>,
    _user: CurrentUser,
    Path(category): Path<String>,
    Query(query): Query<ContentListQuery>,
) -> Result<Json<Vec<Value>>, ApiError> {
    let spec = category_spec(&category).ok_or(ApiError::NotFound)?;

    let projection = if spec.include_data {
        "t.data".to_string()
    } else {
        summary_projection(spec)
    };
    let mut sql = format!(
        "SELECT {projection} FROM {t} t {ACTIVE_JOIN}",
        t = spec.table
    );
    if query.q.is_some() {
        sql.push_str(" WHERE t.name LIKE ? ESCAPE '\\'");
    }
    sql.push_str(" ORDER BY t.name");
    if query.limit.is_some() {
        sql.push_str(" LIMIT ?");
    }

    let mut db_query = sqlx::query_scalar::<_, String>(&sql);
    if let Some(q) = &query.q {
        let escaped = q
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        db_query = db_query.bind(format!("%{escaped}%"));
    }
    if let Some(limit) = query.limit {
        db_query = db_query.bind(limit.clamp(1, 10_000));
    }

    let rows = db_query.fetch_all(&state.db).await?;
    rows.into_iter()
        .map(|json| serde_json::from_str(&json).map_err(|e| ApiError::Internal(e.into())))
        .collect::<Result<Vec<Value>, _>>()
        .map(Json)
}

/// `GET /api/content/{category}/{uuid}` — one entry's stored record
/// JSON, verbatim. Disabled-module entries 404.
pub async fn get_entry(
    State(state): State<ApiState>,
    _user: CurrentUser,
    Path((category, uuid)): Path<(String, Uuid)>,
) -> Result<Response, ApiError> {
    let spec = category_spec(&category).ok_or(ApiError::NotFound)?;
    let sql = format!(
        "SELECT t.data FROM {t} t {ACTIVE_JOIN} WHERE t.uuid = ?",
        t = spec.table
    );
    let data: Option<String> = sqlx::query_scalar(&sql)
        .bind(uuid.to_string())
        .fetch_optional(&state.db)
        .await?;
    let data = data.ok_or(ApiError::NotFound)?;
    // The column already holds the serialized record — pass it through
    // without a decode/encode round trip.
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        data,
    )
        .into_response())
}

/// Builds a `json_object(...)` projection of identity columns, indexed
/// extras, and the spec's record-JSON summary fields, so SQLite
/// assembles each slim row directly.
fn summary_projection(spec: &CategorySpec) -> String {
    let mut pairs = vec![
        "'uuid', t.uuid".to_string(),
        "'key', t.key".to_string(),
        "'slug', t.slug".to_string(),
        "'name', t.name".to_string(),
        "'content_module_uuid', t.content_module_uuid".to_string(),
    ];
    // Indexed extras surface under their record-JSON field name (the
    // pointer tail), so list rows look exactly like the stored records
    // ('school', not 'school_uuid') and client code ports 1:1 from the
    // mobile app.
    pairs.extend(spec.extras.iter().map(|(col, ptr)| {
        let field = ptr.trim_start_matches('/');
        format!("'{field}', t.{col}")
    }));
    pairs.extend(
        spec.summary_fields
            .iter()
            .map(|field| format!("'{field}', json_extract(t.data, '$.{field}')")),
    );
    format!("json_object({})", pairs.join(", "))
}
