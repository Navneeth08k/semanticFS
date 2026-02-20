use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unable to read config file: {0}")]
    Read(String),
    #[error("invalid config format: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFsConfig {
    pub workspace: WorkspaceConfig,
    pub filter: FilterConfig,
    pub index: IndexConfig,
    pub embedding: EmbeddingConfig,
    pub retrieval: RetrievalConfig,
    pub fuse_cache: FuseCacheConfig,
    #[serde(default)]
    pub fuse_session: FuseSessionConfig,
    pub map: MapConfig,
    pub policy: PolicyConfig,
    pub observability: ObservabilityConfig,
    pub mcp: McpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub repo_root: String,
    pub mount_point: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub mode: String,
    pub allow_roots: Vec<String>,
    pub deny_globs: Vec<String>,
    pub max_file_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub debounce_ms: u64,
    pub publish_mode: String,
    pub chunk_max_lines: usize,
    pub chunk_overlap_lines: usize,
    #[serde(default = "default_bulk_event_threshold")]
    pub bulk_event_threshold: usize,
    #[serde(default = "default_hotset_max_paths")]
    pub hotset_max_paths: usize,
    #[serde(default = "default_pending_path_report_limit")]
    pub pending_path_report_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model: String,
    pub runtime: String,
    pub quantization: String,
    pub dimension: usize,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfig {
    pub rrf_mode: String,
    pub rrf_k: u32,
    pub topn_symbol: usize,
    pub topn_bm25: usize,
    pub topn_vector: usize,
    pub topn_final: usize,
    pub symbol_exact_boost: f32,
    pub symbol_prefix_boost: f32,
    pub allow_stale: bool,
    #[serde(default = "default_code_path_boost")]
    pub code_path_boost: f32,
    #[serde(default = "default_docs_path_penalty")]
    pub docs_path_penalty: f32,
    #[serde(default = "default_test_path_penalty")]
    pub test_path_penalty: f32,
    #[serde(default = "default_recency_half_life_hours")]
    pub recency_half_life_hours: f32,
    #[serde(default = "default_recency_min_boost")]
    pub recency_min_boost: f32,
    #[serde(default = "default_recency_max_boost")]
    pub recency_max_boost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuseCacheConfig {
    pub max_virtual_inodes: usize,
    pub max_cached_mb: usize,
    pub entry_ttl_ms: u64,
    pub attr_ttl_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuseSessionConfig {
    #[serde(default = "default_fuse_session_mode")]
    pub mode: String,
    #[serde(default = "default_fuse_session_max_entries")]
    pub max_entries: usize,
}

impl Default for FuseSessionConfig {
    fn default() -> Self {
        Self {
            mode: default_fuse_session_mode(),
            max_entries: default_fuse_session_max_entries(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConfig {
    pub base_summary_mode: String,
    pub llm_enrichment: String,
    pub cache_ttl_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub read_only: bool,
    pub deny_secret_paths: bool,
    pub search_result_redaction: bool,
    pub trust_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub metrics_bind: String,
    pub health_bind: String,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub enabled: bool,
    pub mode: String,
}

impl SemanticFsConfig {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let raw = fs::read_to_string(path).map_err(|e| ConfigError::Read(e.to_string()))?;
        toml::from_str::<SemanticFsConfig>(&raw).map_err(|e| ConfigError::Parse(e.to_string()))
    }
}

fn default_bulk_event_threshold() -> usize {
    80
}

fn default_hotset_max_paths() -> usize {
    32
}

fn default_pending_path_report_limit() -> usize {
    20
}

fn default_code_path_boost() -> f32 {
    1.15
}

fn default_docs_path_penalty() -> f32 {
    0.85
}

fn default_test_path_penalty() -> f32 {
    0.95
}

fn default_recency_half_life_hours() -> f32 {
    24.0
}

fn default_recency_min_boost() -> f32 {
    0.85
}

fn default_recency_max_boost() -> f32 {
    1.20
}

fn default_fuse_session_mode() -> String {
    "pinned".to_string()
}

fn default_fuse_session_max_entries() -> usize {
    512
}
