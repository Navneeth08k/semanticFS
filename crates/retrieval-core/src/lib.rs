use anyhow::Result;
use policy_guard::PolicyGuard;
use rusqlite::{params, params_from_iter, types::Value, Connection};
use semanticfs_common::{
    cosine_similarity, embed_text_hash, GroundedHit, HitSource, IndexingStatus, RetrievalConfig,
    TrustLevel,
};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering as AtomicOrdering},
    Mutex,
};
use std::time::{Duration, SystemTime};

#[cfg(feature = "lancedb")]
use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct PartialHit {
    path: String,
    domain_id: String,
    start_line: u32,
    end_line: u32,
    file_hash: String,
    modified_unix_ms: Option<i64>,
    trust_label: String,
    trust_level: TrustLevel,
    source: HitSource,
    symbol_kind: Option<String>,
    score_symbol: Option<f32>,
    score_bm25: Option<f32>,
    score_vector: Option<f32>,
    why_selected: String,
}

#[derive(Debug, Clone)]
struct QueryScoreContext {
    overlap_terms: Vec<String>,
    is_symbol_like: bool,
    is_config_like: bool,
    is_narrative_docs: bool,
    is_docs_schema_like: bool,
    is_command_like: bool,
    is_workflow_like: bool,
    is_systemd_unit_like: bool,
}

impl QueryScoreContext {
    fn new(query: &str) -> Self {
        let is_narrative_docs = is_narrative_docs_query(query);
        Self {
            overlap_terms: ranking_overlap_terms(query),
            is_symbol_like: is_symbol_like_query(query),
            is_config_like: is_config_like_query(query),
            is_narrative_docs,
            is_docs_schema_like: is_docs_schema_like_query(query, is_narrative_docs),
            is_command_like: is_command_like_query(query),
            is_workflow_like: is_workflow_like_query(query),
            is_systemd_unit_like: is_systemd_unit_like_query(query),
        }
    }

    fn is_structured_literal(&self) -> bool {
        self.is_config_like
            || self.is_command_like
            || self.is_workflow_like
            || self.is_systemd_unit_like
    }

    fn should_run_vector_search(&self, has_symbol_hits: bool) -> bool {
        if self.is_structured_literal() {
            return false;
        }

        if self.is_symbol_like && has_symbol_hits {
            return false;
        }

        true
    }
}

#[derive(Debug, Clone)]
struct PathScoreContext {
    lower: String,
    path_terms: Vec<String>,
    file_name_terms: Vec<String>,
    path_prior: f32,
    recency_prior: f32,
}

#[derive(Debug, Clone)]
struct CachedPathScoreContext {
    ctx: PathScoreContext,
    cached_at: SystemTime,
}

#[derive(Debug, Clone, Copy)]
struct PriorBreakdown {
    total: f32,
    path: f32,
    recency: f32,
}

pub struct RetrievalCore {
    repo_root: PathBuf,
    cfg: RetrievalConfig,
    embed_dim: usize,
    guard: PolicyGuard,
    conn: Mutex<Connection>,
    path_cache: Mutex<HashMap<String, CachedPathScoreContext>>,
    search_cache: Mutex<HashMap<String, Vec<GroundedHit>>>,
    path_cache_version: AtomicU64,
}

const PATH_SCORE_CACHE_TTL: Duration = Duration::from_secs(120);
const SEARCH_RESULT_CACHE_MAX_ENTRIES: usize = 256;

impl RetrievalCore {
    pub fn open(
        db_path: &Path,
        repo_root: &Path,
        cfg: RetrievalConfig,
        embed_dim: usize,
        guard: PolicyGuard,
    ) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.set_prepared_statement_cache_capacity(32);
        Ok(Self {
            repo_root: repo_root.to_path_buf(),
            cfg,
            embed_dim: embed_dim.max(1),
            guard,
            conn: Mutex::new(conn),
            path_cache: Mutex::new(HashMap::new()),
            search_cache: Mutex::new(HashMap::new()),
            path_cache_version: AtomicU64::new(0),
        })
    }

    pub fn search(
        &self,
        query: &str,
        snapshot_version: u64,
        active_version: u64,
    ) -> Result<Vec<GroundedHit>> {
        self.refresh_caches_for_snapshot(snapshot_version);
        let cache_key = search_cache_key(query, snapshot_version, active_version);
        if let Some(cached) = self.cached_search_result(&cache_key) {
            return Ok(cached);
        }
        let query_ctx = QueryScoreContext::new(query);
        let mut rank_lists: Vec<Vec<PartialHit>> = Vec::new();

        let exact = self.symbol_exact(query, snapshot_version)?;
        if query_ctx.is_symbol_like && !exact.is_empty() {
            let rendered = self.guard.redact_sensitive_hits(self.render_direct_hits(
                &query_ctx,
                exact,
                snapshot_version,
                active_version,
            ));
            self.store_search_result(cache_key, &rendered);
            return Ok(rendered);
        }
        if !exact.is_empty() {
            rank_lists.push(exact.clone());
        }

        let skip_prefix_for_exact_symbol = query_ctx.is_symbol_like && !exact.is_empty();
        let prefix = if skip_prefix_for_exact_symbol {
            Vec::new()
        } else {
            self.symbol_prefix(query, snapshot_version)?
        };
        if !prefix.is_empty() {
            rank_lists.push(prefix.clone());
        }

        let skip_bm25_for_exact_symbol = query_ctx.is_symbol_like && !exact.is_empty();
        let bm25_limit = self.bm25_limit_for_query(&query_ctx);
        let bm25 = if skip_bm25_for_exact_symbol {
            Vec::new()
        } else {
            self.bm25(query, &query_ctx, snapshot_version, bm25_limit)?
        };
        let has_bm25_hits = !bm25.is_empty();
        let has_bm25_docs_hint = bm25
            .iter()
            .take(3)
            .any(|hit| is_docs_path(&hit.path.to_ascii_lowercase()));
        if !bm25.is_empty() {
            rank_lists.push(bm25);
        }

        let has_symbol_hits = !exact.is_empty() || !prefix.is_empty();
        if let Some(vector_limit) = self.vector_limit_for_query(
            &query_ctx,
            has_symbol_hits,
            has_bm25_hits,
            has_bm25_docs_hint,
        ) {
            let vector = self.vector_search(query, snapshot_version, vector_limit)?;
            if !vector.is_empty() {
                rank_lists.push(vector);
            }
        }

        let hit_lookup = first_hit_by_key(&rank_lists);
        let mut prior_cache: HashMap<String, PriorBreakdown> = HashMap::new();

        let fused = rrf_fuse(&rank_lists, self.cfg.rrf_k as f32);
        let mut adjusted_fused = fused
            .iter()
            .map(|(path_key, base_score)| {
                let prior = hit_lookup
                    .get(path_key)
                    .map(|h| cached_score_prior(self, &query_ctx, h, &mut prior_cache).total)
                    .unwrap_or(1.0);
                (path_key.clone(), *base_score * prior)
            })
            .collect::<Vec<_>>();
        adjusted_fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let symbol_hint = query
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':');
        let mut ordered_keys: Vec<String> = Vec::new();
        let mut seen = HashSet::new();

        if !exact.is_empty() {
            for hit in &exact {
                let k = make_key(hit);
                if seen.insert(k.clone()) {
                    ordered_keys.push(k);
                }
            }
        } else if symbol_hint && !prefix.is_empty() {
            for hit in &prefix {
                let k = make_key(hit);
                if seen.insert(k.clone()) {
                    ordered_keys.push(k);
                }
            }
        }

        for (path_key, _) in &adjusted_fused {
            if seen.insert(path_key.clone()) {
                ordered_keys.push(path_key.clone());
            }
        }

        let ordered_keys =
            dedupe_ranked_keys_by_path(&ordered_keys, &rank_lists, self.cfg.topn_final);
        let fused_scores: HashMap<String, f32> = adjusted_fused.into_iter().collect();
        let mut out = Vec::new();
        for (idx, path_key) in ordered_keys.into_iter().enumerate() {
            if let Some(hit) = hit_lookup.get(&path_key).cloned() {
                let prior = cached_score_prior(self, &query_ctx, &hit, &mut prior_cache);
                out.push(GroundedHit {
                    rank: (idx + 1) as u32,
                    path: hit.path,
                    domain_id: hit.domain_id,
                    start_line: hit.start_line,
                    end_line: hit.end_line,
                    file_hash: hit.file_hash,
                    snapshot_version,
                    active_version,
                    score_rrf: fused_scores.get(&path_key).copied().unwrap_or(0.0),
                    score_symbol: hit.score_symbol,
                    score_bm25: hit.score_bm25,
                    score_vector: hit.score_vector,
                    source: hit.source,
                    symbol_kind: hit.symbol_kind,
                    stale: snapshot_version != active_version,
                    trust_label: hit.trust_label,
                    trust_level: hit.trust_level,
                    why_selected: format!(
                        "{}; prior(path={:.2}, recency={:.2})",
                        hit.why_selected, prior.path, prior.recency
                    ),
                });
            }
        }

        let rendered = self.guard.redact_sensitive_hits(out);
        self.store_search_result(cache_key, &rendered);
        Ok(rendered)
    }

    pub fn active_version(&self) -> Result<u64> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut stmt = conn.prepare_cached(
            "SELECT version FROM index_versions WHERE state='active' ORDER BY version DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            return Ok(row.get(0)?);
        }
        Ok(0)
    }

    pub fn indexing_status(&self) -> Result<Option<IndexingStatus>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut stmt = conn.prepare_cached(
            "SELECT value FROM runtime_state WHERE key='indexing_status' LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let raw: String = row.get(0)?;
            let parsed = serde_json::from_str::<IndexingStatus>(&raw).ok();
            return Ok(parsed);
        }
        Ok(None)
    }

    fn symbol_exact(&self, query: &str, version: u64) -> Result<Vec<PartialHit>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let variants = symbol_query_variants(query);
        if variants.is_empty() {
            return Ok(Vec::new());
        }

        let exact_hits = self.symbol_exact_variants_query(&conn, version, &variants, false)?;
        if !exact_hits.is_empty() {
            return Ok(exact_hits);
        }

        let lowered = lower_dedup_variants(variants);
        self.symbol_exact_variants_query(&conn, version, &lowered, true)
    }

    fn symbol_exact_variants_query(
        &self,
        conn: &Connection,
        version: u64,
        variants: &[String],
        normalized: bool,
    ) -> Result<Vec<PartialHit>> {
        if variants.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = vec!["?"; variants.len()].join(", ");
        let name_expr = if normalized {
            "LOWER(s.symbol_name)"
        } else {
            "s.symbol_name"
        };
        let sql = format!(
            r#"
            SELECT s.path, s.line_start, s.line_end, s.symbol_kind, s.file_hash, f.domain_id, f.trust_label, f.modified_unix_ms
            FROM symbols s
            JOIN files f
              ON f.path = s.path
             AND f.index_version = s.index_version
            WHERE s.index_version=? AND {name_expr} IN ({placeholders})
            ORDER BY s.exported DESC
            LIMIT ?
            "#
        );
        let mut stmt = conn.prepare_cached(&sql)?;

        let mut bind = Vec::with_capacity(variants.len() + 2);
        bind.push(Value::Integer(version as i64));
        for variant in variants {
            bind.push(Value::Text(variant.clone()));
        }
        bind.push(Value::Integer(self.cfg.topn_symbol as i64));

        let rows = stmt.query_map(params_from_iter(bind), |row| {
            let trust_label: String = row.get(6)?;
            Ok(PartialHit {
                path: row.get(0)?,
                start_line: row.get::<_, i64>(1)? as u32,
                end_line: row.get::<_, i64>(2)? as u32,
                symbol_kind: Some(row.get(3)?),
                file_hash: row.get(4)?,
                modified_unix_ms: Some(row.get(7)?),
                domain_id: row.get(5)?,
                trust_level: trust_level_from_label(&trust_label),
                trust_label,
                source: HitSource::Symbol,
                score_symbol: Some(self.cfg.symbol_exact_boost),
                score_bm25: None,
                score_vector: None,
                why_selected: "exact symbol match".to_string(),
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn symbol_prefix(&self, query: &str, version: u64) -> Result<Vec<PartialHit>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let variants = lower_dedup_variants(symbol_query_variants(query));
        if variants.is_empty() {
            return Ok(Vec::new());
        }

        let like_clauses = vec!["LOWER(symbol_name) LIKE ?"; variants.len()].join(" OR ");
        let sql = format!(
            r#"
            SELECT s.path, s.line_start, s.line_end, s.symbol_kind, s.file_hash, f.domain_id, f.trust_label, f.modified_unix_ms
            FROM symbols s
            JOIN files f
              ON f.path = s.path
             AND f.index_version = s.index_version
            WHERE s.index_version=? AND ({like_clauses})
            ORDER BY s.exported DESC
            LIMIT ?
            "#
        );
        let mut stmt = conn.prepare_cached(&sql)?;

        let mut bind = Vec::with_capacity(variants.len() + 2);
        bind.push(Value::Integer(version as i64));
        for variant in variants {
            bind.push(Value::Text(format!("{variant}%")));
        }
        bind.push(Value::Integer(self.cfg.topn_symbol as i64));

        let rows = stmt.query_map(params_from_iter(bind), |row| {
            let trust_label: String = row.get(6)?;
            Ok(PartialHit {
                path: row.get(0)?,
                start_line: row.get::<_, i64>(1)? as u32,
                end_line: row.get::<_, i64>(2)? as u32,
                symbol_kind: Some(row.get(3)?),
                file_hash: row.get(4)?,
                modified_unix_ms: Some(row.get(7)?),
                domain_id: row.get(5)?,
                trust_level: trust_level_from_label(&trust_label),
                trust_label,
                source: HitSource::Symbol,
                score_symbol: Some(self.cfg.symbol_prefix_boost),
                score_bm25: None,
                score_vector: None,
                why_selected: "prefix symbol match".to_string(),
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn bm25(
        &self,
        query: &str,
        query_ctx: &QueryScoreContext,
        version: u64,
        topn: usize,
    ) -> Result<Vec<PartialHit>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let path_filter = bm25_path_filter_clause(query_ctx)
            .map(|clause| format!(" AND ({clause})"))
            .unwrap_or_default();
        let sql = format!(
            r#"
            SELECT m.path, m.start_line, m.end_line, m.file_hash, m.domain_id, m.trust_label
            FROM chunks_fts
            JOIN chunks_meta m ON m.chunk_id = chunks_fts.chunk_id AND m.path = chunks_fts.path
            WHERE chunks_fts.content MATCH ?1 AND m.index_version=?2{path_filter}
            ORDER BY bm25(chunks_fts)
            LIMIT ?3
            "#,
        );
        let mut stmt = conn.prepare_cached(&sql)?;

        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let topn = topn.max(1);
        let per_variant_limit = (topn as i64).max(1);

        for variant in bm25_query_variants(query) {
            if out.len() >= topn {
                break;
            }

            let rows = match stmt.query_map(params![variant, version, per_variant_limit], |row| {
                let trust_label: String = row.get(5)?;
                Ok(PartialHit {
                    path: row.get(0)?,
                    start_line: row.get::<_, i64>(1)? as u32,
                    end_line: row.get::<_, i64>(2)? as u32,
                    file_hash: row.get(3)?,
                    modified_unix_ms: None,
                    domain_id: row.get(4)?,
                    trust_level: trust_level_from_label(&trust_label),
                    trust_label,
                    source: HitSource::BM25,
                    symbol_kind: None,
                    score_symbol: None,
                    score_bm25: Some(1.0),
                    score_vector: None,
                    why_selected: "bm25 keyword match".to_string(),
                })
            }) {
                Ok(rows) => rows,
                Err(_) => continue,
            };

            for hit in rows.filter_map(|r| r.ok()) {
                let key = make_key(&hit);
                if seen.insert(key) {
                    out.push(hit);
                }
                if out.len() >= topn {
                    break;
                }
            }
        }

        Ok(out)
    }

    fn vector_search(&self, query: &str, version: u64, topn: usize) -> Result<Vec<PartialHit>> {
        let query_embedding = embed_text_hash(query, self.embed_dim);
        let topn = topn.max(1);

        #[cfg(feature = "lancedb")]
        if vector_backend_enabled() {
            if let Ok(hits) = self.vector_search_lancedb(&query_embedding, version, topn) {
                if !hits.is_empty() {
                    return Ok(hits);
                }
            }
        }

        let candidate_pool = (topn * 50).max(500);
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let mut stmt = conn.prepare_cached(
            r#"
            SELECT m.path, m.start_line, m.end_line, m.file_hash, m.domain_id, m.trust_label, v.embedding_json
            FROM chunks_vec v
            JOIN chunks_meta m
              ON m.chunk_id = v.chunk_id
             AND m.path = v.path
             AND m.index_version = v.index_version
            WHERE v.index_version=?1
            LIMIT ?2
            "#,
        )?;

        let rows = stmt.query_map(params![version, candidate_pool as i64], |row| {
            let embedding_json: String = row.get(6)?;
            let trust_label: String = row.get(5)?;
            let embedding: Vec<f32> = serde_json::from_str(&embedding_json).unwrap_or_default();
            let score = cosine_similarity(&query_embedding, &embedding);
            Ok(PartialHit {
                path: row.get(0)?,
                start_line: row.get::<_, i64>(1)? as u32,
                end_line: row.get::<_, i64>(2)? as u32,
                file_hash: row.get(3)?,
                modified_unix_ms: None,
                domain_id: row.get(4)?,
                trust_level: trust_level_from_label(&trust_label),
                trust_label,
                source: HitSource::Vector,
                symbol_kind: None,
                score_symbol: None,
                score_bm25: None,
                score_vector: Some(score),
                why_selected: "vector semantic similarity".to_string(),
            })
        })?;

        let mut hits: Vec<PartialHit> = rows
            .filter_map(|r| r.ok())
            .filter(|h| h.score_vector.unwrap_or(0.0) > 0.0)
            .collect();

        hits.sort_by(|a, b| {
            b.score_vector
                .unwrap_or(0.0)
                .partial_cmp(&a.score_vector.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(topn);
        Ok(hits)
    }

    fn vector_limit_for_query(
        &self,
        query: &QueryScoreContext,
        has_symbol_hits: bool,
        has_bm25_hits: bool,
        has_bm25_docs_hint: bool,
    ) -> Option<usize> {
        if !query.should_run_vector_search(has_symbol_hits) {
            return None;
        }

        let topn = self.cfg.topn_vector.max(1);
        if query.is_narrative_docs && has_bm25_docs_hint {
            if query.is_docs_schema_like {
                return Some(topn.min(8).max(1));
            }
            if query.overlap_terms.len() >= 7 {
                return None;
            }
            return Some(topn.min(6).max(1));
        }
        if query.is_narrative_docs && has_bm25_hits {
            if query.is_docs_schema_like {
                return Some(topn.min(8).max(1));
            }
            if query.overlap_terms.len() <= 4 {
                return Some(topn.min(6).max(1));
            }
            return Some(topn.min(8).max(1));
        }

        Some(topn)
    }

    fn bm25_limit_for_query(&self, query: &QueryScoreContext) -> usize {
        let topn = self.cfg.topn_bm25.max(1);

        if query.is_workflow_like || query.is_systemd_unit_like {
            return topn.min(6).max(1);
        }
        if query.is_config_like || query.is_command_like {
            return topn.min(8).max(1);
        }
        if query.is_narrative_docs {
            if query.is_docs_schema_like {
                return topn.min(10).max(1);
            }
            return topn.min(8).max(1);
        }

        topn
    }

    fn refresh_caches_for_snapshot(&self, snapshot_version: u64) {
        if self.path_cache_version.load(AtomicOrdering::Relaxed) == snapshot_version {
            return;
        }

        let mut path_cache = self
            .path_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut search_cache = self
            .search_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if self.path_cache_version.load(AtomicOrdering::Relaxed) != snapshot_version {
            path_cache.clear();
            search_cache.clear();
            self.path_cache_version
                .store(snapshot_version, AtomicOrdering::Relaxed);
        }
    }

    fn cached_search_result(&self, cache_key: &str) -> Option<Vec<GroundedHit>> {
        self.search_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(cache_key)
            .cloned()
    }

    fn store_search_result(&self, cache_key: String, hits: &[GroundedHit]) {
        let mut cache = self
            .search_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !cache.contains_key(&cache_key) && cache.len() >= SEARCH_RESULT_CACHE_MAX_ENTRIES {
            cache.clear();
        }
        cache.insert(cache_key, hits.to_vec());
    }

    fn path_score_context(&self, path: &str, modified_unix_ms: Option<i64>) -> PathScoreContext {
        let now = SystemTime::now();
        if let Some(cached) = self
            .path_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(path)
            .cloned()
        {
            let is_fresh = now
                .duration_since(cached.cached_at)
                .map(|age| age <= PATH_SCORE_CACHE_TTL)
                .unwrap_or(false);
            if is_fresh {
                return cached.ctx;
            }
        }

        let lower = path.to_ascii_lowercase();
        let file_name = Path::new(path)
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or(path);

        let ctx = PathScoreContext {
            lower: lower.clone(),
            path_terms: ranking_overlap_terms(path),
            file_name_terms: ranking_overlap_terms(file_name),
            path_prior: self.path_prior_multiplier_from_lower(&lower),
            recency_prior: modified_unix_ms
                .and_then(|ts| self.recency_prior_from_modified_unix_ms(ts))
                .unwrap_or_else(|| self.recency_prior_from_path(path)),
        };

        self.path_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(
                path.to_string(),
                CachedPathScoreContext {
                    ctx: ctx.clone(),
                    cached_at: now,
                },
            );

        ctx
    }

    fn render_direct_hits(
        &self,
        query: &QueryScoreContext,
        hits: Vec<PartialHit>,
        snapshot_version: u64,
        active_version: u64,
    ) -> Vec<GroundedHit> {
        let mut prior_cache: HashMap<String, PriorBreakdown> = HashMap::new();
        let mut ranked = hits
            .into_iter()
            .enumerate()
            .map(|(idx, hit)| {
                let prior = cached_score_prior(self, query, &hit, &mut prior_cache);
                let score = hit.score_symbol.unwrap_or(1.0) * prior.total;
                (idx, hit, prior, score)
            })
            .collect::<Vec<_>>();

        ranked.sort_by(|left, right| {
            right
                .3
                .partial_cmp(&left.3)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.0.cmp(&right.0))
        });

        let mut out = Vec::new();
        let mut seen_paths = HashSet::new();
        for (_idx, hit, prior, score) in ranked {
            if !seen_paths.insert(hit.path.clone()) {
                continue;
            }
            out.push(GroundedHit {
                rank: (out.len() + 1) as u32,
                path: hit.path,
                domain_id: hit.domain_id,
                start_line: hit.start_line,
                end_line: hit.end_line,
                file_hash: hit.file_hash,
                snapshot_version,
                active_version,
                score_rrf: score,
                score_symbol: hit.score_symbol,
                score_bm25: hit.score_bm25,
                score_vector: hit.score_vector,
                source: hit.source,
                symbol_kind: hit.symbol_kind,
                stale: snapshot_version != active_version,
                trust_label: hit.trust_label,
                trust_level: hit.trust_level,
                why_selected: format!(
                    "{}; exact-symbol-fast-path; prior(path={:.2}, recency={:.2})",
                    hit.why_selected, prior.path, prior.recency
                ),
            });
            if out.len() >= self.cfg.topn_final {
                break;
            }
        }

        out
    }

    fn score_prior(&self, query: &QueryScoreContext, path: &PathScoreContext) -> PriorBreakdown {
        let total = path.path_prior
            * self.query_path_overlap_multiplier(query, path)
            * self.file_name_query_overlap_multiplier(query, path)
            * self.config_query_path_multiplier(query, path)
            * self.narrative_docs_query_multiplier(query, path)
            * self.command_query_path_multiplier(query, path)
            * self.workflow_query_path_multiplier(query, path)
            * self.systemd_unit_query_path_multiplier(query, path)
            * path.recency_prior;

        PriorBreakdown {
            total,
            path: path.path_prior,
            recency: path.recency_prior,
        }
    }

    fn path_prior_multiplier_from_lower(&self, lower: &str) -> f32 {
        let mut mult = 1.0f32;
        let is_code = is_code_path(lower);

        if is_code {
            mult *= self.cfg.code_path_boost.max(0.1);
        }
        if is_docs_path(lower) {
            mult *= self.cfg.docs_path_penalty.max(0.1);
        }
        if is_test_path(lower) {
            mult *= self.cfg.test_path_penalty.max(0.1);
        }
        if !is_code && is_asset_path(lower) {
            // Keep non-code assets retrievable without letting them outrank likely source hits.
            mult *= self.cfg.asset_path_penalty.max(0.1);
        }
        if is_generated_artifact_path(lower) {
            // Keep generated artifacts searchable, but prevent them from shadowing source paths.
            mult *= 0.30;
        }
        mult
    }

    fn query_path_overlap_multiplier(
        &self,
        query: &QueryScoreContext,
        path: &PathScoreContext,
    ) -> f32 {
        if query.overlap_terms.is_empty() || path.path_terms.is_empty() {
            return 1.0;
        }

        let matched = query
            .overlap_terms
            .iter()
            .filter(|q| {
                path.path_terms
                    .iter()
                    .any(|p| p == *q || p.starts_with(q.as_str()) || q.starts_with(p.as_str()))
            })
            .count();

        if matched == 0 {
            return 1.0;
        }

        let ratio = matched as f32 / query.overlap_terms.len() as f32;
        1.0 + 0.35 * ratio.clamp(0.0, 1.0)
    }

    fn file_name_query_overlap_multiplier(
        &self,
        query: &QueryScoreContext,
        path: &PathScoreContext,
    ) -> f32 {
        if query.overlap_terms.is_empty() || path.file_name_terms.is_empty() {
            return 1.0;
        }

        let matched = query
            .overlap_terms
            .iter()
            .filter(|q| {
                path.file_name_terms
                    .iter()
                    .any(|p| p == *q || p.starts_with(q.as_str()) || q.starts_with(p.as_str()))
            })
            .count();

        if matched == 0 {
            return 1.0;
        }

        let ratio = matched as f32 / query.overlap_terms.len() as f32;
        1.0 + 0.55 * ratio.clamp(0.0, 1.0)
    }

    fn recency_prior_from_modified_unix_ms(&self, modified_unix_ms: i64) -> Option<f32> {
        if modified_unix_ms <= 0 {
            return None;
        }

        let now_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_millis() as i128;
        let age_ms = (now_ms - modified_unix_ms as i128).max(0);
        Some(self.recency_prior_from_age_hours(age_ms as f32 / 3_600_000.0))
    }

    fn recency_prior_from_age_hours(&self, age_hours: f32) -> f32 {
        let half_life = self.cfg.recency_half_life_hours.max(0.1);
        let min_boost = self.cfg.recency_min_boost.max(0.1);
        let max_boost = self.cfg.recency_max_boost.max(min_boost);
        let decay = 0.5f32.powf(age_hours / half_life);
        min_boost + (max_boost - min_boost) * decay.clamp(0.0, 1.0)
    }

    fn recency_prior_from_path(&self, path: &str) -> f32 {
        let full_path = self
            .guard
            .resolve_virtual_path(path)
            .map(|resolved| resolved.absolute_path)
            .unwrap_or_else(|| {
                let mut fallback = self.repo_root.clone();
                fallback.push(path);
                fallback
            });
        let Ok(meta) = std::fs::metadata(&full_path) else {
            return 1.0;
        };
        let Ok(modified) = meta.modified() else {
            return 1.0;
        };
        let Ok(age) = SystemTime::now().duration_since(modified) else {
            return self
                .cfg
                .recency_max_boost
                .max(self.cfg.recency_min_boost.max(0.1));
        };
        self.recency_prior_from_age_hours(age.as_secs_f32() / 3600.0)
    }

    fn config_query_path_multiplier(
        &self,
        query: &QueryScoreContext,
        path: &PathScoreContext,
    ) -> f32 {
        if !query.is_config_like {
            return 1.0;
        }

        if is_config_path(&path.lower) {
            return 1.30;
        }
        if is_docs_path(&path.lower) {
            return 0.95;
        }
        if is_code_path(&path.lower) {
            return 0.88;
        }
        1.0
    }

    fn narrative_docs_query_multiplier(
        &self,
        query: &QueryScoreContext,
        path: &PathScoreContext,
    ) -> f32 {
        if !query.is_narrative_docs {
            return 1.0;
        }

        if query.is_docs_schema_like {
            if is_docs_path(&path.lower) {
                return 2.80;
            }
            if is_config_path(&path.lower) {
                return 0.65;
            }
            if is_script_path(&path.lower) {
                return 0.80;
            }
            if is_code_path(&path.lower) {
                return 0.35;
            }
            return 1.0;
        }

        if is_docs_path(&path.lower) {
            return 1.28;
        }
        if is_script_path(&path.lower) {
            return 0.86;
        }
        if is_code_path(&path.lower) {
            return 0.92;
        }
        1.0
    }

    fn command_query_path_multiplier(
        &self,
        query: &QueryScoreContext,
        path: &PathScoreContext,
    ) -> f32 {
        if !query.is_command_like {
            return 1.0;
        }

        if is_script_path(&path.lower) {
            return 2.10;
        }
        if is_docs_path(&path.lower) {
            return 0.68;
        }
        if is_config_path(&path.lower) {
            return 0.68;
        }
        if is_code_path(&path.lower) {
            return 0.72;
        }
        1.0
    }

    fn workflow_query_path_multiplier(
        &self,
        query: &QueryScoreContext,
        path: &PathScoreContext,
    ) -> f32 {
        if !query.is_workflow_like {
            return 1.0;
        }

        if is_workflow_path(&path.lower) {
            return 1.90;
        }
        if is_docs_path(&path.lower) {
            return 0.82;
        }
        if is_config_path(&path.lower) {
            return 0.86;
        }
        if is_code_path(&path.lower) {
            return 0.72;
        }
        1.0
    }

    fn systemd_unit_query_path_multiplier(
        &self,
        query: &QueryScoreContext,
        path: &PathScoreContext,
    ) -> f32 {
        if !query.is_systemd_unit_like {
            return 1.0;
        }

        if is_systemd_unit_path(&path.lower) {
            return 1.40;
        }
        if is_docs_path(&path.lower) {
            return 0.92;
        }
        if is_config_path(&path.lower) {
            return 0.95;
        }
        if is_code_path(&path.lower) {
            return 0.88;
        }
        1.0
    }

    #[cfg(feature = "lancedb")]
    fn domain_metadata_for_path(&self, path: &str) -> (String, String, TrustLevel) {
        if let Some(resolved) = self.guard.resolve_virtual_path(path) {
            let trust_level = trust_level_from_label(&resolved.trust_label);
            return (resolved.domain_id, resolved.trust_label, trust_level);
        }

        (
            "default".to_string(),
            "trusted".to_string(),
            TrustLevel::Trusted,
        )
    }

    #[cfg(feature = "lancedb")]
    fn vector_search_lancedb(
        &self,
        query_embedding: &[f32],
        version: u64,
        topn: usize,
    ) -> Result<Vec<PartialHit>> {
        use arrow_array::{Int32Array, RecordBatch, StringArray};
        use futures_util::TryStreamExt;
        use lancedb::connect;
        use lancedb::query::{ExecutableQuery, QueryBase};

        let uri = std::env::var("SEMANTICFS_LANCEDB_URI")
            .unwrap_or_else(|_| "./.semanticfs/lancedb".to_string());
        let table_name = format!("chunks_v{}", version);
        let topn = topn.max(1);

        let batches = run_async(async move {
            let db = connect(&uri).execute().await?;
            let table = db.open_table(&table_name).execute().await?;
            let stream = table
                .query()
                .limit(topn)
                .nearest_to(query_embedding)?
                .execute()
                .await?;
            let batches = stream.try_collect::<Vec<_>>().await?;
            Ok::<Vec<RecordBatch>, anyhow::Error>(batches)
        })?;

        let mut hits = Vec::new();
        for batch in batches {
            let Some(path_col_any) = batch.column_by_name("path") else {
                continue;
            };
            let Some(start_col_any) = batch.column_by_name("start_line") else {
                continue;
            };
            let Some(end_col_any) = batch.column_by_name("end_line") else {
                continue;
            };
            let Some(hash_col_any) = batch.column_by_name("file_hash") else {
                continue;
            };
            let domain_col = batch
                .column_by_name("domain_id")
                .and_then(|col| col.as_any().downcast_ref::<StringArray>());
            let trust_col = batch
                .column_by_name("trust_label")
                .and_then(|col| col.as_any().downcast_ref::<StringArray>());

            let Some(path_col) = path_col_any.as_any().downcast_ref::<StringArray>() else {
                continue;
            };
            let Some(start_col) = start_col_any.as_any().downcast_ref::<Int32Array>() else {
                continue;
            };
            let Some(end_col) = end_col_any.as_any().downcast_ref::<Int32Array>() else {
                continue;
            };
            let Some(hash_col) = hash_col_any.as_any().downcast_ref::<StringArray>() else {
                continue;
            };

            for i in 0..batch.num_rows() {
                let path = path_col.value(i).to_string();
                let (domain_id, trust_label, trust_level) =
                    if let (Some(domain_col), Some(trust_col)) = (domain_col, trust_col) {
                        let trust_label = trust_col.value(i).to_string();
                        (
                            domain_col.value(i).to_string(),
                            trust_label.clone(),
                            trust_level_from_label(&trust_label),
                        )
                    } else {
                        self.domain_metadata_for_path(&path)
                    };
                hits.push(PartialHit {
                    path,
                    domain_id,
                    start_line: start_col.value(i) as u32,
                    end_line: end_col.value(i) as u32,
                    file_hash: hash_col.value(i).to_string(),
                    modified_unix_ms: None,
                    trust_label,
                    trust_level,
                    source: HitSource::Vector,
                    symbol_kind: None,
                    score_symbol: None,
                    score_bm25: None,
                    score_vector: Some(1.0),
                    why_selected: "lancedb nearest vector".to_string(),
                });
            }
        }

        hits.sort_by(|a, b| {
            b.score_vector
                .unwrap_or(0.0)
                .partial_cmp(&a.score_vector.unwrap_or(0.0))
                .unwrap_or(Ordering::Equal)
        });
        hits.truncate(self.cfg.topn_vector);
        Ok(hits)
    }
}

#[cfg(feature = "lancedb")]
fn run_async<F, T>(fut: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return Ok(tokio::task::block_in_place(|| handle.block_on(fut))?);
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(fut)
}

#[cfg(feature = "lancedb")]
fn vector_backend_enabled() -> bool {
    std::env::var("SEMANTICFS_VECTOR_BACKEND")
        .map(|v| v.eq_ignore_ascii_case("lancedb"))
        .unwrap_or(true)
}

fn trust_level_from_label(label: &str) -> TrustLevel {
    if label.eq_ignore_ascii_case("untrusted") {
        TrustLevel::Untrusted
    } else {
        TrustLevel::Trusted
    }
}

fn is_docs_path(path: &str) -> bool {
    path.contains("/docs/")
        || path.ends_with(".md")
        || path.ends_with(".rst")
        || path.ends_with(".adoc")
        || path.ends_with("readme")
        || path.ends_with("readme.md")
}

fn is_test_path(path: &str) -> bool {
    path.contains("/tests/")
        || path.contains("/test/")
        || path.ends_with("_test.rs")
        || path.ends_with("_test.py")
        || path.ends_with(".spec.ts")
        || path.ends_with(".test.ts")
}

fn is_code_path(path: &str) -> bool {
    path.ends_with(".rs")
        || path.ends_with(".py")
        || path.ends_with(".ts")
        || path.ends_with(".tsx")
        || path.ends_with(".js")
        || path.ends_with(".jsx")
        || path.ends_with(".go")
        || path.ends_with(".java")
        || path.ends_with(".c")
        || path.ends_with(".cpp")
        || path.ends_with(".h")
        || path.ends_with(".hpp")
        || path.ends_with(".cs")
}

fn is_script_path(path: &str) -> bool {
    path.starts_with("scripts/")
        || path.contains("/scripts/")
        || path.ends_with(".ps1")
        || path.ends_with(".sh")
        || path.ends_with(".bat")
}

fn is_workflow_path(path: &str) -> bool {
    (path.starts_with("github/workflows/")
        || path.starts_with(".github/workflows/")
        || path.contains("/workflows/"))
        && (path.ends_with(".yml") || path.ends_with(".yaml"))
}

fn is_systemd_unit_path(path: &str) -> bool {
    path.starts_with("systemd/")
        || path.contains("/systemd/")
        || path.ends_with(".service")
        || path.ends_with(".socket")
        || path.ends_with(".timer")
        || path.ends_with(".mount")
        || path.ends_with(".target")
}

fn is_config_path(path: &str) -> bool {
    path.starts_with("config/")
        || path.contains("/config/")
        || path.ends_with(".toml")
        || path.ends_with(".yaml")
        || path.ends_with(".yml")
        || path.ends_with(".json")
        || path.ends_with(".ini")
        || path.ends_with(".conf")
}

fn is_generated_artifact_path(path: &str) -> bool {
    path.contains("/.next/")
        || path.contains("/.nuxt/")
        || path.contains("/.svelte-kit/")
        || path.contains("/.turbo/")
        || path.contains("/.dart_tool/")
        || path.contains("/__generated__/")
        || path.contains("/dist/")
        || path.contains("/build/")
        || path.contains("/out/")
        || path.contains("/coverage/")
        || path.contains("/target/")
        || path.ends_with(".min.js")
}

fn is_asset_path(path: &str) -> bool {
    path.contains("/assets/")
        || path.contains("/static/")
        || path.contains("/media/")
        || path.ends_with(".dat")
        || path.ends_with(".bin")
        || path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".gif")
        || path.ends_with(".svg")
        || path.ends_with(".ico")
        || path.ends_with(".mp3")
        || path.ends_with(".wav")
        || path.ends_with(".ogg")
        || path.ends_with(".pdf")
        || path.ends_with(".zip")
        || path.ends_with(".onnx")
        || path.ends_with(".pt")
        || path.ends_with(".ckpt")
        || path.ends_with(".pb")
        || path.ends_with(".tflite")
}

fn is_config_like_query(query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return false;
    }

    let lower = trimmed.to_ascii_lowercase();
    if [
        ".toml",
        ".yaml",
        ".yml",
        ".json",
        "mount_point",
        "allow_roots",
        "deny_globs",
        "trust_label",
        "metrics_bind",
        "health_bind",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return true;
    }

    let punctuation_count = trimmed
        .chars()
        .filter(|c| matches!(*c, '=' | '[' | ']' | '{' | '}' | '"' | '\''))
        .count();

    trimmed.contains('=') && punctuation_count >= 2
}

fn is_symbol_like_query(query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() || trimmed.chars().any(|c| c.is_whitespace()) {
        return false;
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '-'))
    {
        return false;
    }

    trimmed.contains('_') || trimmed.chars().any(|c| c.is_ascii_uppercase())
}

fn is_command_like_query(query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return false;
    }

    let lower = trimmed.to_ascii_lowercase();
    [
        "run-step",
        "ap.add_argument(",
        "invoke-expression",
        "powershell",
        "executionpolicy",
        " -file ",
        ".ps1",
        ".sh",
        "bash -lc",
        "git ls-files",
        "cargo run",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn is_workflow_like_query(query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return false;
    }

    let lower = trimmed.to_ascii_lowercase();
    [
        "runs-on:",
        "uses:",
        "workflow_dispatch",
        "pull_request:",
        "actions/",
        "ubuntu-latest",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn is_systemd_unit_like_query(query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return false;
    }

    let lower = trimmed.to_ascii_lowercase();
    [
        "[unit]",
        "[service]",
        "[install]",
        "description=",
        "after=",
        "execstart=",
        "wantedby=",
        "restartsec=",
        "type=simple",
        ".service",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn is_narrative_docs_query(query: &str) -> bool {
    if is_config_like_query(query) || is_command_like_query(query) {
        return false;
    }

    let terms = query_terms(query);
    if terms.len() < 5 {
        return false;
    }

    let alpha_len = query.chars().filter(|c| c.is_ascii_alphabetic()).count();
    let punct = query
        .chars()
        .filter(|c| matches!(*c, '=' | '[' | ']' | '{' | '}' | '"' | '\'' | '`'))
        .count();

    alpha_len >= 20 && punct <= 2
}

fn is_docs_schema_like_query(query: &str, is_narrative_docs: bool) -> bool {
    if !is_narrative_docs {
        return false;
    }

    query.chars().filter(|c| *c == '_').count() >= 3
}

fn symbol_query_variants(query: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return variants;
    }

    push_unique_variant(&mut variants, trimmed.to_string());

    let collapsed_ws = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
    push_unique_variant(&mut variants, collapsed_ws.clone());
    push_unique_variant(&mut variants, collapsed_ws.replace(' ', "_"));

    let terms = query_terms(trimmed);
    if !terms.is_empty() {
        let snake = terms.join("_");
        let compact = terms.join("");
        push_unique_variant(&mut variants, snake.clone());
        push_unique_variant(&mut variants, format!("_{snake}"));
        push_unique_variant(&mut variants, compact);
        push_unique_variant(&mut variants, to_pascal_case(&terms));
    }

    if let Some(without_prefix) = trimmed.strip_prefix('_') {
        push_unique_variant(&mut variants, without_prefix.to_string());
    }

    variants
}

fn bm25_query_variants(query: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return variants;
    }

    push_unique_variant(&mut variants, trimmed.to_string());
    push_unique_variant(&mut variants, trimmed.replace('_', " "));

    let terms = query_terms(trimmed);
    if !terms.is_empty() {
        push_unique_variant(&mut variants, terms.join(" "));
    }

    let mut deduped = Vec::with_capacity(variants.len());
    let mut seen_lower = HashSet::new();
    for variant in variants {
        let lowered = variant.to_ascii_lowercase();
        if seen_lower.insert(lowered) {
            deduped.push(variant);
        }
    }

    deduped
}

fn bm25_path_filter_clause(query: &QueryScoreContext) -> Option<&'static str> {
    if query.is_workflow_like {
        return Some(
            "m.path LIKE 'github/workflows/%' \
             OR m.path LIKE '.github/workflows/%' \
             OR m.path LIKE '%/workflows/%.yml' \
             OR m.path LIKE '%/workflows/%.yaml'",
        );
    }

    if query.is_systemd_unit_like {
        return Some(
            "m.path LIKE 'systemd/%' \
             OR m.path LIKE '%/systemd/%' \
             OR m.path LIKE '%.service' \
             OR m.path LIKE '%.socket' \
             OR m.path LIKE '%.timer' \
             OR m.path LIKE '%.mount' \
             OR m.path LIKE '%.target'",
        );
    }

    if query.is_command_like {
        return Some(
            "m.path LIKE 'scripts/%' \
             OR m.path LIKE '%/scripts/%' \
             OR m.path LIKE '%.ps1' \
             OR m.path LIKE '%.sh' \
             OR m.path LIKE '%.bat'",
        );
    }

    None
}

fn lower_dedup_variants(variants: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for variant in variants {
        let lowered = variant.to_ascii_lowercase();
        if !out.iter().any(|v| v == &lowered) {
            out.push(lowered);
        }
    }
    out
}

fn query_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut current = String::new();
    for c in query.chars() {
        if c.is_ascii_alphanumeric() {
            current.push(c.to_ascii_lowercase());
        } else if !current.is_empty() {
            terms.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        terms.push(current);
    }
    terms
}

fn ranking_overlap_terms(text: &str) -> Vec<String> {
    query_terms(text)
        .into_iter()
        .filter(|term| term.len() >= 3)
        .collect()
}

fn to_pascal_case(terms: &[String]) -> String {
    let mut out = String::new();
    for term in terms {
        let mut chars = term.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

fn push_unique_variant(variants: &mut Vec<String>, candidate: String) {
    let normalized = candidate.trim();
    if normalized.is_empty() {
        return;
    }
    if !variants.iter().any(|v| v == normalized) {
        variants.push(normalized.to_string());
    }
}

fn rrf_fuse(rank_lists: &[Vec<PartialHit>], k: f32) -> Vec<(String, f32)> {
    let mut scores: HashMap<String, f32> = HashMap::new();

    for ranked in rank_lists {
        for (idx, hit) in ranked.iter().enumerate() {
            let key = make_key(hit);
            let rank = (idx + 1) as f32;
            let score = 1.0 / (k + rank);
            *scores.entry(key).or_insert(0.0) += score;
        }
    }

    let mut sorted: BTreeMap<(i64, String), f32> = BTreeMap::new();
    for (key, value) in scores {
        sorted.insert((-(value * 1_000_000.0) as i64, key), value);
    }

    sorted
        .into_iter()
        .map(|((_, key), score)| (key, score))
        .collect()
}

fn make_key(hit: &PartialHit) -> String {
    format!("{}:{}:{}", hit.path, hit.start_line, hit.end_line)
}

fn first_hit_by_key(rank_lists: &[Vec<PartialHit>]) -> HashMap<String, PartialHit> {
    let mut out = HashMap::new();
    for ranked in rank_lists {
        for hit in ranked {
            out.entry(make_key(hit)).or_insert_with(|| hit.clone());
        }
    }
    out
}

fn dedupe_ranked_keys_by_path(
    ordered_keys: &[String],
    rank_lists: &[Vec<PartialHit>],
    topn: usize,
) -> Vec<String> {
    let hit_lookup = first_hit_by_key(rank_lists);
    let mut out = Vec::new();
    let mut seen_paths = HashSet::new();

    for key in ordered_keys {
        if let Some(hit) = hit_lookup.get(key) {
            if seen_paths.insert(hit.path.clone()) {
                out.push(key.clone());
                if out.len() >= topn {
                    break;
                }
            }
        }
    }

    out
}

fn cached_score_prior(
    core: &RetrievalCore,
    query: &QueryScoreContext,
    hit: &PartialHit,
    cache: &mut HashMap<String, PriorBreakdown>,
) -> PriorBreakdown {
    if let Some(prior) = cache.get(&hit.path) {
        return *prior;
    }

    let path_ctx = core.path_score_context(&hit.path, hit.modified_unix_ms);
    let prior = core.score_prior(query, &path_ctx);
    cache.insert(hit.path.clone(), prior);
    prior
}

fn search_cache_key(query: &str, snapshot_version: u64, active_version: u64) -> String {
    format!(
        "{}|snapshot={}|active={}",
        query, snapshot_version, active_version
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(path: &str, line: u32) -> PartialHit {
        PartialHit {
            path: path.to_string(),
            domain_id: "default".to_string(),
            start_line: line,
            end_line: line + 1,
            file_hash: "abc".to_string(),
            modified_unix_ms: None,
            trust_label: "trusted".to_string(),
            trust_level: TrustLevel::Trusted,
            source: HitSource::BM25,
            symbol_kind: None,
            score_symbol: None,
            score_bm25: Some(1.0),
            score_vector: None,
            why_selected: "test".to_string(),
        }
    }

    #[test]
    fn rrf_prefers_items_ranked_by_multiple_lists() {
        let a = sample("src/a.rs", 1);
        let b = sample("src/b.rs", 1);
        let c = sample("src/c.rs", 1);

        let one = vec![a.clone(), b.clone()];
        let two = vec![a, c];
        let fused = rrf_fuse(&[one, two], 60.0);

        assert!(!fused.is_empty());
        assert!(fused[0].0.contains("src/a.rs"));
    }

    #[test]
    fn symbol_first_ordering_keeps_symbol_hits_first() {
        let exact = vec![sample("src/symbol.rs", 10), sample("src/alt.rs", 20)];
        let bm25 = vec![sample("docs/readme.md", 1), sample("src/symbol.rs", 10)];
        let fused = rrf_fuse(&[exact.clone(), bm25], 60.0);

        let mut ordered = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for h in &exact {
            let k = make_key(h);
            if seen.insert(k.clone()) {
                ordered.push(k);
            }
        }
        for (k, _) in &fused {
            if seen.insert(k.clone()) {
                ordered.push(k.clone());
            }
        }

        assert!(!ordered.is_empty());
        assert!(ordered[0].starts_with("src/symbol.rs"));
    }

    #[test]
    fn path_prior_penalizes_docs_vs_code() {
        assert!(is_code_path("src/lib/auth.rs"));
        assert!(is_docs_path("docs/architecture.md"));
        assert!(!is_docs_path("src/lib/auth.rs"));
    }

    #[test]
    fn generated_path_detection_flags_transpiled_output() {
        assert!(is_generated_artifact_path(
            "client/.next/dev/server/chunks/ssr/app_page.js"
        ));
        assert!(is_generated_artifact_path("web/dist/assets/main.js"));
        assert!(!is_generated_artifact_path("client/app/page.tsx"));
    }

    #[test]
    fn asset_path_detection_flags_non_code_assets() {
        assert!(is_asset_path(
            "FtcRobotController/src/main/assets/Skystone.dat"
        ));
        assert!(is_asset_path("web/static/logo.png"));
        assert!(!is_asset_path("src/lib/auth.rs"));
    }

    #[test]
    fn symbol_variants_include_human_and_symbol_forms() {
        let variants = symbol_query_variants("create bert model");
        assert!(variants.iter().any(|v| v == "create_bert_model"));
        assert!(variants.iter().any(|v| v == "_create_bert_model"));
        assert!(variants.iter().any(|v| v == "CreateBertModel"));
    }

    #[test]
    fn bm25_variants_sanitize_symbol_query() {
        let variants = bm25_query_variants("_add_metrics");
        assert!(variants.iter().any(|v| v == "_add_metrics"));
        assert!(variants.iter().any(|v| v == "add metrics"));
    }

    #[test]
    fn bm25_variants_dedupe_case_only_duplicates() {
        let variants = bm25_query_variants("Benchmark Fixture Architecture");
        assert_eq!(variants, vec!["Benchmark Fixture Architecture".to_string()]);
    }

    #[test]
    fn bm25_path_filter_clause_matches_structured_intents() {
        let workflow = QueryScoreContext::new("runs-on: ubuntu-latest");
        let systemd = QueryScoreContext::new("Description=SemanticFS MCP Service");
        let command = QueryScoreContext::new("Run-Step \"Phase 3 domain plan build\"");
        let narrative = QueryScoreContext::new("Benchmark Fixture Architecture");

        assert!(bm25_path_filter_clause(&workflow)
            .unwrap_or_default()
            .contains("workflows"));
        assert!(bm25_path_filter_clause(&systemd)
            .unwrap_or_default()
            .contains(".service"));
        assert!(bm25_path_filter_clause(&command)
            .unwrap_or_default()
            .contains("scripts"));
        assert!(bm25_path_filter_clause(&narrative).is_none());
    }

    #[test]
    fn query_path_overlap_boosts_exact_filename_terms() {
        let cfg = RetrievalConfig {
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
        };
        let allow = vec!["**".to_string()];
        let deny = Vec::new();
        let guard = PolicyGuard::new(&allow, &deny).unwrap();
        let core =
            RetrievalCore::open(Path::new(":memory:"), Path::new("."), cfg, 384, guard).unwrap();

        let query = QueryScoreContext::new("future steps log");
        let future_steps = core.query_path_overlap_multiplier(
            &query,
            &core.path_score_context("docs/future-steps-log.md", None),
        );
        let unrelated = core.query_path_overlap_multiplier(
            &query,
            &core.path_score_context("config/relevance-real.toml", None),
        );

        assert!(future_steps > unrelated);
        assert!(future_steps > 1.0);
        assert!((unrelated - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn file_name_overlap_boosts_matching_doc_title() {
        let cfg = RetrievalConfig {
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
        };
        let allow = vec!["**".to_string()];
        let deny = Vec::new();
        let guard = PolicyGuard::new(&allow, &deny).unwrap();
        let core =
            RetrievalCore::open(Path::new(":memory:"), Path::new("."), cfg, 384, guard).unwrap();

        let query = QueryScoreContext::new("future steps log");
        let future_steps = core.file_name_query_overlap_multiplier(
            &query,
            &core.path_score_context("docs/future-steps-log.md", None),
        );
        let phase3 = core.file_name_query_overlap_multiplier(
            &query,
            &core.path_score_context("docs/phase3_execution_plan.md", None),
        );

        assert!(future_steps > phase3);
        assert!(future_steps > 1.0);
        assert!((phase3 - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn config_query_prior_prefers_config_paths() {
        let cfg = RetrievalConfig {
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
        };
        let allow = vec!["**".to_string()];
        let deny = Vec::new();
        let guard = PolicyGuard::new(&allow, &deny).unwrap();
        let core =
            RetrievalCore::open(Path::new(":memory:"), Path::new("."), cfg, 384, guard).unwrap();

        let query = QueryScoreContext::new("mount_point = \"/mnt/ai\"");
        let config_prior = core.config_query_path_multiplier(
            &query,
            &core.path_score_context("config/relevance-multiroot.toml", None),
        );
        let code_prior = core.config_query_path_multiplier(
            &query,
            &core.path_score_context("code/semanticfs-cli/src/main.rs", None),
        );

        assert!(config_prior > 1.0);
        assert!(code_prior < 1.0);
        assert!(config_prior > code_prior);
    }

    #[test]
    fn ranked_keys_collapse_duplicate_file_paths() {
        let first = sample("code/semanticfs-cli/src/benchmark.rs", 10);
        let second = sample("code/semanticfs-cli/src/benchmark.rs", 20);
        let third = sample("config/relevance-multiroot.toml", 1);
        let ordered = vec![make_key(&first), make_key(&second), make_key(&third)];
        let deduped = dedupe_ranked_keys_by_path(&ordered, &[vec![first, second, third]], 5);

        assert_eq!(deduped.len(), 2);
        assert_eq!(deduped[0], ordered[0]);
        assert_eq!(deduped[1], ordered[2]);
    }

    #[test]
    fn narrative_docs_queries_prefer_docs_over_scripts() {
        let cfg = RetrievalConfig {
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
        };
        let allow = vec!["**".to_string()];
        let deny = Vec::new();
        let guard = PolicyGuard::new(&allow, &deny).unwrap();
        let core =
            RetrievalCore::open(Path::new(":memory:"), Path::new("."), cfg, 384, guard).unwrap();

        let query = QueryScoreContext::new(
            "stabilize multi-root retrieval on representative mixed-domain workloads",
        );
        let docs_prior = core.narrative_docs_query_multiplier(
            &query,
            &core.path_score_context("docs/v1_2_execution_plan.md", None),
        );
        let script_prior = core.narrative_docs_query_multiplier(
            &query,
            &core.path_score_context("scripts/nightly_representative.ps1", None),
        );

        assert!(docs_prior > 1.0);
        assert!(script_prior < 1.0);
        assert!(docs_prior > script_prior);
    }

    #[test]
    fn docs_schema_queries_prefer_docs_over_code_and_config() {
        let cfg = RetrievalConfig {
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
        };
        let allow = vec!["**".to_string()];
        let deny = Vec::new();
        let guard = PolicyGuard::new(&allow, &deny).unwrap();
        let core =
            RetrievalCore::open(Path::new(":memory:"), Path::new("."), cfg, 384, guard).unwrap();

        let query = QueryScoreContext::new(
            "ts actor op target snapshot_id policy_result reason_code latency_ms result_total",
        );
        let docs_prior = core.narrative_docs_query_multiplier(
            &query,
            &core.path_score_context("docs/runbook.md", None),
        );
        let code_prior = core.narrative_docs_query_multiplier(
            &query,
            &core.path_score_context("code/policy-guard/src/lib.rs", None),
        );
        let config_prior = core.narrative_docs_query_multiplier(
            &query,
            &core.path_score_context("config/relevance-multiroot.toml", None),
        );

        assert!(docs_prior > code_prior);
        assert!(docs_prior > config_prior);
        assert!(docs_prior > 1.0);
    }

    #[test]
    fn command_queries_prefer_scripts_over_docs() {
        let cfg = RetrievalConfig {
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
        };
        let allow = vec!["**".to_string()];
        let deny = Vec::new();
        let guard = PolicyGuard::new(&allow, &deny).unwrap();
        let core =
            RetrievalCore::open(Path::new(":memory:"), Path::new("."), cfg, 384, guard).unwrap();

        let query = QueryScoreContext::new("Run-Step \"Phase 3 domain plan build\"");
        let git_query = QueryScoreContext::new("git ls-files failed for");
        let script_prior = core.command_query_path_multiplier(
            &query,
            &core.path_score_context("scripts/daytime_action_items.ps1", None),
        );
        let docs_prior = core.command_query_path_multiplier(
            &query,
            &core.path_score_context("docs/phase3_execution_plan.md", None),
        );

        assert!(script_prior > 1.0);
        assert!(docs_prior < 1.0);
        assert!(script_prior > docs_prior);
        assert!(git_query.is_command_like);
    }

    #[test]
    fn workflow_queries_prefer_workflow_paths() {
        let cfg = RetrievalConfig {
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
        };
        let allow = vec!["**".to_string()];
        let deny = Vec::new();
        let guard = PolicyGuard::new(&allow, &deny).unwrap();
        let core =
            RetrievalCore::open(Path::new(":memory:"), Path::new("."), cfg, 384, guard).unwrap();

        let query = QueryScoreContext::new("runs-on: ubuntu-latest");
        let workflow_prior = core.workflow_query_path_multiplier(
            &query,
            &core.path_score_context("github/workflows/ci.yml", None),
        );
        let docs_prior = core.workflow_query_path_multiplier(
            &query,
            &core.path_score_context("docs/phase3_execution_status.md", None),
        );

        assert!(workflow_prior > 1.0);
        assert!(docs_prior < 1.0);
        assert!(workflow_prior > docs_prior);
    }

    #[test]
    fn systemd_queries_prefer_unit_paths() {
        let cfg = RetrievalConfig {
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
        };
        let allow = vec!["**".to_string()];
        let deny = Vec::new();
        let guard = PolicyGuard::new(&allow, &deny).unwrap();
        let core =
            RetrievalCore::open(Path::new(":memory:"), Path::new("."), cfg, 384, guard).unwrap();

        let query = QueryScoreContext::new("WantedBy=multi-user.target");
        let unit_prior = core.systemd_unit_query_path_multiplier(
            &query,
            &core.path_score_context("systemd/semanticfs-mcp.service", None),
        );
        let docs_prior = core.systemd_unit_query_path_multiplier(
            &query,
            &core.path_score_context("docs/phase3_execution_status.md", None),
        );

        assert!(unit_prior > 1.0);
        assert!(docs_prior < 1.0);
        assert!(unit_prior > docs_prior);
    }

    #[test]
    fn structured_literal_queries_skip_vector_search() {
        let config = QueryScoreContext::new("allow_roots = [\"docs/**\"]");
        let command = QueryScoreContext::new("Run-Step \"Phase 3 domain plan build\"");
        let workflow = QueryScoreContext::new("runs-on: ubuntu-latest");
        let systemd = QueryScoreContext::new("Description=SemanticFS MCP Service");

        assert!(!config.should_run_vector_search(false));
        assert!(!command.should_run_vector_search(false));
        assert!(!workflow.should_run_vector_search(false));
        assert!(!systemd.should_run_vector_search(false));
    }

    #[test]
    fn narrative_queries_keep_vector_search() {
        let query = QueryScoreContext::new(
            "explain how the multi root scheduler enforces domain ownership across runtime paths",
        );

        assert!(query.should_run_vector_search(false));
    }

    #[test]
    fn symbol_queries_skip_vector_search_when_symbol_hits_exist() {
        let query = QueryScoreContext::new("normalize_result_path");

        assert!(!query.should_run_vector_search(true));
        assert!(query.should_run_vector_search(false));
    }

    #[test]
    fn short_narrative_queries_trim_vector_limit_when_bm25_already_has_signal() {
        let cfg = RetrievalConfig {
            rrf_mode: "plain".to_string(),
            rrf_k: 60,
            topn_symbol: 10,
            topn_bm25: 20,
            topn_vector: 12,
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
        };
        let allow = vec!["**".to_string()];
        let deny = Vec::new();
        let guard = PolicyGuard::new(&allow, &deny).unwrap();
        let core =
            RetrievalCore::open(Path::new(":memory:"), Path::new("."), cfg, 384, guard).unwrap();

        let query = QueryScoreContext::new("explain domain ownership in map summaries");

        assert_eq!(
            core.vector_limit_for_query(&query, false, true, true),
            Some(6)
        );
        assert_eq!(
            core.vector_limit_for_query(&query, false, true, false),
            Some(8)
        );
        assert_eq!(
            core.vector_limit_for_query(&query, false, false, false),
            Some(12)
        );
    }

    #[test]
    fn long_narrative_queries_aggressively_trim_vector_when_docs_bm25_is_already_strong() {
        let cfg = RetrievalConfig {
            rrf_mode: "plain".to_string(),
            rrf_k: 60,
            topn_symbol: 10,
            topn_bm25: 20,
            topn_vector: 12,
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
        };
        let allow = vec!["**".to_string()];
        let deny = Vec::new();
        let guard = PolicyGuard::new(&allow, &deny).unwrap();
        let core =
            RetrievalCore::open(Path::new(":memory:"), Path::new("."), cfg, 384, guard).unwrap();

        let query = QueryScoreContext::new(
            "this runbook explains how domain ownership, policy decisions, and latency reporting fit together",
        );

        assert_eq!(core.vector_limit_for_query(&query, false, true, true), None);
        assert_eq!(
            core.vector_limit_for_query(&query, false, true, false),
            Some(8)
        );
    }

    #[test]
    fn docs_schema_queries_trim_vector_budget_without_docs_hint() {
        let cfg = RetrievalConfig {
            rrf_mode: "plain".to_string(),
            rrf_k: 60,
            topn_symbol: 10,
            topn_bm25: 20,
            topn_vector: 12,
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
        };
        let allow = vec!["**".to_string()];
        let deny = Vec::new();
        let guard = PolicyGuard::new(&allow, &deny).unwrap();
        let core =
            RetrievalCore::open(Path::new(":memory:"), Path::new("."), cfg, 384, guard).unwrap();

        let query = QueryScoreContext::new(
            "ts actor op target snapshot_id policy_result reason_code latency_ms result_total",
        );

        assert_eq!(
            core.vector_limit_for_query(&query, false, true, false),
            Some(8)
        );
    }

    #[test]
    fn structured_and_narrative_queries_trim_bm25_budget() {
        let cfg = RetrievalConfig {
            rrf_mode: "plain".to_string(),
            rrf_k: 60,
            topn_symbol: 10,
            topn_bm25: 12,
            topn_vector: 12,
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
        };
        let allow = vec!["**".to_string()];
        let deny = Vec::new();
        let guard = PolicyGuard::new(&allow, &deny).unwrap();
        let core =
            RetrievalCore::open(Path::new(":memory:"), Path::new("."), cfg, 384, guard).unwrap();

        let workflow = QueryScoreContext::new("runs-on: ubuntu-latest");
        let config = QueryScoreContext::new("mount_point = \"/mnt/ai\"");
        let narrative = QueryScoreContext::new(
            "stabilize multi-root retrieval on representative mixed-domain workloads",
        );
        assert_eq!(core.bm25_limit_for_query(&workflow), 6);
        assert_eq!(core.bm25_limit_for_query(&config), 8);
        assert_eq!(core.bm25_limit_for_query(&narrative), 8);
    }

    #[test]
    fn direct_hit_render_uses_exact_symbol_fast_path_marker() {
        let cfg = RetrievalConfig {
            rrf_mode: "plain".to_string(),
            rrf_k: 60,
            topn_symbol: 10,
            topn_bm25: 12,
            topn_vector: 12,
            topn_final: 1,
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
        };
        let allow = vec!["**".to_string()];
        let deny = Vec::new();
        let guard = PolicyGuard::new(&allow, &deny).unwrap();
        let core =
            RetrievalCore::open(Path::new(":memory:"), Path::new("."), cfg, 384, guard).unwrap();
        let query = QueryScoreContext::new("normalize_result_path");

        let rendered = core.render_direct_hits(
            &query,
            vec![
                PartialHit {
                    path: "code/semanticfs-cli/src/benchmark.rs".to_string(),
                    domain_id: "code".to_string(),
                    start_line: 1,
                    end_line: 5,
                    file_hash: "hash-a".to_string(),
                    modified_unix_ms: None,
                    trust_label: "trusted".to_string(),
                    trust_level: TrustLevel::Trusted,
                    source: HitSource::Symbol,
                    symbol_kind: Some("function".to_string()),
                    score_symbol: Some(2.0),
                    score_bm25: None,
                    score_vector: None,
                    why_selected: "exact symbol match".to_string(),
                },
                PartialHit {
                    path: "code/semanticfs-cli/src/benchmark.rs".to_string(),
                    domain_id: "code".to_string(),
                    start_line: 10,
                    end_line: 12,
                    file_hash: "hash-b".to_string(),
                    modified_unix_ms: None,
                    trust_label: "trusted".to_string(),
                    trust_level: TrustLevel::Trusted,
                    source: HitSource::Symbol,
                    symbol_kind: Some("function".to_string()),
                    score_symbol: Some(2.0),
                    score_bm25: None,
                    score_vector: None,
                    why_selected: "exact symbol match".to_string(),
                },
            ],
            10,
            10,
        );

        assert_eq!(rendered.len(), 1);
        assert!(rendered[0].why_selected.contains("exact-symbol-fast-path"));
    }

    #[test]
    fn search_cache_key_changes_with_snapshot_context() {
        let a = search_cache_key("query", 10, 10);
        let b = search_cache_key("query", 11, 10);
        let c = search_cache_key("query", 10, 11);

        assert_ne!(a, b);
        assert_ne!(a, c);
    }

}
