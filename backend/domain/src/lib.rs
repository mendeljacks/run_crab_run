use spacetimedb::{log, reducer, ReducerContext, SpacetimeType, Table, Timestamp};

// ── Enums ──────────────────────────────────────────────────────────────

#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Running,
    Succeeded,
    Failed,
}

// ── Tables ─────────────────────────────────────────────────────────────

#[spacetimedb::table(accessor = jobs, name = "jobs", public)]
pub struct Job {
    #[primary_key]
    pub id: String,
    pub name: String,
    pub command: String,
    pub schedule: Option<String>,
    pub enabled: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[spacetimedb::table(accessor = runs, name = "runs", public)]
pub struct Run {
    #[primary_key]
    pub id: String,
    pub job_id: String,
    pub terminal_output: Option<String>,
    pub status: RunStatus,
    pub started_at: Timestamp,
    pub finished_at: Option<Timestamp>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

// ── Reducers ───────────────────────────────────────────────────────────

#[reducer]
pub fn insert_job(
    ctx: &ReducerContext,
    id: String,
    name: String,
    command: String,
    schedule: Option<String>,
    enabled: bool,
) {
    let now = ctx.timestamp;
    let job = Job {
        id,
        name,
        command,
        schedule,
        enabled,
        created_at: now,
        updated_at: now,
    };
    ctx.db.jobs().insert(job);
    log::info!("Job inserted");
}

#[reducer]
pub fn insert_run(
    ctx: &ReducerContext,
    id: String,
    job_id: String,
    terminal_output: Option<String>,
    status: RunStatus,
    started_at: Timestamp,
    finished_at: Option<Timestamp>,
) {
    let now = ctx.timestamp;
    let run = Run {
        id,
        job_id,
        terminal_output,
        status,
        started_at,
        finished_at,
        created_at: now,
        updated_at: now,
    };
    ctx.db.runs().insert(run);
    log::info!("Run inserted");
}

#[reducer(init)]
pub fn init(_ctx: &ReducerContext) {
    log::info!("Run Crab Run domain module initialized");
}