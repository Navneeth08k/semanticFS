use anyhow::{anyhow, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use semanticfs_common::{
    AuditEvent, GroundedHit, SemanticFsConfig, TrustLevel, WorkspaceDomainPlan,
};
use std::{fs, path::{Path, PathBuf}};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchTarget {
    pub path: PathBuf,
    pub recursive: bool,
    pub priority: i32,
}

#[derive(Debug, Clone)]
struct DomainRule {
    id: String,
    root: Option<PathBuf>,
    root_is_hidden: bool,
    trust_label: String,
    watch_enabled: bool,
    watch_priority: i32,
    max_indexed_files: usize,
    allow_hidden_paths: bool,
    allow_roots_patterns: Vec<String>,
    allow_set: GlobSet,
    has_allow_roots: bool,
    deny_set: GlobSet,
    uses_virtual_prefix: bool,
}

#[derive(Debug, Clone)]
pub struct PolicyGuard {
    domains: Vec<DomainRule>,
    explicit_multi_root: bool,
    max_watch_targets: usize,
    deny_secret_paths: bool,
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
                root_is_hidden: false,
                trust_label: "trusted".to_string(),
                watch_enabled: true,
                watch_priority: 100,
                max_indexed_files: 0,
                allow_hidden_paths: true,
                allow_roots_patterns: allow_roots.to_vec(),
                allow_set,
                has_allow_roots: !allow_roots.is_empty(),
                deny_set,
                uses_virtual_prefix: false,
            }],
            explicit_multi_root: false,
            max_watch_targets: 0,
            deny_secret_paths: false,
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
            max_watch_targets: cfg.workspace.scheduler.max_watch_targets,
            deny_secret_paths: cfg.policy.deny_secret_paths,
            secret_patterns: default_secret_patterns()?,
        })
    }

    pub fn is_explicit_multi_root(&self) -> bool {
        self.explicit_multi_root
    }

    pub fn watch_roots(&self) -> Vec<PathBuf> {
        self.watch_targets()
            .into_iter()
            .map(|target| target.path)
            .collect()
    }

    pub fn scan_targets(&self) -> Vec<WatchTarget> {
        let mut targets = Vec::new();
        for domain in &self.domains {
            targets.extend(domain.scan_targets());
        }
        prune_watch_targets(targets, 0)
    }

    pub fn watch_targets(&self) -> Vec<WatchTarget> {
        let mut targets = Vec::new();
        for domain in &self.domains {
            targets.extend(domain.watch_targets());
        }
        prune_watch_targets(targets, self.max_watch_targets)
    }

    pub fn scan_target_count(&self) -> usize {
        self.scan_targets().len()
    }

    pub fn watch_target_count(&self) -> usize {
        self.watch_targets().len()
    }

    pub fn watch_enabled_domain_count(&self) -> usize {
        self.domains.iter().filter(|domain| domain.watch_enabled).count()
    }

    pub fn budgeted_domain_count(&self) -> usize {
        self.domains
            .iter()
            .filter(|domain| domain.max_indexed_files > 0)
            .count()
    }

    pub fn domain_index_budget(&self, domain_id: &str) -> usize {
        self.domains
            .iter()
            .find(|domain| domain.id == domain_id)
            .map(|domain| domain.max_indexed_files)
            .unwrap_or(0)
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

        if self.deny_secret_paths
            && !domain.allow_hidden_paths
            && (domain.root_is_hidden || contains_hidden_path_segment(&resolved.domain_relative_path))
        {
            return PolicyDecision {
                allow: false,
                reason: Some(format!(
                    "hidden path blocked by deny_secret_paths for domain `{}`",
                    domain.id
                )),
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
        let root_is_hidden = root
            .file_name()
            .and_then(|segment| segment.to_str())
            .map(|segment| segment.starts_with('.'))
            .unwrap_or(false);
        Ok(Self {
            id: plan.id.clone(),
            root: Some(root),
            root_is_hidden,
            trust_label: plan.trust_label.clone(),
            watch_enabled: plan.watch_enabled,
            watch_priority: plan.watch_priority,
            max_indexed_files: plan.max_indexed_files,
            allow_hidden_paths: plan.allow_hidden_paths,
            allow_roots_patterns: plan.effective_allow_roots.clone(),
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

    fn scan_targets(&self) -> Vec<WatchTarget> {
        let Some(root) = &self.root else {
            return Vec::new();
        };

        if self.allow_roots_patterns.is_empty() {
            return vec![WatchTarget {
                path: root.clone(),
                recursive: true,
                priority: self.watch_priority,
            }];
        }

        let mut targets = Vec::with_capacity(self.allow_roots_patterns.len());
        for pattern in &self.allow_roots_patterns {
            if is_broad_root_pattern(pattern) {
                if let Some(fanned_out) = fan_out_root_targets(root, self.watch_priority) {
                    return fanned_out;
                }
            }
            let Some(target) = watch_target_for_pattern(root, pattern, self.watch_priority) else {
                return vec![WatchTarget {
                    path: root.clone(),
                    recursive: true,
                    priority: self.watch_priority,
                }];
            };
            targets.push(target);
        }
        prune_watch_targets(targets, 0)
    }

    fn watch_targets(&self) -> Vec<WatchTarget> {
        if !self.watch_enabled {
            return Vec::new();
        }
        self.scan_targets()
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

fn contains_hidden_path_segment(path: &str) -> bool {
    path.replace('\\', "/").split('/').any(|segment| {
        let trimmed = segment.trim();
        !trimmed.is_empty() && trimmed.starts_with('.')
    })
}

fn is_exact_watch_pattern(pattern: &str) -> bool {
    !pattern.contains('*')
        && !pattern.contains('?')
        && !pattern.contains('[')
        && !pattern.contains('{')
}

fn watch_target_for_pattern(root: &Path, pattern: &str, priority: i32) -> Option<WatchTarget> {
    let normalized = normalize_virtual_path(pattern);

    if is_exact_watch_pattern(&normalized) {
        let path = join_domain_path(root, &normalized);
        return Some(WatchTarget {
            recursive: path.is_dir(),
            path,
            priority,
        });
    }

    if !has_glob_tokens(&normalized) {
        return None;
    }

    if normalized == "**" || normalized == "**/*" {
        return Some(WatchTarget {
            path: root.to_path_buf(),
            recursive: true,
            priority,
        });
    }

    if let Some(prefix) = normalized.strip_suffix("/**") {
        if !prefix.is_empty() && !has_glob_tokens(prefix) {
            return Some(WatchTarget {
                path: join_domain_path(root, prefix),
                recursive: true,
                priority,
            });
        }
    }

    if !normalized.contains('/') {
        return Some(WatchTarget {
            path: root.to_path_buf(),
            recursive: false,
            priority,
        });
    }

    if let Some(prefix) = normalized.split("/**/").next() {
        if !prefix.is_empty() && normalized.contains("/**/") && !has_glob_tokens(prefix) {
            return Some(WatchTarget {
                path: join_domain_path(root, prefix),
                recursive: true,
                priority,
            });
        }
    }

    if let Some((parent, _leaf)) = normalized.rsplit_once('/') {
        if !parent.is_empty() && !has_glob_tokens(parent) {
            return Some(WatchTarget {
                path: join_domain_path(root, parent),
                recursive: false,
                priority,
            });
        }
    }

    None
}

fn has_glob_tokens(pattern: &str) -> bool {
    pattern.contains('*')
        || pattern.contains('?')
        || pattern.contains('[')
        || pattern.contains('{')
}

fn is_broad_root_pattern(pattern: &str) -> bool {
    let normalized = normalize_virtual_path(pattern);
    normalized == "**" || normalized == "**/*"
}

fn fan_out_root_targets(root: &Path, priority: i32) -> Option<Vec<WatchTarget>> {
    let Ok(entries) = fs::read_dir(root) else {
        return None;
    };

    let mut items = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    if items.is_empty() {
        return None;
    }

    items.sort_by(|left, right| {
        let left_name = left
            .file_name()
            .map(|name| name.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        let right_name = right
            .file_name()
            .map(|name| name.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        left_name.cmp(&right_name).then_with(|| left.cmp(right))
    });

    let mut targets = Vec::with_capacity(items.len());
    for path in items {
        targets.push(WatchTarget {
            recursive: path.is_dir(),
            path,
            priority,
        });
    }

    Some(prune_watch_targets(targets, 0))
}

fn prune_watch_targets(
    mut targets: Vec<WatchTarget>,
    max_watch_targets: usize,
) -> Vec<WatchTarget> {
    targets.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.recursive.cmp(&right.recursive))
            .then_with(|| {
                right
                    .path
                    .components()
                    .count()
                    .cmp(&left.path.components().count())
            })
            .then_with(|| left.path.cmp(&right.path))
    });

    let mut out = Vec::new();
    for target in targets {
        if out.iter().any(|existing: &WatchTarget| {
            existing.recursive && is_same_or_ancestor(&existing.path, &target.path)
        }) {
            continue;
        }
        if let Some(existing) = out.iter_mut().find(|existing| existing.path == target.path) {
            existing.recursive |= target.recursive;
            existing.priority = existing.priority.max(target.priority);
            continue;
        }
        out.push(target);
        if max_watch_targets > 0 && out.len() >= max_watch_targets {
            break;
        }
    }
    out
}

fn is_same_or_ancestor(parent: &Path, child: &Path) -> bool {
    child == parent || child.starts_with(parent)
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
        WorkspaceDomainConfig, WorkspaceSchedulerConfig,
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
                root: workspace_root()
                    .join("crates")
                    .to_string_lossy()
                    .to_string(),
                trust_label: "trusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 100,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec!["**/target/**".to_string()],
            },
            WorkspaceDomainConfig {
                id: "docs".to_string(),
                root: workspace_root().join("docs").to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 50,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
        ]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let docs_file = workspace_root().join("docs").join("new-chat-handoff.md");
        let resolved = guard.resolve_disk_path(&docs_file).unwrap();

        assert_eq!(resolved.domain_id, "docs");
        assert_eq!(resolved.virtual_path, "docs/new-chat-handoff.md");
        assert_eq!(
            guard.trust_level_for_virtual_path(&resolved.virtual_path),
            TrustLevel::Untrusted
        );
    }

    #[test]
    fn overlapping_domains_prefer_more_specific_root() {
        let cfg = sample_config(vec![
            WorkspaceDomainConfig {
                id: "primary".to_string(),
                root: workspace_root().to_string_lossy().to_string(),
                trust_label: "trusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 100,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
            WorkspaceDomainConfig {
                id: "docs".to_string(),
                root: workspace_root().join("docs").to_string_lossy().to_string(),
                trust_label: "trusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 50,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
        ]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let docs_file = workspace_root().join("docs").join("new-chat-handoff.md");
        let resolved = guard.resolve_disk_path(&docs_file).unwrap();

        assert_eq!(resolved.domain_id, "docs");
        assert_eq!(resolved.virtual_path, "docs/new-chat-handoff.md");
        assert!(guard
            .resolve_virtual_path("primary/docs/new-chat-handoff.md")
            .is_none());
        assert!(guard
            .resolve_virtual_path("docs/new-chat-handoff.md")
            .is_some());
    }

    #[test]
    fn schedule_rank_matches_domain_order() {
        let cfg = sample_config(vec![
            WorkspaceDomainConfig {
                id: "code".to_string(),
                root: workspace_root()
                    .join("crates")
                    .to_string_lossy()
                    .to_string(),
                trust_label: "trusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 100,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
            WorkspaceDomainConfig {
                id: "docs".to_string(),
                root: workspace_root().join("docs").to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 50,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
        ]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();

        assert_eq!(guard.domain_schedule_rank("code"), 1);
        assert_eq!(guard.domain_schedule_rank("docs"), 2);
        assert_eq!(guard.domain_schedule_rank("missing"), 3);
    }

    #[test]
    fn watch_targets_use_exact_allow_roots_before_broad_root_watch() {
        let root = workspace_root();
        let cfg = sample_config(vec![
            WorkspaceDomainConfig {
                id: "workspace_meta".to_string(),
                root: root.to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 90,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["README.md".to_string(), "Cargo.toml".to_string()],
                deny_globs: vec![],
            },
            WorkspaceDomainConfig {
                id: "code".to_string(),
                root: root.join("crates").to_string_lossy().to_string(),
                trust_label: "trusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 100,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
        ]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let targets = guard.watch_targets();

        assert!(targets
            .iter()
            .any(|target| target.path.ends_with("README.md") && !target.recursive));
        assert!(targets
            .iter()
            .any(|target| target.path.ends_with("Cargo.toml") && !target.recursive));
        assert!(targets
            .iter()
            .any(|target| target.path.starts_with(root.join("crates")) && target.recursive));
        assert!(!targets
            .iter()
            .any(|target| target.path == root && target.recursive));
    }

    #[test]
    fn watch_targets_respect_max_watch_target_limit() {
        let root = workspace_root();
        let mut cfg = sample_config(vec![
            WorkspaceDomainConfig {
                id: "workspace_meta".to_string(),
                root: root.to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 100,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["README.md".to_string(), "Cargo.toml".to_string()],
                deny_globs: vec![],
            },
            WorkspaceDomainConfig {
                id: "docs".to_string(),
                root: root.join("docs").to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 10,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
        ]);
        cfg.workspace.scheduler.max_watch_targets = 2;

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let targets = guard.watch_targets();

        assert_eq!(targets.len(), 2);
    }

    #[test]
    fn watch_targets_skip_disabled_domains() {
        let root = workspace_root();
        let cfg = sample_config(vec![
            WorkspaceDomainConfig {
                id: "workspace_meta".to_string(),
                root: root.to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                watch_enabled: false,
                watch_priority: 100,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["README.md".to_string()],
                deny_globs: vec![],
            },
            WorkspaceDomainConfig {
                id: "code".to_string(),
                root: root.join("crates").to_string_lossy().to_string(),
                trust_label: "trusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 50,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
        ]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let targets = guard.watch_targets();

        assert!(!targets.is_empty());
        assert!(targets
            .iter()
            .all(|target| target.path.starts_with(root.join("crates"))));
        assert!(targets.iter().all(|target| target.priority == 50));

        let scan_targets = guard.scan_targets();
        assert!(scan_targets
            .iter()
            .any(|target| target.path.ends_with("README.md")));
        assert!(scan_targets
            .iter()
            .any(|target| target.path.starts_with(root.join("crates"))));
        assert_eq!(guard.watch_enabled_domain_count(), 1);
        assert_eq!(guard.watch_target_count(), targets.len());
        assert!(guard.scan_target_count() >= targets.len() + 1);
    }

    #[test]
    fn budgeted_domain_count_only_counts_capped_domains() {
        let root = workspace_root();
        let cfg = sample_config(vec![
            WorkspaceDomainConfig {
                id: "docs".to_string(),
                root: root.join("docs").to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 10,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
            WorkspaceDomainConfig {
                id: "inventory".to_string(),
                root: root.join("inventory").to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                watch_enabled: false,
                watch_priority: 5,
                max_indexed_files: 2,
                allow_hidden_paths: false,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
        ]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();

        assert_eq!(guard.budgeted_domain_count(), 1);
    }

    #[test]
    fn watch_targets_prioritize_higher_priority_domains_under_budget() {
        let root = workspace_root();
        let mut cfg = sample_config(vec![
            WorkspaceDomainConfig {
                id: "docs".to_string(),
                root: root.join("docs").to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 10,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
            WorkspaceDomainConfig {
                id: "workspace_meta".to_string(),
                root: root.to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 90,
                max_indexed_files: 0,
                allow_hidden_paths: false,
                allow_roots: vec!["README.md".to_string()],
                deny_globs: vec![],
            },
        ]);
        cfg.workspace.scheduler.max_watch_targets = 1;

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let targets = guard.watch_targets();

        assert_eq!(targets.len(), 1);
        assert!(targets[0].path.ends_with("README.md"));
        assert!(!targets[0].recursive);
        assert_eq!(targets[0].priority, 90);
    }

    #[test]
    fn scan_targets_decompose_constrained_patterns_into_bounded_targets() {
        let root = workspace_root();
        let cfg = sample_config(vec![WorkspaceDomainConfig {
            id: "home_like".to_string(),
            root: root.to_string_lossy().to_string(),
            trust_label: "untrusted".to_string(),
            enabled: true,
            watch_enabled: false,
            watch_priority: 25,
            max_indexed_files: 0,
            allow_hidden_paths: false,
            allow_roots: vec![
                "*.url".to_string(),
                "rules/*.rules".to_string(),
                "skills/**/*.md".to_string(),
            ],
            deny_globs: vec![],
        }]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let targets = guard.scan_targets();

        assert!(targets.iter().any(|target| target.path == root && !target.recursive));
        assert!(targets
            .iter()
            .any(|target| target.path == root.join("rules") && !target.recursive));
        assert!(targets
            .iter()
            .any(|target| target.path == root.join("skills") && target.recursive));
        assert!(!targets
            .iter()
            .any(|target| target.path == root && target.recursive));
    }

    #[test]
    fn broad_root_pattern_fans_out_into_top_level_targets() {
        let root = workspace_root();
        let cfg = sample_config(vec![WorkspaceDomainConfig {
            id: "home_like".to_string(),
            root: root.to_string_lossy().to_string(),
            trust_label: "untrusted".to_string(),
            enabled: true,
            watch_enabled: false,
            watch_priority: 25,
            max_indexed_files: 0,
            allow_hidden_paths: false,
            allow_roots: vec!["**".to_string()],
            deny_globs: vec![],
        }]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let targets = guard.scan_targets();

        assert!(!targets.is_empty());
        assert!(!targets.iter().any(|target| target.path == root && target.recursive));
        assert!(targets.iter().all(|target| target.path.starts_with(&root)));
    }

    #[test]
    fn domain_index_budget_matches_config() {
        let root = workspace_root();
        let cfg = sample_config(vec![
            WorkspaceDomainConfig {
                id: "docs".to_string(),
                root: root.join("docs").to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 10,
                max_indexed_files: 3,
                allow_hidden_paths: false,
                allow_roots: vec!["**".to_string()],
                deny_globs: vec![],
            },
            WorkspaceDomainConfig {
                id: "workspace_meta".to_string(),
                root: root.to_string_lossy().to_string(),
                trust_label: "untrusted".to_string(),
                enabled: true,
                watch_enabled: true,
                watch_priority: 90,
                max_indexed_files: 1,
                allow_hidden_paths: false,
                allow_roots: vec!["README.md".to_string()],
                deny_globs: vec![],
            },
        ]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();

        assert_eq!(guard.domain_index_budget("docs"), 3);
        assert_eq!(guard.domain_index_budget("workspace_meta"), 1);
        assert_eq!(guard.domain_index_budget("missing"), 0);
    }

    #[test]
    fn deny_secret_paths_blocks_hidden_paths_without_domain_override() {
        let root = workspace_root();
        let cfg = sample_config(vec![WorkspaceDomainConfig {
            id: "workspace_meta".to_string(),
            root: root.to_string_lossy().to_string(),
            trust_label: "untrusted".to_string(),
            enabled: true,
            watch_enabled: true,
            watch_priority: 90,
            max_indexed_files: 0,
            allow_hidden_paths: false,
            allow_roots: vec!["**".to_string()],
            deny_globs: vec![],
        }]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let decision = guard.should_index_path("workspace_meta/.hidden/notes.txt");

        assert!(!decision.allow);
        assert!(decision
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("hidden path blocked"));
    }

    #[test]
    fn deny_secret_paths_allows_hidden_paths_with_domain_override() {
        let root = workspace_root();
        let cfg = sample_config(vec![WorkspaceDomainConfig {
            id: "workspace_meta".to_string(),
            root: root.to_string_lossy().to_string(),
            trust_label: "untrusted".to_string(),
            enabled: true,
            watch_enabled: true,
            watch_priority: 90,
            max_indexed_files: 0,
            allow_hidden_paths: true,
            allow_roots: vec![".hidden/notes.txt".to_string()],
            deny_globs: vec![],
        }]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let decision = guard.should_index_path("workspace_meta/.hidden/notes.txt");

        assert!(decision.allow);
    }

    #[test]
    fn deny_secret_paths_blocks_hidden_domain_root_without_override() {
        let root = workspace_root().join(".hidden-home");
        let cfg = sample_config(vec![WorkspaceDomainConfig {
            id: "hidden_home".to_string(),
            root: root.to_string_lossy().to_string(),
            trust_label: "untrusted".to_string(),
            enabled: true,
            watch_enabled: true,
            watch_priority: 90,
            max_indexed_files: 0,
            allow_hidden_paths: false,
            allow_roots: vec!["config.toml".to_string()],
            deny_globs: vec![],
        }]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let decision = guard.should_index_path("hidden_home/config.toml");

        assert!(!decision.allow);
        assert!(decision
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("hidden path blocked"));
    }

    #[test]
    fn explicit_domain_exact_deny_glob_blocks_top_level_file() {
        let root = workspace_root();
        let cfg = sample_config(vec![WorkspaceDomainConfig {
            id: "home_full".to_string(),
            root: root.to_string_lossy().to_string(),
            trust_label: "untrusted".to_string(),
            enabled: true,
            watch_enabled: false,
            watch_priority: 1,
            max_indexed_files: 1,
            allow_hidden_paths: false,
            allow_roots: vec!["**".to_string()],
            deny_globs: vec!["_viminfo".to_string()],
        }]);

        let guard = PolicyGuard::from_config(&cfg).unwrap();
        let decision = guard.should_index_path("home_full/_viminfo");

        assert!(!decision.allow);
        assert!(decision
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("deny list"));
    }
}


