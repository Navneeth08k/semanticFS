use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingStatus {
    pub in_progress: bool,
    pub phase: String,
    pub started_unix_ms: u64,
    pub updated_unix_ms: u64,
    pub total_changed_paths: usize,
    pub hotset_total: usize,
    pub deferred_total: usize,
    pub pending_paths: Vec<String>,
    pub message: String,
}
