mod jobs;
mod runs;
mod trigger;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/jobs", get(jobs::list_jobs).post(jobs::create_job))
        .route("/jobs/{id}", get(jobs::get_job).patch(jobs::update_job).delete(jobs::delete_job))
        .route("/runs", get(runs::list_runs))
        .route("/runs/{id}", get(runs::get_run).delete(runs::cancel_run))
        .route("/jobs/{id}/trigger", post(trigger::trigger_job))
        .with_state(state.clone());

    Router::new()
        .nest("/api", api)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}