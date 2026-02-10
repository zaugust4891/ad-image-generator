use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]

pub enum RunEvent {
    Started { run_id: String, total: u64 },
    Log { run_id: String, msg: String },
    Progress { run_id: String, done: u64, total: u64, cost_so_far: f64 },
    Finished { run_id: String },
    Failed { run_id: String, error: String },
}


