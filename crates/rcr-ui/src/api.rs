use gloo_net::http::Request;
use rcr_core::models::job::{CreateJob, UpdateJob};
use rcr_core::models::job::Job;
use rcr_core::models::run::{RunsFilter, RunsResponse, Run};
use serde::de::DeserializeOwned;

/// Fetch all jobs.
pub async fn fetch_jobs() -> Result<Vec<Job>, String> {
    api_get("/jobs").await
}

/// Fetch a single job by ID.
pub async fn fetch_job(id: &str) -> Result<Job, String> {
    api_get(&format!("/jobs/{}", id)).await
}

/// Create a new job.
pub async fn create_job(job: &CreateJob) -> Result<Job, String> {
    api_post("/jobs", job).await
}

/// Update an existing job.
pub async fn update_job(id: &str, job: &UpdateJob) -> Result<Job, String> {
    api_patch(&format!("/jobs/{}", id), job).await
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

/// Fetch runs with filter parameters.
pub async fn fetch_runs(filter: &RunsFilter) -> Result<RunsResponse, String> {
    let mut params = Vec::new();
    if let Some(ref job_id) = filter.job_id {
        params.push(format!("job_id={}", url_encode(job_id)));
    }
    if let Some(ref status) = filter.status {
        params.push(format!("status={}", status));
    }
    if let Some(ref search) = filter.search {
        params.push(format!("search={}", url_encode(search)));
    }
    if let Some(ref sort_by) = filter.sort_by {
        params.push(format!("sort_by={}", sort_by));
    }
    if let Some(ref sort_order) = filter.sort_order {
        params.push(format!("sort_order={}", sort_order));
    }
    if let Some(limit) = filter.limit {
        params.push(format!("limit={}", limit));
    }
    if let Some(offset) = filter.offset {
        params.push(format!("offset={}", offset));
    }
    let qs = if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) };
    api_get(&format!("/runs{}", qs)).await
}

/// Fetch a single run by ID.
pub async fn fetch_run(id: &str) -> Result<Run, String> {
    api_get(&format!("/runs/{}", id)).await
}

fn url_encode(s: &str) -> String {
    s.replace('%', "%25")
        .replace(' ', "%20")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace('/', "%2F")
        .replace('+', "%2B")
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

async fn api_patch<T: serde::Serialize, R: DeserializeOwned>(path: &str, body: &T) -> Result<R, String> {
    let body_str = serde_json::to_string(body).map_err(|e| format!("Serialize error: {}", e))?;

    let resp = Request::patch(&format!("/api{}", path))
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