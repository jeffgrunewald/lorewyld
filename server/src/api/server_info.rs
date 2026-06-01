use axum::{Json, extract::State};
use chrono::{DateTime, NaiveDate, Utc};
use lorewyld_types::{
    api_v1::{GameServerSummary, ServerInfo},
    content_module::ContentModule,
};
use uuid::Uuid;

use crate::api::{ApiState, error::ApiError};

pub async fn server_info(
    State(state): State<ApiState>,
) -> Result<Json<ServerInfo>, ApiError> {
    let (id, name, version): (String, String, String) =
        sqlx::query_as("SELECT id, name, version FROM game_server LIMIT 1")
            .fetch_one(&state.db)
            .await?;

    let server = GameServerSummary {
        uuid: Uuid::parse_str(&id).map_err(|e| ApiError::Internal(e.into()))?,
        name,
        version,
    };

    let modules = list_active_modules(&state).await?;
    Ok(Json(ServerInfo { server, modules }))
}

#[derive(sqlx::FromRow)]
struct ContentModuleRow {
    uuid: String,
    name: String,
    slug: String,
    license: String,
    license_url: Option<String>,
    schema_version: i64,
    release_date: Option<NaiveDate>,
    authors: String,
    publisher: Option<String>,
    description: Option<String>,
    website_url: Option<String>,
    is_active: i64,
    ordering: i64,
    version_string: String,
    previous_version_uuid: Option<String>,
    published_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ContentModuleRow {
    fn into_dto(self) -> Result<ContentModule, ApiError> {
        Ok(ContentModule {
            uuid: Uuid::parse_str(&self.uuid).map_err(|e| ApiError::Internal(e.into()))?,
            name: self.name,
            slug: self.slug,
            license: self.license,
            license_url: self.license_url,
            schema_version: self.schema_version as u32,
            release_date: self.release_date,
            authors: serde_json::from_str(&self.authors)
                .map_err(|e| ApiError::Internal(e.into()))?,
            publisher: self.publisher,
            description: self.description,
            website_url: self.website_url,
            is_active: self.is_active != 0,
            ordering: self.ordering as i32,
            version_string: self.version_string,
            previous_version_uuid: self
                .previous_version_uuid
                .as_deref()
                .map(|s| Uuid::parse_str(s).map_err(|e| ApiError::Internal(e.into())))
                .transpose()?,
            published_at: self.published_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

async fn list_active_modules(state: &ApiState) -> Result<Vec<ContentModule>, ApiError> {
    let rows: Vec<ContentModuleRow> = sqlx::query_as(
        "SELECT uuid, name, slug, license, license_url, schema_version,
                release_date, authors, publisher, description, website_url,
                is_active, ordering, version_string, previous_version_uuid,
                published_at, created_at, updated_at
           FROM content_module
          WHERE is_active = 1
          ORDER BY ordering, name",
    )
    .fetch_all(&state.db)
    .await?;

    rows.into_iter().map(ContentModuleRow::into_dto).collect()
}
