use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub command: String,
    pub schedule: Option<String>,
    pub enabled: bool,
    pub max_concurrent: i32,
    pub env_vars: serde_json::Value,
    pub containerized: bool,
    pub container_image: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJob {
    pub name: String,
    pub command: String,
    pub schedule: Option<String>,
    pub enabled: Option<bool>,
    pub max_concurrent: Option<i32>,
    pub env_vars: Option<serde_json::Value>,
    pub containerized: Option<bool>,
    pub container_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateJob {
    pub name: Option<String>,
    pub command: Option<String>,
    pub schedule: Option<String>,
    pub enabled: Option<bool>,
    pub max_concurrent: Option<i32>,
    pub env_vars: Option<serde_json::Value>,
    pub containerized: Option<bool>,
    pub container_image: Option<String>,
}

impl Job {
    pub fn new(create: CreateJob) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: create.name,
            command: create.command,
            schedule: create.schedule,
            enabled: create.enabled.unwrap_or(true),
            max_concurrent: create.max_concurrent.unwrap_or(1),
            env_vars: create.env_vars.unwrap_or(serde_json::json!({})),
            containerized: create.containerized.unwrap_or(false),
            container_image: create.container_image,
            created_at: now,
            updated_at: now,
        }
    }
}