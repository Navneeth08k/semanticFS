use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use semanticfs_common::{AuditEvent, GroundedHit};

#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub allow: bool,
    pub reason: Option<String>,
}

#[derive(Debug)]
pub struct PolicyGuard {
    allow_set: GlobSet,
    has_allow_roots: bool,
    deny_set: GlobSet,
    secret_patterns: Vec<Regex>,
}

impl PolicyGuard {
    pub fn new(allow_roots: &[String], deny_globs: &[String]) -> Result<Self> {
        let allow_set = build_glob_set(allow_roots)?;
        let deny_set = build_glob_set(deny_globs)?;

        let secret_patterns = vec![
            Regex::new(r#"(?i)api[_-]?key\s*[:=]\s*["']?[a-z0-9_\-]{12,}"#)?,
            Regex::new(r#"(?i)secret\s*[:=]\s*["']?[a-z0-9_\-]{12,}"#)?,
            Regex::new(r#"(?i)token\s*[:=]\s*["']?[a-z0-9_\-]{12,}"#)?,
            Regex::new(r"-----BEGIN (RSA|EC|OPENSSH) PRIVATE KEY-----")?,
            Regex::new(r"ghp_[A-Za-z0-9]{20,}")?,
        ];

        Ok(Self {
            allow_set,
            has_allow_roots: !allow_roots.is_empty(),
            deny_set,
            secret_patterns,
        })
    }

    pub fn should_index_path(&self, relative_path: &str) -> PolicyDecision {
        if self.deny_set.is_match(relative_path) {
            return PolicyDecision {
                allow: false,
                reason: Some("path matched deny list".to_string()),
            };
        }

        if !self.has_allow_roots || self.allow_set.is_match(relative_path) {
            return PolicyDecision {
                allow: true,
                reason: None,
            };
        }

        PolicyDecision {
            allow: false,
            reason: Some("path not included in allow roots".to_string()),
        }
    }

    pub fn contains_secret(&self, content: &str) -> bool {
        self.secret_patterns.iter().any(|p| p.is_match(content))
            || high_entropy_token_present(content)
    }

    pub fn redact_sensitive_hits(&self, hits: Vec<GroundedHit>) -> Vec<GroundedHit> {
        hits.into_iter()
            .filter(|hit| !self.deny_set.is_match(&hit.path))
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

fn build_glob_set(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
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
}
