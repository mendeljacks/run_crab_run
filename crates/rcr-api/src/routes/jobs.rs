use axum::extract::{Path, State};
use axum::Json;
use rcr_core::models::job::{CreateJob, Job, UpdateJob};

use crate::error::ApiError;
use crate::state::AppState;

pub async fn list_jobs(
    State(state): State<AppState>,
) -> Result<Json<Vec<Job>>, ApiError> {
    let jobs = state.db.list_jobs().await?;
    Ok(Json(jobs))
}

pub async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Job>, ApiError> {
    let job = state.db.get_job(&id).await?;
    Ok(Json(job))
}

pub async fn create_job(
    State(state): State<AppState>,
    Json(create): Json<CreateJob>,
) -> Result<Json<Job>, ApiError> {
    let job = state.db.create_job(create).await?;
    Ok(Json(job))
}

pub async fn update_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(update): Json<UpdateJob>,
) -> Result<Json<Job>, ApiError> {
    let job = state.db.update_job(&id, update).await?;
    Ok(Json(job))
}

pub async fn delete_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.db.delete_job(&id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}