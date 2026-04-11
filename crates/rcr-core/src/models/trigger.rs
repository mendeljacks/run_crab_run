use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trigger {
    Schedule,
    Manual,
    Webhook { name: String },
}

impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Trigger::Schedule => write!(f, "schedule"),
            Trigger::Manual => write!(f, "manual"),
            Trigger::Webhook { name } => write!(f, "webhook:{}", name),
        }
    }
}