use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HitSource {
    Symbol,
    BM25,
    Vector,
    Hybrid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrustLevel {
    Trusted,
    Untrusted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedHit {
    pub rank: u32,
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub file_hash: String,
    pub snapshot_version: u64,
    pub active_version: u64,
    pub score_rrf: f32,
    pub score_symbol: Option<f32>,
    pub score_bm25: Option<f32>,
    pub score_vector: Option<f32>,
    pub source: HitSource,
    pub symbol_kind: Option<String>,
    pub stale: bool,
    pub trust_level: TrustLevel,
    pub why_selected: String,
}
