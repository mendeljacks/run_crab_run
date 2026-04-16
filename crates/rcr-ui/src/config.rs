/// Returns the configured API base URL for the frontend.
///
/// In JAMstack mode, the frontend is served from a different origin than
/// the API, so it needs to know the full API URL. This is configured at
/// deploy time via the `window.__API_BASE_URL__` global in index.html.
///
/// For local development with Trunk's proxy, this defaults to an empty
/// string, which makes all API calls relative (e.g. `/api/jobs`) — the
/// Trunk dev server proxies those to the backend.
///
/// For production, set `window.__API_BASE_URL__` to the full API origin,
/// e.g. `"https://api.example.com"` — calls become
/// `"https://api.example.com/api/jobs"`.
pub fn api_base_url() -> String {
    web_sys::window()
        .and_then(|w| {
            js_sys::Reflect::get(
                &w.into(),
                &wasm_bindgen::JsValue::from_str("__API_BASE_URL__")
            ).ok()
        })
        .and_then(|v: wasm_bindgen::JsValue| v.as_string())
        .unwrap_or_default()
}

/// Build a full URL for an SSE endpoint like `/api/events/runs`.
///
/// Handles the same base URL logic as `api.rs::api_url`, but for
/// EventSource URLs which need a single string rather than a Request builder.
pub fn sse_url(path: &str) -> String {
    let base = api_base_url();
    if base.is_empty() {
        format!("/api{}", path)
    } else {
        format!("{}/api{}", base, path)
    }
}