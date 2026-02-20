use anyhow::{Context, Result};
use lru::LruCache;
use map_engine::MapEngine;
use parking_lot::Mutex;
use policy_guard::PolicyGuard;
use retrieval_core::RetrievalCore;
use semanticfs_common::{
    FuseCacheConfig, GroundedHit, RetrievalConfig, SemanticFsConfig, TrustLevel,
};
use std::{
    fs,
    num::NonZeroUsize,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};

#[cfg(target_os = "linux")]
mod linux_mount;
#[cfg(not(target_os = "linux"))]
mod linux_mount {
    use super::FuseBridge;
    use anyhow::{bail, Result};

    pub fn serve_mount(_bridge: FuseBridge) -> Result<()> {
        bail!("Linux FUSE mounting is only supported on Linux targets")
    }
}

pub struct FuseBridge {
    cfg: SemanticFsConfig,
    retrieval: RetrievalCore,
    map_engine: MapEngine,
    inode_cache: Arc<Mutex<LruCache<String, u64>>>,
    content_cache: Arc<Mutex<LruCache<String, Vec<u8>>>>,
    stats: Arc<BridgeStats>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuseSessionMode {
    PerRequest,
    Pinned,
}

#[derive(Debug, Clone)]
pub struct BridgeStatsSnapshot {
    pub read_total: u64,
    pub read_errors: u64,
    pub latency_count: u64,
    pub latency_sum_ms: u64,
    pub latency_buckets: Vec<(u64, u64)>,
    pub inode_cache_hits: u64,
    pub inode_cache_misses: u64,
    pub content_cache_hits: u64,
    pub content_cache_misses: u64,
    pub stale_hits_total: u64,
    pub policy_denies_total: u64,
}

struct BridgeStats {
    read_total: AtomicU64,
    read_errors: AtomicU64,
    latency_count: AtomicU64,
    latency_sum_ms: AtomicU64,
    latency_buckets: Mutex<Vec<u64>>,
    inode_cache_hits: AtomicU64,
    inode_cache_misses: AtomicU64,
    content_cache_hits: AtomicU64,
    content_cache_misses: AtomicU64,
    stale_hits_total: AtomicU64,
    policy_denies_total: AtomicU64,
}

const LATENCY_BUCKETS_MS: [u64; 10] = [1, 5, 10, 25, 50, 100, 250, 500, 1000, 2000];

impl BridgeStats {
    fn new() -> Self {
        Self {
            read_total: AtomicU64::new(0),
            read_errors: AtomicU64::new(0),
            latency_count: AtomicU64::new(0),
            latency_sum_ms: AtomicU64::new(0),
            latency_buckets: Mutex::new(vec![0; LATENCY_BUCKETS_MS.len()]),
            inode_cache_hits: AtomicU64::new(0),
            inode_cache_misses: AtomicU64::new(0),
            content_cache_hits: AtomicU64::new(0),
            content_cache_misses: AtomicU64::new(0),
            stale_hits_total: AtomicU64::new(0),
            policy_denies_total: AtomicU64::new(0),
        }
    }

    fn observe_read_latency(&self, elapsed_ms: u64) {
        self.latency_count.fetch_add(1, Ordering::Relaxed);
        self.latency_sum_ms.fetch_add(elapsed_ms, Ordering::Relaxed);

        let bucket_index = LATENCY_BUCKETS_MS
            .iter()
            .position(|bound| elapsed_ms <= *bound)
            .unwrap_or(LATENCY_BUCKETS_MS.len() - 1);
        let mut buckets = self.latency_buckets.lock();
        if let Some(v) = buckets.get_mut(bucket_index) {
            *v += 1;
        }
    }

    fn snapshot(&self) -> BridgeStatsSnapshot {
        let buckets = self.latency_buckets.lock().clone();
        let latency_buckets = LATENCY_BUCKETS_MS
            .iter()
            .copied()
            .zip(buckets)
            .collect::<Vec<_>>();

        BridgeStatsSnapshot {
            read_total: self.read_total.load(Ordering::Relaxed),
            read_errors: self.read_errors.load(Ordering::Relaxed),
            latency_count: self.latency_count.load(Ordering::Relaxed),
            latency_sum_ms: self.latency_sum_ms.load(Ordering::Relaxed),
            latency_buckets,
            inode_cache_hits: self.inode_cache_hits.load(Ordering::Relaxed),
            inode_cache_misses: self.inode_cache_misses.load(Ordering::Relaxed),
            content_cache_hits: self.content_cache_hits.load(Ordering::Relaxed),
            content_cache_misses: self.content_cache_misses.load(Ordering::Relaxed),
            stale_hits_total: self.stale_hits_total.load(Ordering::Relaxed),
            policy_denies_total: self.policy_denies_total.load(Ordering::Relaxed),
        }
    }
}

impl FuseBridge {
    pub fn new(cfg: SemanticFsConfig, sqlite_path: &std::path::Path) -> Result<Self> {
        let guard = PolicyGuard::new(&cfg.filter.allow_roots, &cfg.filter.deny_globs)?;
        let retrieval = RetrievalCore::open(
            sqlite_path,
            PathBuf::from(&cfg.workspace.repo_root).as_path(),
            cfg.retrieval.clone(),
            cfg.embedding.dimension,
            guard,
        )?;
        let map_engine = MapEngine::open(sqlite_path)?;

        let inode_cache = LruCache::new(nz(cfg.fuse_cache.max_virtual_inodes));
        let bytes_per_entry = 64 * 1024;
        let max_entries = ((cfg.fuse_cache.max_cached_mb * 1024 * 1024) / bytes_per_entry).max(1);
        let content_cache = LruCache::new(nz(max_entries));

        Ok(Self {
            cfg,
            retrieval,
            map_engine,
            inode_cache: Arc::new(Mutex::new(inode_cache)),
            content_cache: Arc::new(Mutex::new(content_cache)),
            stats: Arc::new(BridgeStats::new()),
        })
    }

    pub fn read_virtual(
        &self,
        virtual_path: &str,
        snapshot_version: u64,
        active_version: u64,
    ) -> Result<Vec<u8>> {
        let started = Instant::now();
        let cache_key = content_cache_key(virtual_path, snapshot_version, active_version);

        let result = (|| {
            if let Some(cached) = self.content_cache.lock().get(&cache_key).cloned() {
                self.stats
                    .content_cache_hits
                    .fetch_add(1, Ordering::Relaxed);
                return Ok(cached);
            }
            self.stats
                .content_cache_misses
                .fetch_add(1, Ordering::Relaxed);

            self.ensure_inode(virtual_path);

            let payload = if let Some(raw_path) = virtual_path.strip_prefix("/raw/") {
                self.read_raw(raw_path)?
            } else if let Some(query_file) = virtual_path.strip_prefix("/search/") {
                self.render_search(query_file, snapshot_version, active_version)?
            } else if let Some(map_path) = virtual_path.strip_prefix("/map/") {
                self.render_map(map_path, snapshot_version)?
            } else if virtual_path == "/.well-known/health.json" {
                b"{\"live\":true,\"ready\":true}".to_vec()
            } else {
                anyhow::bail!("unsupported virtual path")
            };

            self.content_cache.lock().put(cache_key, payload.clone());

            Ok(payload)
        })();

        self.stats.read_total.fetch_add(1, Ordering::Relaxed);
        let elapsed_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        self.stats.observe_read_latency(elapsed_ms);

        if let Err(err) = &result {
            self.stats.read_errors.fetch_add(1, Ordering::Relaxed);
            let msg = err.to_string().to_ascii_lowercase();
            if msg.contains("deny") || msg.contains("secret") || msg.contains("redact") {
                self.stats
                    .policy_denies_total
                    .fetch_add(1, Ordering::Relaxed);
            }
        }

        result
    }

    fn ensure_inode(&self, key: &str) {
        let mut cache = self.inode_cache.lock();
        if cache.get(key).is_none() {
            self.stats
                .inode_cache_misses
                .fetch_add(1, Ordering::Relaxed);
            let inode = hash_inode(key);
            cache.put(key.to_string(), inode);
        } else {
            self.stats.inode_cache_hits.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn read_raw(&self, raw_path: &str) -> Result<Vec<u8>> {
        let mut path = PathBuf::from(&self.cfg.workspace.repo_root);
        path.push(raw_path);

        let canonical = path
            .canonicalize()
            .with_context(|| format!("canonicalize path: {}", path.display()))?;
        let repo_root = PathBuf::from(&self.cfg.workspace.repo_root).canonicalize()?;

        if !canonical.starts_with(&repo_root) {
            anyhow::bail!("path escape detected");
        }

        Ok(fs::read(canonical)?)
    }

    fn render_search(
        &self,
        query_file: &str,
        snapshot_version: u64,
        active_version: u64,
    ) -> Result<Vec<u8>> {
        let query = normalize_query(query_file);
        let hits = self
            .retrieval
            .search(&query, snapshot_version, active_version)?;
        let stale_count = hits.iter().filter(|h| h.stale).count() as u64;
        self.stats
            .stale_hits_total
            .fetch_add(stale_count, Ordering::Relaxed);
        let status = self.retrieval.indexing_status().ok().flatten();
        let md = render_search_markdown(&query, &hits, status.as_ref());
        Ok(md.into_bytes())
    }

    fn render_map(&self, map_path: &str, snapshot_version: u64) -> Result<Vec<u8>> {
        let dir = map_path.trim_end_matches("/directory_overview.md");
        let body = self
            .map_engine
            .get_directory_overview(dir, snapshot_version)?
            .unwrap_or_else(|| "# Directory Overview\n\nNo summary yet.".to_string());
        Ok(body.into_bytes())
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let inode = self.inode_cache.lock().len();
        let content = self.content_cache.lock().len();
        (inode, content)
    }

    pub fn stats_snapshot(&self) -> BridgeStatsSnapshot {
        self.stats.snapshot()
    }

    pub fn active_version(&self) -> Result<u64> {
        self.retrieval.active_version()
    }

    pub fn read_virtual_current(&self, virtual_path: &str) -> Result<Vec<u8>> {
        let active = self.active_version()?;
        self.read_virtual(virtual_path, active, active)
    }

    pub fn mount(self) -> Result<()> {
        linux_mount::serve_mount(self)
    }

    pub fn retrieval_config(&self) -> &RetrievalConfig {
        &self.cfg.retrieval
    }

    pub fn cache_config(&self) -> &FuseCacheConfig {
        &self.cfg.fuse_cache
    }

    pub fn fuse_session_mode(&self) -> FuseSessionMode {
        parse_fuse_session_mode(&self.cfg.fuse_session.mode)
    }

    pub fn fuse_session_max_entries(&self) -> usize {
        self.cfg.fuse_session.max_entries.max(1)
    }

    pub fn mount_point(&self) -> &str {
        &self.cfg.workspace.mount_point
    }

    pub fn repo_root(&self) -> &str {
        &self.cfg.workspace.repo_root
    }
}

fn normalize_query(query_file: &str) -> String {
    query_file
        .trim_end_matches(".md")
        .replace('_', " ")
        .replace("%20", " ")
}

fn render_search_markdown(
    query: &str,
    hits: &[GroundedHit],
    status: Option<&semanticfs_common::IndexingStatus>,
) -> String {
    let mut out = format!("# Search Results\n\nQuery: `{}`\n\n", query);
    if let Some(s) = status {
        if s.in_progress {
            let pending = if s.pending_paths.is_empty() {
                "none".to_string()
            } else {
                s.pending_paths.join(", ")
            };
            out.push_str(&format!(
                "> [INDEXING IN PROGRESS] `{}`. Changed paths pending: {}. Results may be incomplete; re-run query on latest snapshot.\n\n",
                s.phase, pending
            ));
        }
    }

    if hits.is_empty() {
        out.push_str("No results found.\n");
        return out;
    }

    for hit in hits {
        let symbol = hit.symbol_kind.as_deref().unwrap_or("n/a");
        let trust = trust_label(hit.trust_level);
        let short_hash = &hit.file_hash.chars().take(12).collect::<String>();
        out.push_str(&format!(
            "## {}. `{}`:{}-{}\n`// Source: {} | Symbol: {} | Hash: {} | Snapshot: {} (active: {}) | Trust: {}`\n- source: {:?}\n- hash: `{}`\n- snapshot: {} (active: {})\n- stale: {}\n- why: {}\n\n",
            hit.rank,
            hit.path,
            hit.start_line,
            hit.end_line,
            hit.path,
            symbol,
            short_hash,
            hit.snapshot_version,
            hit.active_version,
            trust,
            hit.source,
            short_hash,
            hit.snapshot_version,
            hit.active_version,
            hit.stale,
            hit.why_selected
        ));
    }

    out
}

fn hash_inode(key: &str) -> u64 {
    let mut hash: u64 = 1469598103934665603;
    for b in key.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

fn content_cache_key(virtual_path: &str, snapshot_version: u64, active_version: u64) -> String {
    format!(
        "{}|snapshot={}|active={}",
        virtual_path, snapshot_version, active_version
    )
}

fn parse_fuse_session_mode(raw: &str) -> FuseSessionMode {
    if raw.eq_ignore_ascii_case("per_request") {
        FuseSessionMode::PerRequest
    } else {
        FuseSessionMode::Pinned
    }
}

fn trust_label(level: TrustLevel) -> &'static str {
    match level {
        TrustLevel::Trusted => "trusted",
        TrustLevel::Untrusted => "untrusted",
    }
}

fn nz(v: usize) -> NonZeroUsize {
    NonZeroUsize::new(v.max(1)).unwrap_or(NonZeroUsize::MIN)
}

#[cfg(test)]
mod tests {
    use super::{
        content_cache_key, parse_fuse_session_mode, render_search_markdown, FuseSessionMode,
    };
    use semanticfs_common::{GroundedHit, HitSource, IndexingStatus, TrustLevel};

    #[test]
    fn search_markdown_contains_breadcrumb_contract() {
        let hits = vec![GroundedHit {
            rank: 1,
            path: "src/lib/auth.rs".to_string(),
            start_line: 10,
            end_line: 24,
            file_hash: "abcdef1234567890".to_string(),
            snapshot_version: 7,
            active_version: 7,
            score_rrf: 0.1,
            score_symbol: Some(2.0),
            score_bm25: None,
            score_vector: None,
            source: HitSource::Symbol,
            symbol_kind: Some("function".to_string()),
            stale: false,
            trust_level: TrustLevel::Trusted,
            why_selected: "exact symbol match".to_string(),
        }];

        let md = render_search_markdown("auth handler", &hits, None);
        assert!(md.contains("`// Source: src/lib/auth.rs"));
        assert!(md.contains("| Symbol: function |"));
        assert!(md.contains("| Snapshot: 7 (active: 7) |"));
        assert!(md.contains("| Trust: trusted`"));
    }

    #[test]
    fn search_markdown_surfaces_indexing_in_progress() {
        let status = IndexingStatus {
            in_progress: true,
            phase: "p3_backfill".to_string(),
            started_unix_ms: 1,
            updated_unix_ms: 2,
            total_changed_paths: 2,
            hotset_total: 1,
            deferred_total: 1,
            pending_paths: vec!["src/new_auth.rs".to_string()],
            message: "indexing".to_string(),
        };
        let md = render_search_markdown("auth", &[], Some(&status));
        assert!(md.contains("INDEXING IN PROGRESS"));
        assert!(md.contains("src/new_auth.rs"));
    }

    #[test]
    fn content_cache_key_includes_snapshot_context() {
        let a = content_cache_key("/search/auth.md", 10, 12);
        let b = content_cache_key("/search/auth.md", 11, 12);
        assert_ne!(a, b);
    }

    #[test]
    fn fuse_session_mode_defaults_to_pinned_for_unknown_values() {
        assert_eq!(
            parse_fuse_session_mode("per_request"),
            FuseSessionMode::PerRequest
        );
        assert_eq!(parse_fuse_session_mode("pinned"), FuseSessionMode::Pinned);
        assert_eq!(parse_fuse_session_mode("invalid"), FuseSessionMode::Pinned);
    }
}
