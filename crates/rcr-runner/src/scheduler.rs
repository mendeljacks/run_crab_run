use rcr_core::models::job::Job;
use rcr_core::models::trigger::Trigger;
use rcr_core::Error;
use rcr_db::Database;
use rrule::{RRuleSet, RRuleError, Tz};
use std::sync::Arc;
use tokio::time::{self, Duration};
use chrono::TimeZone;
use tracing::{error, info};

use crate::executor::JobExecutor;

pub struct Scheduler {
    db: Database,
    executor: Arc<JobExecutor>,
    shutdown: Arc<tokio::sync::Notify>,
}

impl Scheduler {
    pub fn new(db: Database, executor: Arc<JobExecutor>) -> Self {
        Self {
            db,
            executor,
            shutdown: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Signal the scheduler to shut down.
    pub fn shutdown(&self) {
        self.shutdown.notify_waiters();
    }

    /// Run the scheduler loop. Checks every minute for jobs that are due.
    pub async fn run(self: Arc<Self>) -> ! {
        let mut interval = time::interval(Duration::from_secs(60));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.check_and_run_scheduled_jobs().await {
                        error!(error = %e, "Scheduler error");
                    }
                }
                _ = self.shutdown.notified() => {
                    info!("Scheduler shutting down");
                    std::process::exit(0);
                }
            }
        }
    }

    async fn check_and_run_scheduled_jobs(&self) -> Result<(), Error> {
        let jobs = self.db.get_enabled_scheduled_jobs().await?;
        let now = chrono::Utc::now();

        for job in jobs {
            if let Some(ref schedule_str) = job.schedule {
                if let Err(e) = self.check_job_schedule(&job, schedule_str, now).await {
                    error!(job_id = %job.id, error = %e, "Failed to check schedule for job");
                }
            }
        }

        Ok(())
    }

    async fn check_job_schedule(
        &self,
        job: &Job,
        schedule_str: &str,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), Error> {
        let rrule_input = if schedule_str.contains("DTSTART") {
            schedule_str.to_string()
        } else {
            format!("DTSTART:{}\nRRULE:{}", now.format("%Y%m%dT%H%M%SZ"), schedule_str)
        };

        let rrule_set: RRuleSet = rrule_input
            .parse()
            .map_err(|e: RRuleError| Error::InvalidRrule(e.to_string()))?;

        let one_min_ago = now - chrono::Duration::seconds(60);
        let now_tz = Tz::Etc__UTC.from_utc_datetime(&now.naive_utc());
        let one_min_ago_tz = Tz::Etc__UTC.from_utc_datetime(&one_min_ago.naive_utc());

        let due = rrule_set
            .before(now_tz)
            .after(one_min_ago_tz)
            .all(1000);

        if !due.dates.is_empty() {
            info!(job_id = %job.id, "Triggering scheduled job");
            self.executor
                .trigger_job(job, Trigger::Schedule)
                .await?;
        }

        Ok(())
    }
}