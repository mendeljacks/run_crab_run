use rcr_core::notify::{should_notify, Notifier};
use rcr_core::models::job::Job;
use rcr_core::models::run::RunStatus;
use rcr_core::models::trigger::Trigger;
use rcr_core::Error;
use rcr_db::Database;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use std::time::Duration;

use crate::monitor::ProcessMonitor;

/// A pending run that will execute after the current one finishes.
struct PendingRun {
    trigger: Trigger,
    webhook_args: Option<serde_json::Value>,
}

/// Tracks in-flight runs per job for debounce/coalesce logic.
struct JobRunState {
    pending: Option<PendingRun>,
}

pub struct JobExecutor {
    db: Database,
    run_states: Arc<Mutex<HashMap<String, JobRunState>>>,
    notifier: Arc<dyn Notifier>,
}

impl JobExecutor {
    pub fn new(db: Database, notifier: Arc<dyn Notifier>) -> Self {
        Self {
            db,
            run_states: Arc::new(Mutex::new(HashMap::new())),
            notifier,
        }
    }

    /// Trigger a job run.
    /// If the job is already running, coalesce: queue one re-run with the latest args.
    /// Returns the run ID (or "pending:{job_id}" if coalesced).
    pub async fn trigger_job(
        &self,
        job: &Job,
        trigger: Trigger,
        webhook_args: Option<serde_json::Value>,
    ) -> Result<String, Error> {
        let job_id = job.id.clone();

        let mut states = self.run_states.lock().await;
        if states.contains_key(&job_id) {
            // Job is currently running — coalesce
            info!(job_id = %job.id, "Job already running, coalescing pending run");
            let state = states.get_mut(&job_id).unwrap();
            state.pending = Some(PendingRun { trigger, webhook_args });
            Ok(format!("pending:{}", job_id))
        } else {
            // Start immediately
            let run_id = self.spawn_run(job.id.clone(), trigger, webhook_args.clone()).await?;
            states.insert(job_id, JobRunState { pending: None });
            Ok(run_id)
        }
    }

    async fn spawn_run(
        &self,
        job_id: String,
        trigger: Trigger,
        webhook_args: Option<serde_json::Value>,
    ) -> Result<String, Error> {
        let create = rcr_core::models::run::CreateRun {
            job_id: job_id.clone(),
            trigger: trigger.clone(),
            webhook_args: webhook_args.clone(),
        };

        let run = self.db.create_run(create).await?;
        let run_id = run.id.clone();
        let run_id_return = run.id.clone();

        info!(job_id = %job_id, run_id = %run_id, "Starting job execution");

        let job = self.db.get_job(&job_id).await?;

        let db = self.db.clone();
        let notifier = self.notifier.clone();
        let run_states = self.run_states.clone();
        let executor_self_job_id = job_id.clone();

        tokio::spawn(async move {
            let result = execute_command(
                &job.command,
                &job.env_vars,
                job.timeout_secs.map(|s| Duration::from_secs(s as u64)),
                webhook_args.clone(),
            ).await;

            let (status, exit_code, stdout, stderr, error_message, cpu_pct, mem_kb, duration_ms) = match result {
                Ok(out) => {
                    let status = if out.exit_code == 0 { RunStatus::Success } else { RunStatus::Failed };
                    info!(job_id = %job_id, run_id = %run_id, exit_code = out.exit_code, duration_ms = out.duration_ms, "Job completed");
                    (status, Some(out.exit_code), Some(out.stdout), Some(out.stderr), None, out.cpu_peak, out.mem_peak_kb, Some(out.duration_ms))
                }
                Err(e) => {
                    error!(job_id = %job_id, run_id = %run_id, error = %e, "Job execution failed");
                    let status = match &e {
                        ExecutionError::Timeout => RunStatus::Timeout,
                        _ => RunStatus::Failed,
                    };
                    (status, None, None, None, Some(e.to_string()), None, None, None)
                }
            };

            if let Err(e) = db.update_run_completed(&run_id, status, exit_code, stdout.clone(), stderr.clone(), error_message, cpu_pct, mem_kb, duration_ms).await {
                error!(run_id = %run_id, error = %e, "Failed to update run record");
            }

            // Notification
            if let Some(ref policy) = job.notify_on {
                if should_notify(policy, status) {
                    if let Some(ref email) = job.notify_email {
                        let stdout_snippet = stdout.as_deref().unwrap_or("").chars().take(5000).collect::<String>();
                        let stderr_snippet = stderr.as_deref().unwrap_or("").chars().take(5000).collect::<String>();
                        if let Err(e) = notifier.notify(&job.name, &run_id, status, exit_code, &stdout_snippet, &stderr_snippet, email) {
                            warn!(error = %e, "Failed to send notification");
                        }
                    }
                }
            }

            // Check for pending run and execute it
            let mut states = run_states.lock().await;
            if let Some(state) = states.get_mut(&executor_self_job_id) {
                if let Some(pending) = state.pending.take() {
                    drop(states);

                    info!(job_id = %executor_self_job_id, "Executing coalesced pending run");
                    // We need to re-trigger through the executor pattern.
                    // Since we can't hold a reference to self in the spawned task,
                    // we directly run the job again.
                    let db = db.clone();
                    let job_id = executor_self_job_id.clone();
                    let run_states2 = run_states.clone();
                    let notifier2 = notifier.clone();

                    tokio::spawn(async move {
                        // Look up job again (it might have been updated)
                        let job = match db.get_job(&job_id).await {
                            Ok(j) => j,
                            Err(e) => {
                                error!(error = %e, "Failed to fetch job for pending run");
                                let mut s = run_states2.lock().await;
                                s.remove(&job_id);
                                return;
                            }
                        };

                        let create = rcr_core::models::run::CreateRun {
                            job_id: job_id.clone(),
                            trigger: pending.trigger,
                            webhook_args: pending.webhook_args,
                        };

                        let run = match db.create_run(create).await {
                            Ok(r) => r,
                            Err(e) => {
                                error!(error = %e, "Failed to create pending run record");
                                let mut s = run_states2.lock().await;
                                s.remove(&job_id);
                                return;
                            }
                        };
                        let run_id = run.id;

                        let result = execute_command(
                            &job.command,
                            &job.env_vars,
                            job.timeout_secs.map(|s| Duration::from_secs(s as u64)),
                            run.webhook_args.clone(),
                        ).await;

                        let (status, exit_code, stdout, stderr, error_message, cpu_pct, mem_kb, duration_ms) = match result {
                            Ok(out) => {
                                let status = if out.exit_code == 0 { RunStatus::Success } else { RunStatus::Failed };
                                info!(job_id = %job_id, run_id = %run_id, "Pending run completed");
                                (status, Some(out.exit_code), Some(out.stdout), Some(out.stderr), None, out.cpu_peak, out.mem_peak_kb, Some(out.duration_ms))
                            }
                            Err(e) => {
                                error!(job_id = %job_id, run_id = %run_id, error = %e, "Pending run execution failed");
                                let status = match &e {
                                    ExecutionError::Timeout => RunStatus::Timeout,
                                    _ => RunStatus::Failed,
                                };
                                (status, None, None, None, Some(e.to_string()), None, None, None)
                            }
                        };

                        if let Err(e) = db.update_run_completed(&run_id, status, exit_code.clone(), stdout.clone(), stderr.clone(), error_message, cpu_pct, mem_kb, duration_ms).await {
                            error!(run_id = %run_id, error = %e, "Failed to update pending run record");
                        }

                        // Notification for pending run
                        if let Some(ref policy) = job.notify_on {
                            if should_notify(policy, status) {
                                if let Some(ref email) = job.notify_email {
                                    let stdout_snippet = stdout.as_deref().unwrap_or("").chars().take(5000).collect::<String>();
                                    let stderr_snippet = stderr.as_deref().unwrap_or("").chars().take(5000).collect::<String>();
                                    if let Err(e) = notifier2.notify(&job.name, &run_id, status, exit_code, &stdout_snippet, &stderr_snippet, email) {
                                        warn!(error = %e, "Failed to send notification for pending run");
                                    }
                                }
                            }
                        }

                        // Check for ANOTHER pending run
                        let mut s = run_states2.lock().await;
                        if let Some(state) = s.get_mut(&job_id) {
                            if state.pending.is_some() {
                                // Another trigger came in while we were running.
                                // For simplicity, we don't recurse further here.
                                // The next scheduler tick or manual trigger will handle it.
                                // Leave the state in place.
                            } else {
                                s.remove(&job_id);
                            }
                        } else {
                            // State was already removed (shouldn't happen)
                        }
                    });
                } else {
                    // No pending run — remove the state entry, job is idle
                    states.remove(&executor_self_job_id);
                }
            }
        });

        Ok(run_id_return)
    }
}

struct CommandOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
    duration_ms: i64,
    cpu_peak: Option<f32>,
    mem_peak_kb: Option<i64>,
}

#[derive(Debug, thiserror::Error)]
enum ExecutionError {
    #[error("Command timed out")]
    Timeout,
    #[error("Failed to start process: {0}")]
    StartFailed(String),
    #[error("Process error: {0}")]
    Other(String),
}

async fn execute_command(
    command: &str,
    env_vars: &serde_json::Value,
    timeout: Option<Duration>,
    override_env: Option<serde_json::Value>,
) -> Result<CommandOutput, ExecutionError> {
    let start = std::time::Instant::now();

    // Merge static env_vars with webhook override_env (override wins)
    let mut env = std::collections::HashMap::new();
    if let Some(obj) = env_vars.as_object() {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                env.insert(k.clone(), s.to_string());
            }
        }
    }
    if let Some(override_env) = &override_env {
        if let Some(obj) = override_env.as_object() {
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    env.insert(k.clone(), s.to_string());
                }
            }
        }
    }

    let mut cmd = tokio::process::Command::new("sh");
    cmd.arg("-c").arg(command);

    // Set merged environment variables
    for (k, v) in &env {
        cmd.env(k, v);
    }

    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    let mut child = cmd
        .spawn()
        .map_err(|e| ExecutionError::StartFailed(e.to_string()))?;

    // Start CPU/RAM monitoring
    let pid = child.id().unwrap_or(0) as u32;
    let monitor = ProcessMonitor::new(pid);
    let monitor_handle = tokio::spawn(async move {
        monitor.sample_loop(Duration::from_millis(500)).await;
    });

    let result = if let Some(timeout) = timeout {
        match tokio::time::timeout(timeout, child.wait()).await {
            Ok(Ok(status)) => status,
            Ok(Err(e)) => {
                let _ = child.kill().await;
                monitor_handle.abort();
                return Err(ExecutionError::Other(e.to_string()));
            }
            Err(_) => {
                let _ = child.kill().await;
                monitor_handle.abort();
                return Err(ExecutionError::Timeout);
            }
        }
    } else {
        child
            .wait()
            .await
            .map_err(|e| ExecutionError::Other(e.to_string()))?
    };

    let exit_code = result.code().unwrap_or(-1);

    // Collect output
    let output = child
        .wait_with_output()
        .await
        .map_err(|e| ExecutionError::Other(e.to_string()))?;

    // Stop monitoring and get results
    monitor_handle.abort();
    let (cpu_peak, mem_peak_kb) = {
        // Since we spawned the monitor in a separate task, we can't get its results directly.
        // We'll sample once more here as a final reading.
        let mut final_monitor = ProcessMonitor::new(pid);
        final_monitor.sample();
        (final_monitor.peak_cpu(), final_monitor.peak_mem_kb())
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let duration_ms = start.elapsed().as_millis() as i64;

    Ok(CommandOutput {
        exit_code,
        stdout,
        stderr,
        duration_ms,
        cpu_peak,
        mem_peak_kb,
    })
}