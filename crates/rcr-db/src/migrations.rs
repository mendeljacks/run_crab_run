pub const MIGRATION_001: &str = r#"
CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    command TEXT NOT NULL,
    schedule TEXT,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    timeout_secs INTEGER,
    max_concurrent INTEGER NOT NULL DEFAULT 1,
    env_vars TEXT NOT NULL DEFAULT '{}',
    webhook_secret TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    notify_on TEXT,
    notify_email TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
"#;

pub const MIGRATION_002: &str = r#"
CREATE TABLE IF NOT EXISTS runs (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    trigger TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'running',
    exit_code INTEGER,
    stdout TEXT,
    stderr TEXT,
    error_message TEXT,
    cpu_pct REAL,
    mem_kb INTEGER,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    duration_ms INTEGER,
    webhook_args TEXT,
    FOREIGN KEY (job_id) REFERENCES jobs(id)
);

CREATE INDEX IF NOT EXISTS idx_runs_job_id ON runs(job_id);
CREATE INDEX IF NOT EXISTS idx_runs_status ON runs(status);
CREATE INDEX IF NOT EXISTS idx_runs_started_at ON runs(started_at);
"#;

pub const MIGRATION_003: &str = r#"
CREATE TABLE IF NOT EXISTS webhook_subscriptions (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    name TEXT NOT NULL UNIQUE,
    secret TEXT NOT NULL,
    arg_mapping TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_webhooks_job_id ON webhook_subscriptions(job_id);
"#;