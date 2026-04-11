use gloo_net::http::Request;
use rcr_core::models::job::CreateJob;
use rcr_core::models::run::RunSummary;
use rcr_core::models::{Job, Run};
use serde::de::DeserializeOwned;

/// Fetch all jobs.
pub async fn fetch_jobs() -> Result<Vec<Job>, String> {
    api_get("/jobs").await
}

/// Create a new job.
pub async fn create_job(job: &CreateJob) -> Result<Job, String> {
    api_post("/jobs", job).await
}

/// Delete a job by ID.
pub async fn delete_job(id: &str) -> Result<(), String> {
    let resp = Request::delete(&format!("/api/jobs/{}", id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        let msg = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v["error"].as_str().map(String::from))
            .unwrap_or(text);
        return Err(msg);
    }

    Ok(())
}

/// Trigger a job by ID.
pub async fn trigger_job(id: &str) -> Result<(), String> {
    let resp = Request::post(&format!("/api/jobs/{}/trigger", id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        let msg = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v["error"].as_str().map(String::from))
            .unwrap_or(text);
        return Err(msg);
    }

    Ok(())
}

/// Fetch recent runs (with optional limit).
pub async fn fetch_runs(limit: i64) -> Result<Vec<RunSummary>, String> {
    api_get(&format!("/runs?limit={}", limit)).await
}

/// Fetch a single run by ID.
pub async fn fetch_run(id: &str) -> Result<Run, String> {
    api_get(&format!("/runs/{}", id)).await
}

// ─── Internal helpers ─────────────────────────────────────────────

async fn api_get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let resp = Request::get(&format!("/api{}", path))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
        let msg = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v["error"].as_str().map(String::from))
            .unwrap_or(text);
        return Err(msg);
    }

    let text = resp.text().await.map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&text).map_err(|e| format!("JSON error: {}", e))
}

async fn api_post<T: serde::Serialize, R: DeserializeOwned>(path: &str, body: &T) -> Result<R, String> {
    let body_str = serde_json::to_string(body).map_err(|e| format!("Serialize error: {}", e))?;

    let resp = Request::post(&format!("/api{}", path))
        .header("Content-Type", "application/json")
        .body(&body_str)
        .map_err(|e| format!("Request error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
        let msg = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v["error"].as_str().map(String::from))
            .unwrap_or(text);
        return Err(msg);
    }

    let text = resp.text().await.map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&text).map_err(|e| format!("JSON error: {}", e))
}