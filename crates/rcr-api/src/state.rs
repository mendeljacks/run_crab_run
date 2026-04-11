use rcr_db::Database;
use rcr_runner::JobExecutor;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub executor: Arc<JobExecutor>,
}