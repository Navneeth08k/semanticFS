use anyhow::Result;
use policy_guard::PolicyGuard;
use rusqlite::{params, Connection};
use semanticfs_common::{
    cosine_similarity, embed_text_hash, GroundedHit, HitSource, RetrievalConfig, TrustLevel,
};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

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
    cfg: RetrievalConfig,
    embed_dim: usize,
    guard: PolicyGuard,
}

impl RetrievalCore {
    pub fn open(
        db_path: &Path,
        cfg: RetrievalConfig,
        embed_dim: usize,
        guard: PolicyGuard,
    ) -> Result<Self> {
        Ok(Self {
            db_path: db_path.to_path_buf(),
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
            rank_lists.push(exact);
        }

        let prefix = self.symbol_prefix(&conn, query, snapshot_version)?;
        if !prefix.is_empty() {
            rank_lists.push(prefix);
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
        let mut out = Vec::new();

        for (idx, (path_key, score)) in fused.into_iter().take(self.cfg.topn_final).enumerate() {
            if let Some(hit) = rank_lists
                .iter()
                .flat_map(|v| v.iter())
                .find(|h| make_key(h) == path_key)
                .cloned()
            {
                out.push(GroundedHit {
                    rank: (idx + 1) as u32,
                    path: hit.path,
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
                    trust_level: TrustLevel::Trusted,
                    why_selected: hit.why_selected,
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

    fn symbol_exact(
        &self,
        conn: &Connection,
        query: &str,
        version: u64,
    ) -> Result<Vec<PartialHit>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT path, line_start, line_end, symbol_kind, file_hash
            FROM symbols
            WHERE index_version=?1 AND symbol_name=?2
            ORDER BY exported DESC
            LIMIT ?3
            "#,
        )?;

        let rows = stmt.query_map(
            params![version, query, self.cfg.topn_symbol as i64],
            |row| {
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
            },
        )?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn symbol_prefix(
        &self,
        conn: &Connection,
        query: &str,
        version: u64,
    ) -> Result<Vec<PartialHit>> {
        let like = format!("{}%", query);
        let mut stmt = conn.prepare(
            r#"
            SELECT path, line_start, line_end, symbol_kind, file_hash
            FROM symbols
            WHERE index_version=?1 AND symbol_name LIKE ?2
            ORDER BY exported DESC
            LIMIT ?3
            "#,
        )?;

        let rows = stmt.query_map(params![version, like, self.cfg.topn_symbol as i64], |row| {
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

        let rows = stmt.query_map(params![query, version, self.cfg.topn_bm25 as i64], |row| {
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
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
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

fn vector_backend_enabled() -> bool {
    std::env::var("SEMANTICFS_VECTOR_BACKEND")
        .map(|v| v.eq_ignore_ascii_case("lancedb"))
        .unwrap_or(false)
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
}
