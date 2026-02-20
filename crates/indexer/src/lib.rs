pub mod chunking;
pub mod db;
pub mod embedding;
pub mod filetype;
pub mod lancedb_sync;
pub mod map_enrichment;
pub mod map_summary;
pub mod symbols;

use anyhow::{Context, Result};
use notify::{
    Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use policy_guard::PolicyGuard;
use semanticfs_common::{IndexVersionState, IndexingStatus, SemanticFsConfig};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tracing::info;

use crate::{
    chunking::{chunk_content, ChunkRecord},
    db::IndexerDb,
    embedding::Embedder,
    filetype::FileType,
    lancedb_sync::sync_vectors_to_lancedb_if_enabled,
    map_enrichment::{run_enrichment_job, run_enrichment_job_blocking, EnrichmentMode},
    map_summary::DirectorySummary,
    symbols::extract_symbols,
};

pub struct Indexer {
    cfg: SemanticFsConfig,
    db: IndexerDb,
    db_path: std::path::PathBuf,
    guard: PolicyGuard,
    embedder: Embedder,
}

impl Indexer {
    pub fn new(cfg: SemanticFsConfig, db_path: &Path) -> Result<Self> {
        let db = IndexerDb::open(db_path)?;
        db.ensure_schema()?;

        let guard = PolicyGuard::new(&cfg.filter.allow_roots, &cfg.filter.deny_globs)?;
        let embedder = Embedder::from_config(&cfg.embedding)?;

        Ok(Self {
            cfg,
            db,
            db_path: db_path.to_path_buf(),
            guard,
            embedder,
        })
    }

    pub fn build_full_index(&self) -> Result<u64> {
        self.build_full_index_with_plan(None)
    }

    fn build_full_index_with_plan(&self, plan: Option<&ReindexPlan>) -> Result<u64> {
        let version = self.db.create_staging_version()?;
        let root = Path::new(&self.cfg.workspace.repo_root);
        let hotset: HashSet<String> = plan
            .map(|p| p.hot_paths.iter().cloned().collect())
            .unwrap_or_default();
        let mut pending_changed: HashSet<String> = plan
            .map(|p| p.metadata_paths.iter().cloned().collect())
            .unwrap_or_default();
        let started_unix_ms = unix_now_ms();
        let mut phase = if hotset.is_empty() {
            "rebuild".to_string()
        } else {
            "p1_hotset".to_string()
        };

        if let Some(plan) = plan {
            self.emit_indexing_status(plan, &phase, &pending_changed, started_unix_ms)?;
        }

        let mut all_files = Vec::new();
        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path().to_path_buf();
            let rel = normalize_rel_path(&path.strip_prefix(root)?.to_string_lossy());
            all_files.push((path, rel));
        }

        all_files.sort_by(|a, b| {
            let a_hot = hotset.contains(&a.1);
            let b_hot = hotset.contains(&b.1);
            b_hot.cmp(&a_hot).then_with(|| a.1.cmp(&b.1))
        });

        for (idx, (path, rel)) in all_files.into_iter().enumerate() {
            let decision = self.guard.should_index_path(&rel);
            if !decision.allow {
                self.db
                    .upsert_file_record(&rel, "", "skipped", "denied", version)?;
                continue;
            }

            let metadata = fs::metadata(&path)?;
            let size_mb = metadata.len() / (1024 * 1024);
            if size_mb > self.cfg.filter.max_file_mb {
                self.db
                    .upsert_file_record(&rel, "", "skipped", "too_large", version)?;
                continue;
            }

            self.index_file(&path, &rel, version)?;

            if pending_changed.remove(&rel) {
                if phase == "p1_hotset" && !pending_changed.iter().any(|p| hotset.contains(p)) {
                    phase = "p3_backfill".to_string();
                    if let Some(plan) = plan {
                        self.emit_indexing_status(plan, &phase, &pending_changed, started_unix_ms)?;
                    }
                }
            }
            if plan.is_some() && idx % 100 == 0 {
                if let Some(plan) = plan {
                    self.emit_indexing_status(plan, &phase, &pending_changed, started_unix_ms)?;
                }
            }
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

        self.event_loop(&rx, &root, debounce)
    }

    fn event_loop(
        &self,
        rx: &Receiver<notify::Result<Event>>,
        root: &Path,
        debounce: Duration,
    ) -> Result<()> {
        loop {
            let mut changed_counts: HashMap<String, u32> = HashMap::new();
            let first = rx.recv()?;
            self.capture_event_changes(first, root, &mut changed_counts);

            loop {
                match rx.recv_timeout(debounce) {
                    Ok(event) => self.capture_event_changes(event, root, &mut changed_counts),
                    Err(RecvTimeoutError::Timeout) => break,
                    Err(RecvTimeoutError::Disconnected) => return Ok(()),
                }
            }

            if changed_counts.is_empty() {
                continue;
            }

            let plan = build_reindex_plan(
                &changed_counts,
                self.cfg.index.bulk_event_threshold.max(1),
                self.cfg.index.hotset_max_paths.max(1),
            );
            let pending_paths =
                plan.report_pending(self.cfg.index.pending_path_report_limit.max(1));

            self.db.set_indexing_status(&IndexingStatus {
                in_progress: true,
                phase: if plan.bulk_event {
                    "queued_bulk".to_string()
                } else {
                    "queued".to_string()
                },
                started_unix_ms: unix_now_ms(),
                updated_unix_ms: unix_now_ms(),
                total_changed_paths: plan.metadata_paths.len(),
                hotset_total: plan.hot_paths.len(),
                deferred_total: plan.deferred_paths.len(),
                pending_paths,
                message: format!(
                    "queue prepared: p1={} p2={} p3={}",
                    plan.hot_paths.len(),
                    plan.metadata_paths.len(),
                    plan.deferred_paths.len()
                ),
            })?;

            let build_result = self.build_full_index_with_plan(Some(&plan));
            match build_result {
                Ok(version) => {
                    self.spawn_map_enrichment(version);
                    self.db.clear_indexing_status()?;
                    info!(
                        version,
                        bulk = plan.bulk_event,
                        p1 = plan.hot_paths.len(),
                        p2 = plan.metadata_paths.len(),
                        p3 = plan.deferred_paths.len(),
                        "watch-triggered reindex published"
                    );
                }
                Err(err) => {
                    let _ = self.db.clear_indexing_status();
                    return Err(err);
                }
            }
        }
    }

    fn capture_event_changes(
        &self,
        event: notify::Result<Event>,
        root: &Path,
        changed_counts: &mut HashMap<String, u32>,
    ) {
        match event {
            Ok(ev) if should_trigger_reindex(&ev) => {
                let paths = event_relative_paths(&ev, root);
                if paths.is_empty() {
                    return;
                }
                for rel in paths {
                    if !self.guard.should_index_path(&rel).allow {
                        continue;
                    }
                    *changed_counts.entry(rel).or_insert(0) += 1;
                }
            }
            Ok(_) => {}
            Err(err) => {
                tracing::warn!(error = %err, "watcher event error");
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

                let batch_texts = chunks.iter().map(|c| c.content.clone()).collect::<Vec<_>>();
                let embeddings = self.embedder.embed_batch(&batch_texts)?;

                for (chunk, embedding) in chunks.into_iter().zip(embeddings.into_iter()) {
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

    fn spawn_map_enrichment(&self, version: u64) {
        if EnrichmentMode::from_config(&self.cfg.map.llm_enrichment) == EnrichmentMode::Disabled {
            return;
        }

        let db_path = self.db_path.clone();
        if let Err(err) = std::thread::Builder::new()
            .name(format!("map-enrichment-v{}", version))
            .spawn(move || run_enrichment_job(db_path, version))
        {
            tracing::warn!(version, error=%err, "failed to spawn map enrichment worker");
        }
    }

    pub fn enrich_map_for_version(&self, version: u64) -> Result<()> {
        if EnrichmentMode::from_config(&self.cfg.map.llm_enrichment) == EnrichmentMode::Disabled {
            return Ok(());
        }
        run_enrichment_job_blocking(&self.db_path, version)
    }

    fn emit_indexing_status(
        &self,
        plan: &ReindexPlan,
        phase: &str,
        pending_changed: &HashSet<String>,
        started_unix_ms: u64,
    ) -> Result<()> {
        let mut pending = pending_changed.iter().cloned().collect::<Vec<_>>();
        pending.sort();
        pending.truncate(self.cfg.index.pending_path_report_limit.max(1));

        self.db.set_indexing_status(&IndexingStatus {
            in_progress: true,
            phase: phase.to_string(),
            started_unix_ms,
            updated_unix_ms: unix_now_ms(),
            total_changed_paths: plan.metadata_paths.len(),
            hotset_total: plan.hot_paths.len(),
            deferred_total: plan.deferred_paths.len(),
            pending_paths: pending,
            message: format!(
                "indexing in progress: p1={} p2={} p3={}",
                plan.hot_paths.len(),
                plan.metadata_paths.len(),
                plan.deferred_paths.len()
            ),
        })?;
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

fn event_relative_paths(event: &Event, root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for p in &event.paths {
        let rel = if let Ok(r) = p.strip_prefix(root) {
            Some(normalize_rel_path(&r.to_string_lossy()))
        } else if !p.is_absolute() {
            Some(normalize_rel_path(&p.to_string_lossy()))
        } else {
            None
        };

        if let Some(path) = rel {
            if !path.is_empty() {
                out.push(path);
            }
        }
    }
    out
}

fn normalize_rel_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn unix_now_ms() -> u64 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
struct ReindexPlan {
    hot_paths: Vec<String>,
    metadata_paths: Vec<String>,
    deferred_paths: Vec<String>,
    bulk_event: bool,
}

impl ReindexPlan {
    fn report_pending(&self, limit: usize) -> Vec<String> {
        let mut paths = self.metadata_paths.clone();
        paths.sort();
        paths.truncate(limit.max(1));
        paths
    }
}

fn build_reindex_plan(
    changed_counts: &HashMap<String, u32>,
    bulk_threshold: usize,
    hotset_max_paths: usize,
) -> ReindexPlan {
    let mut ranked = changed_counts
        .iter()
        .map(|(path, count)| (path.clone(), *count))
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let metadata_paths = ranked
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    let bulk_event = metadata_paths.len() >= bulk_threshold.max(1);
    let hot_take = if bulk_event {
        hotset_max_paths.max(1).min(metadata_paths.len())
    } else {
        metadata_paths.len()
    };
    let hot_paths = metadata_paths
        .iter()
        .take(hot_take)
        .cloned()
        .collect::<Vec<_>>();
    let deferred_paths = metadata_paths
        .iter()
        .skip(hot_take)
        .cloned()
        .collect::<Vec<_>>();

    ReindexPlan {
        hot_paths,
        metadata_paths,
        deferred_paths,
        bulk_event,
    }
}

#[cfg(test)]
mod tests {
    use super::build_reindex_plan;
    use std::collections::HashMap;

    #[test]
    fn reindex_plan_marks_bulk_and_splits_hotset() {
        let mut counts = HashMap::new();
        for i in 0..100 {
            counts.insert(format!("src/f{}.rs", i), 1);
        }

        let plan = build_reindex_plan(&counts, 80, 16);
        assert!(plan.bulk_event);
        assert_eq!(plan.hot_paths.len(), 16);
        assert_eq!(plan.metadata_paths.len(), 100);
        assert_eq!(plan.deferred_paths.len(), 84);
    }

    #[test]
    fn reindex_plan_non_bulk_keeps_all_in_hotset() {
        let mut counts = HashMap::new();
        counts.insert("src/a.rs".to_string(), 3);
        counts.insert("src/b.rs".to_string(), 1);

        let plan = build_reindex_plan(&counts, 80, 16);
        assert!(!plan.bulk_event);
        assert_eq!(plan.hot_paths.len(), 2);
        assert_eq!(plan.deferred_paths.len(), 0);
    }
}
