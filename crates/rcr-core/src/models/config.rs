/// Configuration types shared across crates.
///
/// Each binary reads its own config from environment variables
/// and constructs these structs directly — no TOML/file config.

/// Server configuration for the API crate.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

/// Email notification configuration.
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_address: String,
}