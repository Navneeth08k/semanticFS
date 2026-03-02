use anyhow::{anyhow, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use semanticfs_common::{
    AuditEvent, GroundedHit, SemanticFsConfig, TrustLevel, WorkspaceDomainPlan,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub allow: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedPath {
    pub domain_id: String,
    pub trust_label: String,
    pub virtual_path: String,
    pub domain_relative_path: String,
    pub absolute_path: PathBuf,
}

#[derive(Debug, Clone)]
struct DomainRule {
    id: String,
    root: Option<PathBuf>,
    trust_label: String,
    allow_set: GlobSet,
    has_allow_roots: bool,
    deny_set: GlobSet,
    uses_virtual_prefix: bool,
}

#[derive(Debug, Clone)]
pub struct PolicyGuard {
    domains: Vec<DomainRule>,
    explicit_multi_root: bool,
    secret_patterns: Vec<Regex>,
}

impl PolicyGuard {
    pub fn new(allow_roots: &[String], deny_globs: &[String]) -> Result<Self> {
        let allow_set = build_glob_set(allow_roots)?;
        let deny_set = build_glob_set(deny_globs)?;

        Ok(Self {
            domains: vec![DomainRule {
                id: "default".to_string(),
                root: None,
                trust_label: "trusted".to_string(),
                allow_set,
                has_allow_roots: !allow_roots.is_empty(),
                deny_set,
                uses_virtual_prefix: false,
            }],
            explicit_multi_root: false,
            secret_patterns: default_secret_patterns()?,
        })
    }

    pub fn from_config(cfg: &SemanticFsConfig) -> Result<Self> {
        let report = cfg
            .enforce_workspace_domain_contract()
            .map_err(|err| anyhow!(err.to_string()))?;
        let explicit_multi_root = report.plan_mode == "explicit_multi_root";
        let mut domains = Vec::with_capacity(report.plans.len());

        for plan in &report.plans {
            domains.push(DomainRule::from_plan(plan, explicit_multi_root)?);
        }

        Ok(Self {
            domains,
            explicit_multi_root,
            secret_patterns: default_secret_patterns()?,
        })
    }

    pub fn is_explicit_multi_root(&self) -> bool {
        self.explicit_multi_root
    }

    pub fn watch_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        for domain in &self.domains {
            let Some(root) = &domain.root else {
                continue;
            };
            if !roots.iter().any(|existing: &PathBuf| existing == root) {
                roots.push(root.clone());
            }
        }
        roots
    }

    pub fn domain_ids(&self) -> Vec<String> {
        self.domains.iter().map(|d| d.id.clone()).collect()
    }

    pub fn domain_schedule_rank(&self, domain_id: &str) -> usize {
        self.domains
            .iter()
            .position(|d| d.id == domain_id)
            .map(|idx| idx + 1)
            .unwrap_or(self.domains.len() + 1)
    }

    pub fn resolve_disk_path(&self, absolute_path: &Path) -> Option<ResolvedPath> {
        for domain in &self.domains {
            let Some(root) = &domain.root else {
                continue;
            };
            let Ok(relative) = absolute_path.strip_prefix(root) else {
                continue;
            };
            let domain_relative_path = normalize_relative_path(relative);
            let virtual_path = domain.virtual_path_for(&domain_relative_path);
            return Some(ResolvedPath {
                domain_id: domain.id.clone(),
                trust_label: domain.trust_label.clone(),
                virtual_path,
                domain_relative_path,
                absolute_path: absolute_path.to_path_buf(),
            });
        }
        None
    }

    pub fn resolve_virtual_path(&self, virtual_path: &str) -> Option<ResolvedPath> {
        let normalized = normalize_virtual_path(virtual_path);

        if self.explicit_multi_root {
            let (domain_id, domain_relative_path) = split_domain_path(&normalized)?;
            let domain = self.domains.iter().find(|d| d.id == domain_id)?;
            let root = domain.root.as_ref()?;
            if has_parent_traversal(&domain_relative_path) {
                return None;
            }
            let absolute_path = join_domain_path(root, &domain_relative_path);
            let owner = self.resolve_disk_path(&absolute_path)?;
            if owner.domain_id != domain.id {
                return None;
            }
            return Some(ResolvedPath {
                domain_id: owner.domain_id,
                trust_label: owner.trust_label,
                virtual_path: owner.virtual_path,
                domain_relative_path: owner.domain_relative_path,
                absolute_path,
            });
        }

        let domain = self.domains.first()?;
        if has_parent_traversal(&normalized) {
            return None;
        }
        let absolute_path = if let Some(root) = &domain.root {
            join_domain_path(root, &normalized)
        } else {
            PathBuf::from(&normalized)
        };
        Some(ResolvedPath {
            domain_id: domain.id.clone(),
            trust_label: domain.trust_label.clone(),
            virtual_path: normalized.clone(),
            domain_relative_path: normalized,
            absolute_path,
        })
    }

    pub fn should_index_path(&self, virtual_path: &str) -> PolicyDecision {
        let Some(resolved) = self.resolve_virtual_path(virtual_path) else {
            return PolicyDecision {
                allow: false,
                reason: Some("path is outside configured domain ownership".to_string()),
            };
        };
        self.should_index_resolved(&resolved)
    }

    pub fn should_index_resolved(&self, resolved: &ResolvedPath) -> PolicyDecision {
        let Some(domain) = self.domains.iter().find(|d| d.id == resolved.domain_id) else {
            return PolicyDecision {
                allow: false,
                reason: Some("domain not found for resolved path".to_string()),
            };
        };

        if domain.deny_set.is_match(&resolved.domain_relative_path) {
            return PolicyDecision {
                allow: false,
                reason: Some(format!("path matched deny list for domain `{}`", domain.id)),
            };
        }

        if !domain.has_allow_roots || domain.allow_set.is_match(&resolved.domain_relative_path) {
            return PolicyDecision {
                allow: true,
                reason: None,
            };
        }

        PolicyDecision {
            allow: false,
            reason: Some(format!(
                "path not included in allow roots for domain `{}`",
                domain.id
            )),
        }
    }

    pub fn trust_level_for_virtual_path(&self, virtual_path: &str) -> TrustLevel {
        self.resolve_virtual_path(virtual_path)
            .map(|resolved| trust_level_from_label(&resolved.trust_label))
            .unwrap_or(TrustLevel::Trusted)
    }

    pub fn contains_secret(&self, content: &str) -> bool {
        self.secret_patterns.iter().any(|p| p.is_match(content))
            || high_entropy_token_present(content)
    }

    pub fn redact_sensitive_hits(&self, hits: Vec<GroundedHit>) -> Vec<GroundedHit> {
        hits.into_iter()
            .filter(|hit| self.should_index_path(&hit.path).allow)
            .collect()
    }

    pub fn build_audit(
        &self,
        actor: &str,
        op: &str,
        target: &str,
        snapshot_version: Option<u64>,
        policy_decision: &str,
        reason: Option<String>,
        latency_ms: u32,
        result_count: Option<u32>,
    ) -> AuditEvent {
        AuditEvent {
            ts: chrono::Utc::now().to_rfc3339(),
            actor: actor.to_string(),
            op: op.to_string(),
            target: target.to_string(),
            snapshot_version,
            policy_decision: policy_decision.to_string(),
            reason,
            latency_ms,
            result_count,
        }
    }
}

impl DomainRule {
    fn from_plan(plan: &WorkspaceDomainPlan, explicit_multi_root: bool) -> Result<Self> {
        let root = PathBuf::from(&plan.root)
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(&plan.root));
        Ok(Self {
            id: plan.id.clone(),
            root: Some(root),
            trust_label: plan.trust_label.clone(),
            allow_set: build_glob_set(&plan.effective_allow_roots)?,
            has_allow_roots: !plan.effective_allow_roots.is_empty(),
            deny_set: build_glob_set(&plan.effective_deny_globs)?,
            uses_virtual_prefix: explicit_multi_root,
        })
    }

    fn virtual_path_for(&self, domain_relative_path: &str) -> String {
        if !self.uses_virtual_prefix {
            return domain_relative_path.to_string();
        }
        if domain_relative_path.is_empty() {
            return self.id.clone();
        }
        format!("{}/{}", self.id, domain_relative_path)
    }
}

fn build_glob_set(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}

fn default_secret_patterns() -> Result<Vec<Regex>> {
    Ok(vec![
        Regex::new(r#"(?i)api[_-]?key\s*[:=]\s*["']?[a-z0-9_\-]{12,}"#)?,
        Regex::new(r#"(?i)secret\s*[:=]\s*["']?[a-z0-9_\-]{12,}"#)?,
        Regex::new(r#"(?i)token\s*[:=]\s*["']?[a-z0-9_\-]{12,}"#)?,
        Regex::new(r"-----BEGIN (RSA|EC|OPENSSH) PRIVATE KEY-----")?,
        Regex::new(r"ghp_[A-Za-z0-9]{20,}")?,
    ])
}

fn split_domain_path(path: &str) -> Option<(String, String)> {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return None;
    }

    let mut parts = trimmed.splitn(2, '/');
    let domain_id = parts.next()?.trim();
    if domain_id.is_empty() {
        return None;
    }
    let rest = parts.next().unwrap_or("").trim_matches('/').to_string();
    Some((domain_id.to_string(), rest))
}

fn join_domain_path(root: &Path, domain_relative_path: &str) -> PathBuf {
    let mut out = PathBuf::from(root);
    if !domain_relative_path.is_empty() {
        for segment in domain_relative_path.split('/') {
            if segment.is_empty() {
                continue;
            }
            out.push(segment);
        }
    }
    out
}

fn normalize_relative_path(path: &Path) -> String {
    normalize_virtual_path(&path.to_string_lossy())
}

fn normalize_virtual_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    normalized.trim_matches('/').to_string()
}

fn has_parent_traversal(path: &str) -> bool {
    path.replace('\\', "/")
        .split('/')
        .any(|segment| segment == "..")
}

fn trust_level_from_label(label: &str) -> TrustLevel {
    if label.eq_ignore_ascii_case("trusted") {
        TrustLevel::Trusted
    } else {
        TrustLevel::Untrusted
    }
}

fn high_entropy_token_present(content: &str) -> bool {
    content
        .split_whitespace()
        .filter(|token| token.len() >= 20)
        .any(|token| shannon_entropy(token) > 4.2)
}

fn shannon_entropy(input: &str) -> f64 {
    use std::collections::HashMap;

    let mut counts = HashMap::new();
    for c in input.chars() {
        *counts.entry(c).or_insert(0usize) += 1;
    }
    let len = input.len() as f64;
    counts
        .into_values()
        .map(|count| {
            let p = count as f64 / len;
            -p * p.log2()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use semanticfs_common::{
        EmbeddingConfig, FilterConfig, FuseCacheConfig, FuseSessionConfig, IndexConfig, MapConfig,
        McpConfig, ObservabilityConfig, PolicyConfig, RetrievalConfig, WorkspaceConfig,
        WorkspaceDomainConfig,
    };

    fn workspace_root() -> PathBuf {
        std::env::current_dir()
            .unwrap()
            .join("..")
            .join("..")
            .canonicalize()
            .unwrap()
    }

    fn sample_config(domains: Vec<WorkspaceDomainConfig>) -> SemanticFsConfig {
        let repo_root = workspace_root();
        SemanticFsConfig {
            workspace: WorkspaceConfig {
                repo_root: repo_root.to_string_lossy().to_string(),
                mount_point: "/mnt/ai".to_string(),
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
    fn deny_glob_blocks_path() {
        let guard = PolicyGuard::new(&["src/**".into()], &["**/.git/**".into()]).unwrap();
        let decision = guard.should_index_path("foo/.git/config");
        assert!(!decision.allow);
    }

    #[test]
    fn entropy_detector_finds_secretish_tokens() {
        let guard = PolicyGuard::new(&[], &[]).unwrap();
        assert!(guard.contains_secret("token=AKJH1234ASDF1234ZXCV9999QWER"));
    }

    #[test]
    fn explicit_multi_root_uses_domain_prefixed_virtual_paths() {
        let cfg = sample_config(vec![
            WorkspaceDomainConfig {
                id: "code".to_string(),
                root: workspace_root().join("crates").to_string_lossy().to_string(),
                trust_label: "trusted".to_string(),
                enabled: true,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec!["**/target/**".to_string()],
            },
            WorkspaceDomainConfig {
                id: "docs".to_string(),
                root: workspace_root().join("docs").to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
        ]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let docs_file = workspace_root().join("docs").join("new-chat-handoff.md");
        let resolved = guard.resolve_disk_path(&docs_file).unwrap();

        assert_eq!(resolved.domain_id, "docs");
        assert_eq!(resolved.virtual_path, "docs/new-chat-handoff.md");
        assert_eq!(guard.trust_level_for_virtual_path(&resolved.virtual_path), TrustLevel::Untrusted);
    }

    #[test]
    fn overlapping_domains_prefer_more_specific_root() {
        let cfg = sample_config(vec![
            WorkspaceDomainConfig {
                id: "primary".to_string(),
                root: workspace_root().to_string_lossy().to_string(),
                trust_label: "trusted".to_string(),
                enabled: true,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
            WorkspaceDomainConfig {
                id: "docs".to_string(),
                root: workspace_root().join("docs").to_string_lossy().to_string(),
                trust_label: "trusted".to_string(),
                enabled: true,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
        ]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let docs_file = workspace_root().join("docs").join("new-chat-handoff.md");
        let resolved = guard.resolve_disk_path(&docs_file).unwrap();

        assert_eq!(resolved.domain_id, "docs");
        assert_eq!(resolved.virtual_path, "docs/new-chat-handoff.md");
        assert!(guard.resolve_virtual_path("primary/docs/new-chat-handoff.md").is_none());
        assert!(guard.resolve_virtual_path("docs/new-chat-handoff.md").is_some());
    }

    #[test]
    fn schedule_rank_matches_domain_order() {
        let cfg = sample_config(vec![
            WorkspaceDomainConfig {
                id: "code".to_string(),
                root: workspace_root().join("crates").to_string_lossy().to_string(),
                trust_label: "trusted".to_string(),
                enabled: true,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
            WorkspaceDomainConfig {
                id: "docs".to_string(),
                root: workspace_root().join("docs").to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
        ]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();

        assert_eq!(guard.domain_schedule_rank("code"), 1);
        assert_eq!(guard.domain_schedule_rank("docs"), 2);
        assert_eq!(guard.domain_schedule_rank("missing"), 3);
    }
}
