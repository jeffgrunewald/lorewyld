use axum::{Json, extract::State};
use lorewyld_types::{
    api_v1::{GameServerSummary, ServerInfo},
    content_module::ContentModule,
};
use uuid::Uuid;

use crate::api::{
    ApiState,
    error::ApiError,
    rows::{ContentModuleRow, MODULE_SELECT_ACTIVE},
};

pub async fn server_info(State(state): State<ApiState>) -> Result<Json<ServerInfo>, ApiError> {
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

async fn list_active_modules(state: &ApiState) -> Result<Vec<ContentModule>, ApiError> {
    let rows: Vec<ContentModuleRow> = sqlx::query_as(MODULE_SELECT_ACTIVE)
        .fetch_all(&state.db)
        .await?;
    rows.into_iter().map(ContentModuleRow::into_dto).collect()
}
