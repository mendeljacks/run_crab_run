use axum::extract::{Path, Query, State};
use axum::Json;
use rcr_core::models::run::{Run, RunsFilter, RunsResponse};

use crate::error::ApiError;
use crate::state::AppState;

pub async fn list_runs(
    State(state): State<AppState>,
    Query(filter): Query<RunsFilter>,
) -> Result<Json<RunsResponse>, ApiError> {
    let response = state.db.list_runs(filter).await?;
    Ok(Json(response))
}

pub async fn get_run(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Run>, ApiError> {
    let run = state.db.get_run(&id).await?;
    Ok(Json(run))
}

pub async fn cancel_run(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.db.cancel_run(&id).await?;
    Ok(Json(serde_json::json!({ "cancelled": true })))
}