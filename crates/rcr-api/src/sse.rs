use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use rcr_core::models::run::{RunStatus, RunsFilter};
use std::convert::Infallible;
use tokio::time::Duration;
use crate::state::AppState;

/// SSE endpoint that pushes live run updates every 3 seconds.
/// Sends two event types:
///   - "running" — the list of currently-running runs (for live status indicators)
///   - "recent" — recently-completed runs (last 50, for immediate UI refresh after completion)
pub async fn run_events(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let db = state.db.clone();

    let stream = async_stream::stream! {
        loop {
            tokio::time::sleep(Duration::from_secs(3)).await;

            // Currently running runs
            match db.list_runs(RunsFilter {
                job_id: None,
                status: Some(RunStatus::Running),
                search: None,
                sort_by: Some("started_at".to_string()),
                sort_order: Some("desc".to_string()),
                limit: Some(50),
                offset: None,
            }).await {
                Ok(response) => {
                    let data = serde_json::to_string(&response.runs).unwrap_or_else(|_| "[]".to_string());
                    yield Ok(Event::default().event("running").data(data));
                }
                Err(_) => {}
            }

            // Recent runs (last 50) so UI can spot completions
            match db.list_runs(RunsFilter {
                job_id: None,
                status: None,
                search: None,
                sort_by: Some("started_at".to_string()),
                sort_order: Some("desc".to_string()),
                limit: Some(50),
                offset: Some(0),
            }).await {
                Ok(response) => {
                    let data = serde_json::to_string(&response.runs).unwrap_or_else(|_| "[]".to_string());
                    yield Ok(Event::default().event("recent").data(data));
                }
                Err(_) => {}
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    )
}
