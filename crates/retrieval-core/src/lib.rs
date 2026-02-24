use anyhow::Result;
use policy_guard::PolicyGuard;
use rusqlite::{params, params_from_iter, types::Value, Connection};
use semanticfs_common::{
    cosine_similarity, embed_text_hash, GroundedHit, HitSource, IndexingStatus, RetrievalConfig,
    TrustLevel,
};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[cfg(feature = "lancedb")]
use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct PartialHit {
    path: String,
    start_line: u32,
    end_line: u32,
    file_hash: String,
    source: HitSource,
    symbol_kind: Option<String>,
    score_symbol: Option<f32>,
    score_bm25: Option<f32>,
    score_vector: Option<f32>,
    why_selected: String,
}

pub struct RetrievalCore {
    db_path: PathBuf,
    repo_root: PathBuf,
    cfg: RetrievalConfig,
    embed_dim: usize,
    guard: PolicyGuard,
}

impl RetrievalCore {
    pub fn open(
        db_path: &Path,
        repo_root: &Path,
        cfg: RetrievalConfig,
        embed_dim: usize,
        guard: PolicyGuard,
    ) -> Result<Self> {
        Ok(Self {
            db_path: db_path.to_path_buf(),
            repo_root: repo_root.to_path_buf(),
            cfg,
            embed_dim: embed_dim.max(1),
            guard,
        })
    }

    pub fn search(
        &self,
        query: &str,
        snapshot_version: u64,
        active_version: u64,
    ) -> Result<Vec<GroundedHit>> {
        let conn = Connection::open(&self.db_path)?;
        let mut rank_lists: Vec<Vec<PartialHit>> = Vec::new();

        let exact = self.symbol_exact(&conn, query, snapshot_version)?;
        if !exact.is_empty() {
            rank_lists.push(exact.clone());
        }

        let prefix = self.symbol_prefix(&conn, query, snapshot_version)?;
        if !prefix.is_empty() {
            rank_lists.push(prefix.clone());
        }

        let bm25 = self.bm25(&conn, query, snapshot_version)?;
        if !bm25.is_empty() {
            rank_lists.push(bm25);
        }

        let vector = self.vector_search(&conn, query, snapshot_version)?;
        if !vector.is_empty() {
            rank_lists.push(vector);
        }

        let fused = rrf_fuse(&rank_lists, self.cfg.rrf_k as f32);
        let mut adjusted_fused = fused
            .iter()
            .map(|(path_key, base_score)| {
                let prior = rank_lists
                    .iter()
                    .flat_map(|v| v.iter())
                    .find(|h| make_key(h) == *path_key)
                    .map(|h| self.score_prior(&h.path))
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
            if ordered_keys.len() >= self.cfg.topn_final {
                break;
            }
        }

        let fused_scores: HashMap<String, f32> = adjusted_fused.into_iter().collect();
        let mut out = Vec::new();
        for (idx, path_key) in ordered_keys
            .into_iter()
            .take(self.cfg.topn_final)
            .enumerate()
        {
            if let Some(hit) = rank_lists
                .iter()
                .flat_map(|v| v.iter())
                .find(|h| make_key(h) == path_key)
                .cloned()
            {
                let path_prior = self.path_prior_multiplier(&hit.path);
                let recency_prior = self.recency_prior_multiplier(&hit.path);
                out.push(GroundedHit {
                    rank: (idx + 1) as u32,
                    path: hit.path,
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
                    trust_level: TrustLevel::Trusted,
                    why_selected: format!(
                        "{}; prior(path={:.2}, recency={:.2})",
                        hit.why_selected, path_prior, recency_prior
                    ),
                });
            }
        }

        Ok(self.guard.redact_sensitive_hits(out))
    }

    pub fn active_version(&self) -> Result<u64> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT version FROM index_versions WHERE state='active' ORDER BY version DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            return Ok(row.get(0)?);
        }
        Ok(0)
    }

    pub fn indexing_status(&self) -> Result<Option<IndexingStatus>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt =
            conn.prepare("SELECT value FROM runtime_state WHERE key='indexing_status' LIMIT 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let raw: String = row.get(0)?;
            let parsed = serde_json::from_str::<IndexingStatus>(&raw).ok();
            return Ok(parsed);
        }
        Ok(None)
    }

    fn symbol_exact(
        &self,
        conn: &Connection,
        query: &str,
        version: u64,
    ) -> Result<Vec<PartialHit>> {
        let variants = lower_dedup_variants(symbol_query_variants(query));
        if variants.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = vec!["?"; variants.len()].join(", ");
        let sql = format!(
            r#"
            SELECT path, line_start, line_end, symbol_kind, file_hash
            FROM symbols
            WHERE index_version=? AND LOWER(symbol_name) IN ({placeholders})
            ORDER BY exported DESC
            LIMIT ?
            "#
        );
        let mut stmt = conn.prepare(&sql)?;

        let mut bind = Vec::with_capacity(variants.len() + 2);
        bind.push(Value::Integer(version as i64));
        for variant in variants {
            bind.push(Value::Text(variant));
        }
        bind.push(Value::Integer(self.cfg.topn_symbol as i64));

        let rows = stmt.query_map(params_from_iter(bind), |row| {
            Ok(PartialHit {
                path: row.get(0)?,
                start_line: row.get::<_, i64>(1)? as u32,
                end_line: row.get::<_, i64>(2)? as u32,
                symbol_kind: Some(row.get(3)?),
                file_hash: row.get(4)?,
                source: HitSource::Symbol,
                score_symbol: Some(self.cfg.symbol_exact_boost),
                score_bm25: None,
                score_vector: None,
                why_selected: "exact symbol match".to_string(),
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn symbol_prefix(
        &self,
        conn: &Connection,
        query: &str,
        version: u64,
    ) -> Result<Vec<PartialHit>> {
        let variants = lower_dedup_variants(symbol_query_variants(query));
        if variants.is_empty() {
            return Ok(Vec::new());
        }

        let like_clauses = vec!["LOWER(symbol_name) LIKE ?"; variants.len()].join(" OR ");
        let sql = format!(
            r#"
            SELECT path, line_start, line_end, symbol_kind, file_hash
            FROM symbols
            WHERE index_version=? AND ({like_clauses})
            ORDER BY exported DESC
            LIMIT ?
            "#
        );
        let mut stmt = conn.prepare(&sql)?;

        let mut bind = Vec::with_capacity(variants.len() + 2);
        bind.push(Value::Integer(version as i64));
        for variant in variants {
            bind.push(Value::Text(format!("{variant}%")));
        }
        bind.push(Value::Integer(self.cfg.topn_symbol as i64));

        let rows = stmt.query_map(params_from_iter(bind), |row| {
            Ok(PartialHit {
                path: row.get(0)?,
                start_line: row.get::<_, i64>(1)? as u32,
                end_line: row.get::<_, i64>(2)? as u32,
                symbol_kind: Some(row.get(3)?),
                file_hash: row.get(4)?,
                source: HitSource::Symbol,
                score_symbol: Some(self.cfg.symbol_prefix_boost),
                score_bm25: None,
                score_vector: None,
                why_selected: "prefix symbol match".to_string(),
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn bm25(&self, conn: &Connection, query: &str, version: u64) -> Result<Vec<PartialHit>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT m.path, m.start_line, m.end_line, m.file_hash
            FROM chunks_fts f
            JOIN chunks_meta m ON m.chunk_id = f.chunk_id AND m.path = f.path
            WHERE f.content MATCH ?1 AND m.index_version=?2
            LIMIT ?3
            "#,
        )?;

        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let per_variant_limit = (self.cfg.topn_bm25 as i64).max(1);

        for variant in bm25_query_variants(query) {
            if out.len() >= self.cfg.topn_bm25 {
                break;
            }

            let rows = match stmt.query_map(params![variant, version, per_variant_limit], |row| {
                Ok(PartialHit {
                    path: row.get(0)?,
                    start_line: row.get::<_, i64>(1)? as u32,
                    end_line: row.get::<_, i64>(2)? as u32,
                    file_hash: row.get(3)?,
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
                if out.len() >= self.cfg.topn_bm25 {
                    break;
                }
            }
        }

        Ok(out)
    }

    fn vector_search(
        &self,
        conn: &Connection,
        query: &str,
        version: u64,
    ) -> Result<Vec<PartialHit>> {
        let query_embedding = embed_text_hash(query, self.embed_dim);

        #[cfg(feature = "lancedb")]
        if vector_backend_enabled() {
            if let Ok(hits) = self.vector_search_lancedb(&query_embedding, version) {
                if !hits.is_empty() {
                    return Ok(hits);
                }
            }
        }

        let candidate_pool = (self.cfg.topn_vector.max(1) * 50).max(500);

        let mut stmt = conn.prepare(
            r#"
            SELECT m.path, m.start_line, m.end_line, m.file_hash, v.embedding_json
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
            let embedding_json: String = row.get(4)?;
            let embedding: Vec<f32> = serde_json::from_str(&embedding_json).unwrap_or_default();
            let score = cosine_similarity(&query_embedding, &embedding);
            Ok(PartialHit {
                path: row.get(0)?,
                start_line: row.get::<_, i64>(1)? as u32,
                end_line: row.get::<_, i64>(2)? as u32,
                file_hash: row.get(3)?,
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
        hits.truncate(self.cfg.topn_vector);
        Ok(hits)
    }

    fn score_prior(&self, path: &str) -> f32 {
        self.path_prior_multiplier(path) * self.recency_prior_multiplier(path)
    }

    fn path_prior_multiplier(&self, path: &str) -> f32 {
        let lower = path.to_ascii_lowercase();
        let mut mult = 1.0f32;

        if is_code_path(&lower) {
            mult *= self.cfg.code_path_boost.max(0.1);
        }
        if is_docs_path(&lower) {
            mult *= self.cfg.docs_path_penalty.max(0.1);
        }
        if is_test_path(&lower) {
            mult *= self.cfg.test_path_penalty.max(0.1);
        }
        if is_generated_artifact_path(&lower) {
            // Keep generated artifacts searchable, but prevent them from shadowing source paths.
            mult *= 0.30;
        }
        mult
    }

    fn recency_prior_multiplier(&self, path: &str) -> f32 {
        let half_life = self.cfg.recency_half_life_hours.max(0.1);
        let min_boost = self.cfg.recency_min_boost.max(0.1);
        let max_boost = self.cfg.recency_max_boost.max(min_boost);

        let mut full_path = self.repo_root.clone();
        full_path.push(path);
        let Ok(meta) = std::fs::metadata(&full_path) else {
            return 1.0;
        };
        let Ok(modified) = meta.modified() else {
            return 1.0;
        };
        let Ok(age) = SystemTime::now().duration_since(modified) else {
            return max_boost;
        };
        let age_hours = age.as_secs_f32() / 3600.0;
        let decay = 0.5f32.powf(age_hours / half_life);
        min_boost + (max_boost - min_boost) * decay.clamp(0.0, 1.0)
    }

    #[cfg(feature = "lancedb")]
    fn vector_search_lancedb(
        &self,
        query_embedding: &[f32],
        version: u64,
    ) -> Result<Vec<PartialHit>> {
        use arrow_array::{Int32Array, RecordBatch, StringArray};
        use futures_util::TryStreamExt;
        use lancedb::connect;
        use lancedb::query::{ExecutableQuery, QueryBase};

        let uri = std::env::var("SEMANTICFS_LANCEDB_URI")
            .unwrap_or_else(|_| "./.semanticfs/lancedb".to_string());
        let table_name = format!("chunks_v{}", version);
        let topn = self.cfg.topn_vector.max(1);

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
                hits.push(PartialHit {
                    path: path_col.value(i).to_string(),
                    start_line: start_col.value(i) as u32,
                    end_line: end_col.value(i) as u32,
                    file_hash: hash_col.value(i).to_string(),
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

    variants
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(path: &str, line: u32) -> PartialHit {
        PartialHit {
            path: path.to_string(),
            start_line: line,
            end_line: line + 1,
            file_hash: "abc".to_string(),
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
        assert!(is_generated_artifact_path("client/.next/dev/server/chunks/ssr/app_page.js"));
        assert!(is_generated_artifact_path("web/dist/assets/main.js"));
        assert!(!is_generated_artifact_path("client/app/page.tsx"));
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
}
