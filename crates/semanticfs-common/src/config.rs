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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuseCacheConfig {
    pub max_virtual_inodes: usize,
    pub max_cached_mb: usize,
    pub entry_ttl_ms: u64,
    pub attr_ttl_ms: u64,
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
