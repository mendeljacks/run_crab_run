use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSubscription {
    pub id: String,
    pub job_id: String,
    pub name: String, // URL slug, e.g. "github-push"
    pub secret: String,
    pub arg_mapping: serde_json::Value, // JSON: webhook payload → env vars
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookSubscription {
    pub job_id: String,
    pub name: String,
    pub secret: String,
    pub arg_mapping: serde_json::Value,
}