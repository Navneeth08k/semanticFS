use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unable to read config file: {0}")]
    Read(String),
    #[error("invalid config format: {0}")]
    Parse(String),
}

#[derive(Debug, Error)]
pub enum DomainContractError {
    #[error("workspace domain policy contract invalid: {0}")]
    Invalid(String),
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
    #[serde(default)]
    pub scheduler: WorkspaceSchedulerConfig,
    #[serde(default)]
    pub domains: Vec<WorkspaceDomainConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSchedulerConfig {
    #[serde(default)]
    pub max_watch_targets: usize,
}

impl Default for WorkspaceSchedulerConfig {
    fn default() -> Self {
        Self {
            max_watch_targets: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDomainConfig {
    pub id: String,
    pub root: String,
    #[serde(default = "default_domain_trust_label")]
    pub trust_label: String,
    #[serde(default = "default_domain_enabled")]
    pub enabled: bool,
    #[serde(default = "default_domain_watch_enabled")]
    pub watch_enabled: bool,
    #[serde(default = "default_domain_watch_priority")]
    pub watch_priority: i32,
    #[serde(default = "default_domain_max_indexed_files")]
    pub max_indexed_files: usize,
    #[serde(default = "default_domain_allow_hidden_paths")]
    pub allow_hidden_paths: bool,
    #[serde(default)]
    pub allow_roots: Vec<String>,
    #[serde(default)]
    pub deny_globs: Vec<String>,
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
    #[serde(default = "default_asset_path_penalty")]
    pub asset_path_penalty: f32,
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

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceDomainPlan {
    pub schedule_rank: usize,
    pub id: String,
    pub root: String,
    pub trust_label: String,
    pub trust_label_registered: bool,
    pub watch_enabled: bool,
    pub watch_priority: i32,
    pub max_indexed_files: usize,
    pub allow_hidden_paths: bool,
    pub priority_class: String,
    pub root_depth: usize,
    pub effective_allow_roots: Vec<String>,
    pub effective_deny_globs: Vec<String>,
    pub inherits_global_allow_roots: bool,
    pub inherits_global_deny_globs: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceDomainReport {
    pub plan_mode: String,
    pub plans: Vec<WorkspaceDomainPlan>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl WorkspaceDomainReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn ensure_valid(&self) -> Result<(), DomainContractError> {
        if self.is_valid() {
            return Ok(());
        }
        Err(DomainContractError::Invalid(self.errors.join("; ")))
    }
}

impl SemanticFsConfig {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let raw = fs::read_to_string(path).map_err(|e| ConfigError::Read(e.to_string()))?;
        toml::from_str::<SemanticFsConfig>(&raw).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    pub fn effective_workspace_domains(&self) -> Vec<WorkspaceDomainConfig> {
        self.workspace.effective_domains()
    }

    pub fn workspace_domain_report(&self) -> WorkspaceDomainReport {
        let explicit_mode = self.workspace.has_explicit_enabled_domains();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let trust_rank = self
            .policy
            .trust_labels
            .iter()
            .enumerate()
            .map(|(idx, label)| (label.as_str(), idx))
            .collect::<HashMap<_, _>>();
        let mut id_index = HashMap::<String, String>::new();
        let mut root_index = HashMap::<String, String>::new();
        let mut plans = Vec::new();

        for domain in self.effective_workspace_domains() {
            let domain_id = domain.id.trim().to_string();
            let domain_root = normalize_domain_root(&domain.root);

            if domain_id.is_empty() {
                errors.push("workspace domain id must not be empty".to_string());
            } else if let Some(prev_root) = id_index.insert(domain_id.clone(), domain.root.clone())
            {
                errors.push(format!(
                    "workspace domain id `{}` is duplicated across roots `{}` and `{}`",
                    domain_id, prev_root, domain.root
                ));
            }

            if domain_root.is_empty() {
                errors.push(format!(
                    "workspace domain `{}` has an empty root",
                    domain.id
                ));
            } else if let Some(prev_id) = root_index.insert(domain_root.clone(), domain_id.clone())
            {
                errors.push(format!(
                    "workspace domain roots `{}` and `{}` normalize to the same root `{}`",
                    prev_id, domain_id, domain_root
                ));
            }

            let trust_label_registered =
                !explicit_mode || trust_rank.contains_key(domain.trust_label.as_str());
            if !trust_label_registered {
                errors.push(format!(
                    "workspace domain `{}` uses trust_label `{}` which is not listed in policy.trust_labels",
                    domain_id, domain.trust_label
                ));
            }

            let inherits_global_allow_roots = !explicit_mode || domain.allow_roots.is_empty();
            let inherits_global_deny_globs = domain.deny_globs.is_empty();
            let effective_allow_roots = effective_allow_roots(&self.filter, &domain, explicit_mode);
            let effective_deny_globs = merge_globs(&self.filter.deny_globs, &domain.deny_globs);

            validate_policy_patterns(
                &domain_id,
                "allow_roots",
                &effective_allow_roots,
                &mut errors,
            );
            validate_policy_patterns(&domain_id, "deny_globs", &effective_deny_globs, &mut errors);

            plans.push(WorkspaceDomainPlan {
                schedule_rank: 0,
                id: domain_id,
                root: domain_root.clone(),
                trust_label: domain.trust_label,
                trust_label_registered,
                watch_enabled: domain.watch_enabled,
                watch_priority: domain.watch_priority,
                max_indexed_files: domain.max_indexed_files,
                allow_hidden_paths: domain.allow_hidden_paths,
                priority_class: String::new(),
                root_depth: root_depth(&domain_root),
                effective_allow_roots,
                effective_deny_globs,
                inherits_global_allow_roots,
                inherits_global_deny_globs,
            });
        }

        for left_idx in 0..plans.len() {
            for right_idx in (left_idx + 1)..plans.len() {
                let left = &plans[left_idx];
                let right = &plans[right_idx];
                if roots_overlap(&left.root, &right.root) {
                    warnings.push(format!(
                        "workspace domains `{}` and `{}` overlap (`{}` vs `{}`); scheduler will prefer the more specific root first",
                        left.id, right.id, left.root, right.root
                    ));
                }
            }
        }

        plans.sort_by(|left, right| {
            domain_trust_sort_key(explicit_mode, &trust_rank, left)
                .cmp(&domain_trust_sort_key(explicit_mode, &trust_rank, right))
                .then_with(|| right.root_depth.cmp(&left.root_depth))
                .then_with(|| left.root.cmp(&right.root))
                .then_with(|| left.id.cmp(&right.id))
        });

        for (idx, plan) in plans.iter_mut().enumerate() {
            plan.schedule_rank = idx + 1;
            plan.priority_class = domain_priority_class(explicit_mode, &trust_rank, plan);
        }

        WorkspaceDomainReport {
            plan_mode: if explicit_mode {
                "explicit_multi_root".to_string()
            } else {
                "fallback_single_root".to_string()
            },
            plans,
            warnings,
            errors,
        }
    }

    pub fn enforce_workspace_domain_contract(
        &self,
    ) -> Result<WorkspaceDomainReport, DomainContractError> {
        let report = self.workspace_domain_report();
        report.ensure_valid()?;
        Ok(report)
    }
}

impl WorkspaceConfig {
    pub fn effective_domains(&self) -> Vec<WorkspaceDomainConfig> {
        if !self.domains.is_empty() {
            let enabled = self
                .domains
                .iter()
                .filter(|d| d.enabled)
                .cloned()
                .collect::<Vec<_>>();
            if !enabled.is_empty() {
                return enabled;
            }
        }

        vec![WorkspaceDomainConfig {
            id: default_domain_id(),
            root: self.repo_root.clone(),
            trust_label: default_domain_trust_label(),
            enabled: true,
            watch_enabled: default_domain_watch_enabled(),
            watch_priority: default_domain_watch_priority(),
            max_indexed_files: default_domain_max_indexed_files(),
            allow_hidden_paths: default_domain_allow_hidden_paths(),
            allow_roots: vec!["**".to_string()],
            deny_globs: Vec::new(),
        }]
    }

    fn has_explicit_enabled_domains(&self) -> bool {
        !self.domains.is_empty() && self.domains.iter().any(|d| d.enabled)
    }
}

fn effective_allow_roots(
    filter: &FilterConfig,
    domain: &WorkspaceDomainConfig,
    explicit_mode: bool,
) -> Vec<String> {
    if explicit_mode && !domain.allow_roots.is_empty() {
        return dedupe_patterns(&domain.allow_roots);
    }
    if filter.allow_roots.is_empty() {
        return vec!["**".to_string()];
    }
    dedupe_patterns(&filter.allow_roots)
}

fn merge_globs(base: &[String], extra: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for pattern in base.iter().chain(extra.iter()) {
        let trimmed = pattern.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !out.iter().any(|existing: &String| existing == trimmed) {
            out.push(trimmed.to_string());
        }
    }
    out
}

fn dedupe_patterns(patterns: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for pattern in patterns {
        let trimmed = pattern.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !out.iter().any(|existing: &String| existing == trimmed) {
            out.push(trimmed.to_string());
        }
    }
    out
}

fn validate_policy_patterns(
    domain_id: &str,
    field_name: &str,
    patterns: &[String],
    errors: &mut Vec<String>,
) {
    if patterns.is_empty() {
        errors.push(format!(
            "workspace domain `{}` resolved `{}` to an empty pattern set",
            domain_id, field_name
        ));
        return;
    }

    for pattern in patterns {
        if pattern.trim().is_empty() {
            errors.push(format!(
                "workspace domain `{}` contains an empty `{}` entry",
                domain_id, field_name
            ));
            continue;
        }
        if looks_absolute_pattern(pattern) {
            errors.push(format!(
                "workspace domain `{}` contains absolute `{}` entry `{}`; patterns must stay root-relative",
                domain_id, field_name, pattern
            ));
        }
        if has_parent_traversal(pattern) {
            errors.push(format!(
                "workspace domain `{}` contains traversal in `{}` entry `{}`; patterns must stay inside the domain root",
                domain_id, field_name, pattern
            ));
        }
    }
}

fn domain_trust_sort_key(
    explicit_mode: bool,
    trust_rank: &HashMap<&str, usize>,
    plan: &WorkspaceDomainPlan,
) -> usize {
    if !explicit_mode {
        return 0;
    }
    trust_rank
        .get(plan.trust_label.as_str())
        .copied()
        .unwrap_or(usize::MAX / 2)
}

fn domain_priority_class(
    explicit_mode: bool,
    trust_rank: &HashMap<&str, usize>,
    plan: &WorkspaceDomainPlan,
) -> String {
    if !explicit_mode {
        return "fallback_single_root".to_string();
    }
    match trust_rank.get(plan.trust_label.as_str()).copied() {
        Some(rank) => format!("{}:tier_{}", plan.trust_label, rank + 1),
        None => format!("{}:invalid_trust", plan.trust_label),
    }
}

fn normalize_domain_root(raw: &str) -> String {
    let mut root = raw.trim().replace('\\', "/");
    while root.ends_with('/') && root.len() > 1 && !root.ends_with(":/") {
        root.pop();
    }
    root
}

fn root_depth(root: &str) -> usize {
    root.split('/')
        .filter(|segment| !segment.is_empty())
        .count()
}

fn roots_overlap(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let left_prefix = format!("{}/", left);
    let right_prefix = format!("{}/", right);
    left.starts_with(&right_prefix) || right.starts_with(&left_prefix)
}

fn looks_absolute_pattern(pattern: &str) -> bool {
    let raw = pattern.trim();
    if raw.starts_with('/') || raw.starts_with("\\\\") {
        return true;
    }
    let bytes = raw.as_bytes();
    bytes.len() >= 3 && bytes[1] == b':' && (bytes[2] == b'/' || bytes[2] == b'\\')
}

fn has_parent_traversal(pattern: &str) -> bool {
    pattern
        .replace('\\', "/")
        .split('/')
        .any(|segment| segment == "..")
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

fn default_asset_path_penalty() -> f32 {
    0.45
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

fn default_domain_id() -> String {
    "default".to_string()
}

fn default_domain_trust_label() -> String {
    "workspace_default".to_string()
}

fn default_domain_enabled() -> bool {
    true
}

fn default_domain_watch_enabled() -> bool {
    true
}

fn default_domain_watch_priority() -> i32 {
    100
}

fn default_domain_max_indexed_files() -> usize {
    0
}

fn default_domain_allow_hidden_paths() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::{
        EmbeddingConfig, FilterConfig, FuseCacheConfig, FuseSessionConfig, IndexConfig, MapConfig,
        McpConfig, ObservabilityConfig, PolicyConfig, RetrievalConfig, SemanticFsConfig,
        WorkspaceConfig, WorkspaceDomainConfig, WorkspaceSchedulerConfig,
    };

    fn sample_config(domains: Vec<WorkspaceDomainConfig>) -> SemanticFsConfig {
        SemanticFsConfig {
            workspace: WorkspaceConfig {
                repo_root: "/repo".to_string(),
                mount_point: "/mnt/ai".to_string(),
                scheduler: WorkspaceSchedulerConfig::default(),
                domains,
            },
            filter: FilterConfig {
                mode: "repo_first".to_string(),
                allow_roots: vec!["src/**".to_string(), "docs/**".to_string()],
                deny_globs: vec!["**/.git/**".to_string()],
                max_file_mb: 10,
            },
            index: IndexConfig {
                debounce_ms: 500,
                publish_mode: "two_phase".to_string(),
                chunk_max_lines: 120,
                chunk_overlap_lines: 20,
                bulk_event_threshold: 80,
                hotset_max_paths: 32,
                pending_path_report_limit: 20,
            },
            embedding: EmbeddingConfig {
                model: "hash".to_string(),
                runtime: "hash".to_string(),
                quantization: "int8".to_string(),
                dimension: 384,
                batch_size: 64,
            },
            retrieval: RetrievalConfig {
                rrf_mode: "plain".to_string(),
                rrf_k: 60,
                topn_symbol: 10,
                topn_bm25: 20,
                topn_vector: 20,
                topn_final: 5,
                symbol_exact_boost: 2.0,
                symbol_prefix_boost: 1.2,
                allow_stale: false,
                code_path_boost: 1.15,
                docs_path_penalty: 0.85,
                test_path_penalty: 0.95,
                asset_path_penalty: 0.45,
                recency_half_life_hours: 24.0,
                recency_min_boost: 0.85,
                recency_max_boost: 1.20,
            },
            fuse_cache: FuseCacheConfig {
                max_virtual_inodes: 50_000,
                max_cached_mb: 256,
                entry_ttl_ms: 300,
                attr_ttl_ms: 300,
            },
            fuse_session: FuseSessionConfig::default(),
            map: MapConfig {
                base_summary_mode: "deterministic_precompute".to_string(),
                llm_enrichment: "async_optional".to_string(),
                cache_ttl_sec: 3600,
            },
            policy: PolicyConfig {
                read_only: true,
                deny_secret_paths: true,
                search_result_redaction: true,
                trust_labels: vec!["trusted".to_string(), "untrusted".to_string()],
            },
            observability: ObservabilityConfig {
                metrics_bind: "127.0.0.1:9464".to_string(),
                health_bind: "127.0.0.1:9465".to_string(),
                log_level: "info".to_string(),
            },
            mcp: McpConfig {
                enabled: true,
                mode: "minimal".to_string(),
            },
        }
    }

    #[test]
    fn falls_back_to_single_root_domain_when_no_explicit_domains_exist() {
        let cfg = WorkspaceConfig {
            repo_root: "/repo".to_string(),
            mount_point: "/mnt/ai".to_string(),
            scheduler: WorkspaceSchedulerConfig::default(),
            domains: Vec::new(),
        };

        let domains = cfg.effective_domains();
        assert_eq!(domains.len(), 1);
        assert_eq!(domains[0].id, "default");
        assert_eq!(domains[0].root, "/repo");
        assert_eq!(domains[0].trust_label, "workspace_default");
        assert!(domains[0].enabled);
        assert!(domains[0].watch_enabled);
        assert_eq!(domains[0].watch_priority, 100);
        assert_eq!(domains[0].allow_roots, vec!["**".to_string()]);
        assert!(domains[0].deny_globs.is_empty());
    }

    #[test]
    fn returns_only_enabled_explicit_domains() {
        let cfg = WorkspaceConfig {
            repo_root: "/repo".to_string(),
            mount_point: "/mnt/ai".to_string(),
            scheduler: WorkspaceSchedulerConfig::default(),
            domains: vec![
                WorkspaceDomainConfig {
                    id: "code".to_string(),
                    root: "/repo/code".to_string(),
                    trust_label: "trusted".to_string(),
                    enabled: true,
                    watch_enabled: true,
                    watch_priority: 100,
                    max_indexed_files: 0,
                    allow_hidden_paths: false,
                    allow_roots: vec!["src/**".to_string()],
                    deny_globs: vec![],
                },
                WorkspaceDomainConfig {
                    id: "docs".to_string(),
                    root: "/repo/docs".to_string(),
                    trust_label: "untrusted".to_string(),
                    enabled: false,
                    watch_enabled: true,
                    watch_priority: 100,
                    max_indexed_files: 0,
                    allow_hidden_paths: false,
                    allow_roots: vec!["**".to_string()],
                    deny_globs: vec![],
                },
            ],
        };

        let domains = cfg.effective_domains();
        assert_eq!(domains.len(), 1);
        assert_eq!(domains[0].id, "code");
        assert_eq!(domains[0].root, "/repo/code");
    }

    #[test]
    fn fallback_domain_report_is_valid_even_without_registered_default_trust_label() {
        let cfg = sample_config(Vec::new());

        let report = cfg.workspace_domain_report();

        assert!(report.is_valid());
        assert_eq!(report.plan_mode, "fallback_single_root");
        assert_eq!(report.plans.len(), 1);
        assert_eq!(report.plans[0].priority_class, "fallback_single_root");
        assert!(report.plans[0].trust_label_registered);
        assert_eq!(
            report.plans[0].effective_allow_roots,
            vec!["src/**".to_string(), "docs/**".to_string()]
        );
        assert_eq!(
            report.plans[0].effective_deny_globs,
            vec!["**/.git/**".to_string()]
        );
    }

    #[test]
    fn explicit_domain_report_rejects_unknown_trust_and_duplicate_ids() {
        let cfg = sample_config(vec![
            WorkspaceDomainConfig {
                id: "code".to_string(),
                root: "/repo/code".to_string(),
                trust_label: "trusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 100,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["src/**".to_string()],
                deny_globs: vec![],
            },
            WorkspaceDomainConfig {
                id: "code".to_string(),
                root: "/repo/docs".to_string(),
                trust_label: "partner".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 100,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["/abs/**".to_string()],
                deny_globs: vec!["../secret/**".to_string()],
            },
        ]);

        let report = cfg.workspace_domain_report();

        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|msg| msg.contains("workspace domain id `code` is duplicated")));
        assert!(report
            .errors
            .iter()
            .any(|msg| msg.contains("trust_label `partner`")));
        assert!(report
            .errors
            .iter()
            .any(|msg| msg.contains("absolute `allow_roots` entry `/abs/**`")));
        assert!(report
            .errors
            .iter()
            .any(|msg| msg.contains("traversal in `deny_globs` entry `../secret/**`")));
    }

    #[test]
    fn explicit_domain_report_sorts_more_specific_roots_first_and_warns_on_overlap() {
        let cfg = sample_config(vec![
            WorkspaceDomainConfig {
                id: "primary".to_string(),
                root: "/repo".to_string(),
                trust_label: "trusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 100,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec![],
                deny_globs: vec![],
            },
            WorkspaceDomainConfig {
                id: "apps".to_string(),
                root: "/repo/apps".to_string(),
                trust_label: "trusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 100,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["apps/**".to_string()],
                deny_globs: vec!["apps/generated/**".to_string()],
            },
        ]);

        let report = cfg.enforce_workspace_domain_contract().unwrap();

        assert_eq!(report.plan_mode, "explicit_multi_root");
        assert_eq!(report.plans.len(), 2);
        assert_eq!(report.plans[0].id, "apps");
        assert_eq!(report.plans[0].schedule_rank, 1);
        assert_eq!(report.plans[0].priority_class, "trusted:tier_1");
        assert_eq!(
            report.plans[0].effective_deny_globs,
            vec!["**/.git/**".to_string(), "apps/generated/**".to_string()]
        );
        assert_eq!(report.plans[1].id, "primary");
        assert!(report.warnings.iter().any(|msg| msg.contains("overlap")));
    }
}
