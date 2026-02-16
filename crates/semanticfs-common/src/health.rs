use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIndex {
    pub active_version: u64,
    pub staging_version: Option<u64>,
    pub queue_depth: usize,
    pub lag_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub live: bool,
    pub ready: bool,
    pub uptime_seconds: u64,
    pub rss_mb: u64,
    pub index: HealthIndex,
}
