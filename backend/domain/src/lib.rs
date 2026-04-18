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

// ── Job Reducers ───────────────────────────────────────────────────────

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
        id: id.clone(),
        name: name.clone(),
        command,
        schedule,
        enabled,
        created_at: now,
        updated_at: now,
    };
    ctx.db.jobs().insert(job);
    log::info!("Job inserted: {name}");
}

#[reducer]
pub fn update_job(
    ctx: &ReducerContext,
    id: String,
    name: Option<String>,
    command: Option<String>,
    schedule: Option<Option<String>>,
    enabled: Option<bool>,
) {
    let now = ctx.timestamp;
    let Some(mut job) = ctx.db.jobs().id().find(&id) else {
        log::error!("Job not found: {id}");
        return;
    };
    if let Some(v) = name { job.name = v; }
    if let Some(v) = command { job.command = v; }
    // schedule is Option<Option<String>>: None = no change, Some(None) = clear schedule, Some(Some(v)) = set schedule
    if let Some(v) = schedule { job.schedule = v; }
    if let Some(v) = enabled { job.enabled = v; }
    job.updated_at = now;
    ctx.db.jobs().id().update(job);
    log::info!("Job updated: {id}");
}

#[reducer]
pub fn delete_job(
    ctx: &ReducerContext,
    id: String,
) {
    let _ = ctx.db.jobs().id().delete(&id);
    // Also delete all runs for this job
    let runs_to_delete: Vec<String> = ctx.db.runs().iter()
        .filter(|r| r.job_id == id)
        .map(|r| r.id.clone())
        .collect();
    for run_id in runs_to_delete {
        let _ = ctx.db.runs().id().delete(&run_id);
    }
    log::info!("Job deleted: {id}");
}

// ── Run Reducers ───────────────────────────────────────────────────────

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
        id: id.clone(),
        job_id,
        terminal_output,
        status,
        started_at,
        finished_at,
        created_at: now,
        updated_at: now,
    };
    ctx.db.runs().insert(run);
    log::info!("Run inserted: {id}");
}

#[reducer]
pub fn update_run(
    ctx: &ReducerContext,
    id: String,
    terminal_output: Option<Option<String>>,
    status: Option<RunStatus>,
    finished_at: Option<Option<Timestamp>>,
) {
    let now = ctx.timestamp;
    let Some(mut run) = ctx.db.runs().id().find(&id) else {
        log::error!("Run not found: {id}");
        return;
    };
    if let Some(v) = terminal_output { run.terminal_output = v; }
    if let Some(v) = status { run.status = v; }
    if let Some(v) = finished_at { run.finished_at = v; }
    run.updated_at = now;
    ctx.db.runs().id().update(run);
    log::info!("Run updated: {id}");
}

#[reducer]
pub fn delete_run(
    ctx: &ReducerContext,
    id: String,
) {
    let _ = ctx.db.runs().id().delete(&id);
    log::info!("Run deleted: {id}");
}

#[reducer(init)]
pub fn init(_ctx: &ReducerContext) {
    log::info!("Run Crab Run domain module initialized");
}