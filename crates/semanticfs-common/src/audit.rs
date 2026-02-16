use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub ts: String,
    pub actor: String,
    pub op: String,
    pub target: String,
    pub snapshot_version: Option<u64>,
    pub policy_decision: String,
    pub reason: Option<String>,
    pub latency_ms: u32,
    pub result_count: Option<u32>,
}
