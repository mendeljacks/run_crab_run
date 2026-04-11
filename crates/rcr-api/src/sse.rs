use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use rcr_core::models::run::RunsFilter;
use std::convert::Infallible;
use tokio::time::Duration;
use crate::state::AppState;

/// SSE endpoint that polls for run changes every 3 seconds.
pub async fn run_events(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let db = state.db.clone();

    let stream = async_stream::stream! {
        loop {
            tokio::time::sleep(Duration::from_secs(3)).await;

            match db.list_runs(RunsFilter {
                job_id: None,
                status: None,
                limit: Some(50),
                offset: None,
            }).await {
                Ok(runs) => {
                    let data = serde_json::to_string(&runs).unwrap_or_else(|_| "[]".to_string());
                    yield Ok(Event::default().event("runs").data(data));
                }
                Err(_) => {
                    // Skip on error
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    )
}