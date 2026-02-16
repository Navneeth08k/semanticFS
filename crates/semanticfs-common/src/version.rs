use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexVersionState {
    Staging,
    Active,
    Obsolete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexVersionRecord {
    pub version: u64,
    pub state: IndexVersionState,
    pub created_at: String,
    pub published_at: Option<String>,
}
