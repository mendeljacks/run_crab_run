use axum::extract::{Path, State};
use axum::Json;
use rcr_core::models::trigger::Trigger;

use crate::error::ApiError;
use crate::state::AppState;

pub async fn trigger_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let job = state.db.get_job(&id).await?;
    state
        .executor
        .trigger_job(&job, Trigger::Manual, None)
        .await?;
    Ok(Json(serde_json::json!({ "triggered": true, "job_id": id })))
}