mod jobs;
mod runs;
mod trigger;
mod webhooks;

use axum::{
    routing::{delete, get, post},
    Router,
};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use http::Method;
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;
use rcr_core::models::config::ServerConfig;

use crate::sse;
use crate::state::AppState;

pub fn router(state: AppState, server_config: &ServerConfig) -> Router {
    let api = Router::new()
        .route("/jobs", get(jobs::list_jobs).post(jobs::create_job))
        .route("/jobs/{id}", get(jobs::get_job).patch(jobs::update_job).delete(jobs::delete_job))
        .route("/runs", get(runs::list_runs))
        .route("/runs/{id}", get(runs::get_run).delete(runs::cancel_run))
        .route("/jobs/{id}/trigger", post(trigger::trigger_job))
        .route("/webhooks", get(webhooks::list_webhooks).post(webhooks::create_webhook))
        .route("/webhooks/{id}", delete(webhooks::delete_webhook))
        .route("/hook/{name}", post(webhooks::receive_webhook))
        .route("/events/runs", get(sse::run_events))
        .with_state(state.clone());

    let cors_layer = build_cors_layer(&server_config.cors_origins);

    Router::new()
        .nest("/api", api)
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer)
}

fn build_cors_layer(origins: &[String]) -> CorsLayer {
    if origins.is_empty() {
        CorsLayer::permissive()
    } else {
        let allowed_origins: Vec<_> = origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::DELETE,
            ])
            .allow_headers([CONTENT_TYPE, AUTHORIZATION])
    }
}