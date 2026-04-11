use axum::extract::{Path, State};
use axum::Json;
use axum::body::Bytes;
use axum::http::HeaderMap;
use rcr_core::models::webhook::{CreateWebhookSubscription, WebhookSubscription};
use rcr_core::models::trigger::Trigger;
use hmac::Mac;
use tracing::info;

use crate::error::ApiError;
use crate::state::AppState;

type HmacSha256 = hmac::Hmac<sha2::Sha256>;

pub async fn list_webhooks(
    State(state): State<AppState>,
) -> Result<Json<Vec<WebhookSubscription>>, ApiError> {
    let webhooks = state.db.list_webhooks().await?;
    Ok(Json(webhooks))
}

pub async fn create_webhook(
    State(state): State<AppState>,
    Json(create): Json<CreateWebhookSubscription>,
) -> Result<Json<WebhookSubscription>, ApiError> {
    state.db.get_job(&create.job_id).await?;
    let webhook = state.db.create_webhook(create).await?;
    Ok(Json(webhook))
}

pub async fn delete_webhook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.db.delete_webhook(&id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn receive_webhook(
    State(state): State<AppState>,
    Path(name): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, ApiError> {
    let webhook = state.db.get_webhook_by_name(&name).await?;
    let job = state.db.get_job(&webhook.job_id).await?;

    // Verify HMAC-SHA256 signature if X-Hub-Signature-256 is present (GitHub-style)
    if let Some(sig_header) = headers.get("X-Hub-Signature-256") {
        let sig = sig_header.to_str().map_err(|_| rcr_core::Error::WebhookSecretMismatch)?;
        if !verify_hmac(&body, &webhook.secret, sig) {
            return Err(ApiError(rcr_core::Error::WebhookSecretMismatch));
        }
    } else if let Some(provided_secret) = headers.get("X-Webhook-Secret") {
        if provided_secret.to_str().unwrap_or("") != webhook.secret {
            return Err(ApiError(rcr_core::Error::WebhookSecretMismatch));
        }
    }

    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();

    let mut env_vars = serde_json::json!({});
    if let Some(mapping) = webhook.arg_mapping.as_object() {
        for (env_var, json_path) in mapping {
            if let Some(value) = resolve_json_path(&payload, json_path.as_str().unwrap_or("")) {
                env_vars[env_var] = value;
            }
        }
    }

    let merged_env = if let Some(existing) = job.env_vars.as_object() {
        let mut merged = existing.clone();
        if let Some(dynamic) = env_vars.as_object() {
            for (k, v) in dynamic {
                merged.insert(k.clone(), v.clone());
            }
        }
        serde_json::Value::Object(merged)
    } else {
        env_vars
    };

    info!(webhook = %name, job_id = %job.id, "Triggering job from webhook");

    state
        .executor
        .trigger_job(&job, Trigger::Webhook { name: name.clone() }, Some(merged_env))
        .await?;

    Ok(Json(serde_json::json!({ "triggered": true, "job_id": job.id })))
}

fn verify_hmac(body: &[u8], secret: &str, signature: &str) -> bool {
    let sig = signature.strip_prefix("sha256=").unwrap_or(signature);
    let Ok(expected) = hex::decode(sig) else { return false };

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);

    mac.verify_slice(&expected).is_ok()
}

fn resolve_json_path<'a>(value: &'a serde_json::Value, path: &str) -> Option<serde_json::Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;
    for part in parts {
        if part.is_empty() { continue; }
        current = current.get(part)?;
    }
    Some(current.clone())
}