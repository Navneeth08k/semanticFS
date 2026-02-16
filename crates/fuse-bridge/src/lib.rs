use anyhow::{Context, Result};
use lru::LruCache;
use map_engine::MapEngine;
use parking_lot::Mutex;
use policy_guard::PolicyGuard;
use retrieval_core::RetrievalCore;
use semanticfs_common::{FuseCacheConfig, GroundedHit, RetrievalConfig, SemanticFsConfig};
use std::{fs, num::NonZeroUsize, path::PathBuf, sync::Arc};

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
}

impl FuseBridge {
    pub fn new(cfg: SemanticFsConfig, sqlite_path: &std::path::Path) -> Result<Self> {
        let guard = PolicyGuard::new(&cfg.filter.allow_roots, &cfg.filter.deny_globs)?;
        let retrieval = RetrievalCore::open(
            sqlite_path,
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
        })
    }

    pub fn read_virtual(
        &self,
        virtual_path: &str,
        snapshot_version: u64,
        active_version: u64,
    ) -> Result<Vec<u8>> {
        if let Some(cached) = self.content_cache.lock().get(virtual_path).cloned() {
            return Ok(cached);
        }

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

        self.content_cache
            .lock()
            .put(virtual_path.to_string(), payload.clone());

        Ok(payload)
    }

    fn ensure_inode(&self, key: &str) {
        let mut cache = self.inode_cache.lock();
        if cache.get(key).is_none() {
            let inode = hash_inode(key);
            cache.put(key.to_string(), inode);
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
        let md = render_search_markdown(&query, &hits);
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

fn render_search_markdown(query: &str, hits: &[GroundedHit]) -> String {
    let mut out = format!("# Search Results\n\nQuery: `{}`\n\n", query);
    if hits.is_empty() {
        out.push_str("No results found.\n");
        return out;
    }

    for hit in hits {
        out.push_str(&format!(
            "## {}. `{}`:{}-{}\n- source: {:?}\n- hash: `{}`\n- snapshot: {} (active: {})\n- stale: {}\n- why: {}\n\n",
            hit.rank,
            hit.path,
            hit.start_line,
            hit.end_line,
            hit.source,
            &hit.file_hash.chars().take(12).collect::<String>(),
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

fn nz(v: usize) -> NonZeroUsize {
    NonZeroUsize::new(v.max(1)).unwrap_or(NonZeroUsize::MIN)
}
