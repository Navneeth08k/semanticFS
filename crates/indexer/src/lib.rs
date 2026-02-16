pub mod chunking;
pub mod db;
pub mod embedding;
pub mod filetype;
pub mod lancedb_sync;
pub mod map_summary;
pub mod symbols;

use anyhow::{Context, Result};
use notify::{
    Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use policy_guard::PolicyGuard;
use semanticfs_common::{IndexVersionState, SemanticFsConfig};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};
use tracing::info;

use crate::{
    chunking::{chunk_content, ChunkRecord},
    db::IndexerDb,
    embedding::Embedder,
    filetype::FileType,
    lancedb_sync::sync_vectors_to_lancedb_if_enabled,
    map_summary::DirectorySummary,
    symbols::extract_symbols,
};

pub struct Indexer {
    cfg: SemanticFsConfig,
    db: IndexerDb,
    guard: PolicyGuard,
    embedder: Embedder,
}

impl Indexer {
    pub fn new(cfg: SemanticFsConfig, db_path: &Path) -> Result<Self> {
        let db = IndexerDb::open(db_path)?;
        db.ensure_schema()?;

        let guard = PolicyGuard::new(&cfg.filter.allow_roots, &cfg.filter.deny_globs)?;
        let embedder = Embedder::from_config(&cfg.embedding);

        Ok(Self {
            cfg,
            db,
            guard,
            embedder,
        })
    }

    pub fn build_full_index(&self) -> Result<u64> {
        let version = self.db.create_staging_version()?;
        let root = Path::new(&self.cfg.workspace.repo_root);

        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            let rel = path.strip_prefix(root)?.to_string_lossy().to_string();
            let decision = self.guard.should_index_path(&rel);
            if !decision.allow {
                self.db
                    .upsert_file_record(&rel, "", "skipped", "denied", version)?;
                continue;
            }

            let metadata = fs::metadata(path)?;
            let size_mb = metadata.len() / (1024 * 1024);
            if size_mb > self.cfg.filter.max_file_mb {
                self.db
                    .upsert_file_record(&rel, "", "skipped", "too_large", version)?;
                continue;
            }

            self.index_file(path, &rel, version)?;
        }

        self.precompute_directory_summaries(version)?;
        sync_vectors_to_lancedb_if_enabled(&self.db, version)?;
        self.db.publish_staging_version(version)?;
        info!(version, "completed full index");
        Ok(version)
    }

    pub fn active_version(&self) -> Result<u64> {
        self.db.active_version()
    }

    pub fn watch(&self) -> Result<()> {
        let root = Path::new(&self.cfg.workspace.repo_root).to_path_buf();
        let debounce = Duration::from_millis(self.cfg.index.debounce_ms.max(50));
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

        let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            NotifyConfig::default(),
        )?;

        watcher.watch(&root, RecursiveMode::Recursive)?;
        info!(path = %root.display(), "watching filesystem for incremental rebuild triggers");

        let mut last_build = Instant::now() - debounce;
        self.event_loop(&rx, debounce, &mut last_build)
    }

    fn event_loop(
        &self,
        rx: &Receiver<notify::Result<Event>>,
        debounce: Duration,
        last_build: &mut Instant,
    ) -> Result<()> {
        loop {
            let event = rx.recv()?;
            match event {
                Ok(ev) if should_trigger_reindex(&ev) => {
                    if last_build.elapsed() < debounce {
                        continue;
                    }
                    let version = self.build_full_index()?;
                    *last_build = Instant::now();
                    info!(version, "watch-triggered reindex published");
                }
                Ok(_) => {}
                Err(err) => {
                    tracing::warn!(error = %err, "watcher event error");
                }
            }
        }
    }

    fn index_file(&self, absolute: &Path, relative: &str, version: u64) -> Result<()> {
        let bytes = fs::read(absolute).with_context(|| format!("read file {}", relative))?;
        let hash = hash_bytes(&bytes);

        let file_type = FileType::from_path(relative);

        match file_type {
            FileType::Binary => {
                self.db
                    .upsert_file_record(relative, &hash, "binary", "metadata_only", version)?;
            }
            _ => {
                let content = String::from_utf8_lossy(&bytes).to_string();
                let chunks = chunk_content(&content, &file_type, self.cfg.index.chunk_max_lines);
                let symbols = extract_symbols(&content, &file_type, relative);

                self.db.upsert_file_record(
                    relative,
                    &hash,
                    file_type.as_str(),
                    "indexed",
                    version,
                )?;

                self.db.delete_chunks_for_path(relative, version)?;
                self.db.delete_symbols_for_path(relative, version)?;
                self.db.delete_vectors_for_path(relative, version)?;

                for chunk in chunks {
                    let embedding = self.embedder.embed(&chunk.content);
                    self.db.upsert_chunk(&chunk, relative, &hash, version)?;
                    self.db
                        .upsert_vector(&chunk.chunk_id, relative, &embedding, version)?;
                }

                for sym in symbols {
                    self.db.upsert_symbol(&sym, &hash, version)?;
                }
            }
        }

        Ok(())
    }

    fn precompute_directory_summaries(&self, version: u64) -> Result<()> {
        let summaries: Vec<DirectorySummary> = self.db.compute_directory_summaries(version)?;
        for summary in summaries {
            self.db.upsert_map_summary(&summary, version)?;
        }
        Ok(())
    }
}

fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn normalize_index_state(state: IndexVersionState) -> &'static str {
    match state {
        IndexVersionState::Staging => "staging",
        IndexVersionState::Active => "active",
        IndexVersionState::Obsolete => "obsolete",
    }
}

#[allow(dead_code)]
pub fn _chunk_record_type(_: ChunkRecord) {}

fn should_trigger_reindex(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_)
            | EventKind::Modify(_)
            | EventKind::Remove(_)
            | EventKind::Any
            | EventKind::Other
    )
}
