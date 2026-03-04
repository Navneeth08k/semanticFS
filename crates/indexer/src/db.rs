use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::collections::BTreeMap;

use crate::{chunking::ChunkRecord, map_summary::DirectorySummary, symbols::SymbolRecord};
use semanticfs_common::IndexingStatus;

pub struct IndexerDb {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct VectorRow {
    pub chunk_id: String,
    pub path: String,
    pub domain_id: String,
    pub start_line: u32,
    pub end_line: u32,
    pub file_hash: String,
    pub trust_label: String,
    pub embedding: Vec<f32>,
}

impl IndexerDb {
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let conn =
            Connection::open(path).with_context(|| format!("open sqlite db {}", path.display()))?;
        Ok(Self { conn })
    }

    pub fn ensure_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA journal_mode=WAL;

            CREATE TABLE IF NOT EXISTS index_versions (
                version INTEGER PRIMARY KEY,
                state TEXT NOT NULL,
                created_at TEXT NOT NULL,
                published_at TEXT
            );

            CREATE TABLE IF NOT EXISTS files (
                path TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                file_type TEXT NOT NULL,
                parse_status TEXT NOT NULL,
                domain_id TEXT NOT NULL DEFAULT 'default',
                trust_label TEXT NOT NULL DEFAULT 'trusted',
                modified_unix_ms INTEGER NOT NULL DEFAULT 0,
                index_version INTEGER NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY(path, index_version)
            );

            CREATE TABLE IF NOT EXISTS chunks_meta (
                chunk_id TEXT NOT NULL,
                path TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                language TEXT NOT NULL,
                symbol TEXT,
                content TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                index_version INTEGER NOT NULL,
                trust_level TEXT NOT NULL,
                domain_id TEXT NOT NULL DEFAULT 'default',
                trust_label TEXT NOT NULL DEFAULT 'trusted',
                updated_at TEXT NOT NULL,
                PRIMARY KEY(chunk_id, path, index_version)
            );

            CREATE TABLE IF NOT EXISTS chunks_vec (
                chunk_id TEXT NOT NULL,
                path TEXT NOT NULL,
                embedding_json TEXT NOT NULL,
                index_version INTEGER NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY(chunk_id, path, index_version)
            );

            CREATE INDEX IF NOT EXISTS idx_chunks_vec_version_path
            ON chunks_vec(index_version, path);

            CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
                chunk_id,
                path,
                content,
                tokenize = 'unicode61'
            );

            CREATE TABLE IF NOT EXISTS symbols (
                symbol_id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol_name TEXT NOT NULL,
                symbol_kind TEXT NOT NULL,
                path TEXT NOT NULL,
                line_start INTEGER NOT NULL,
                line_end INTEGER NOT NULL,
                language TEXT NOT NULL,
                exported INTEGER NOT NULL,
                scope TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                index_version INTEGER NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_symbols_name_version
            ON symbols(symbol_name, index_version);

            CREATE INDEX IF NOT EXISTS idx_symbols_path_version
            ON symbols(path, index_version);

            CREATE TABLE IF NOT EXISTS map_summaries (
                dir_path TEXT NOT NULL,
                summary_markdown TEXT NOT NULL,
                index_version INTEGER NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY(dir_path, index_version)
            );

            CREATE TABLE IF NOT EXISTS map_enrichments (
                dir_path TEXT NOT NULL,
                enrichment_markdown TEXT NOT NULL,
                index_version INTEGER NOT NULL,
                status TEXT NOT NULL,
                model_info TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY(dir_path, index_version)
            );

            CREATE INDEX IF NOT EXISTS idx_map_enrichments_version_status
            ON map_enrichments(index_version, status);

            CREATE TABLE IF NOT EXISTS runtime_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )?;
        self.ensure_column("files", "domain_id", "TEXT NOT NULL DEFAULT 'default'")?;
        self.ensure_column("files", "trust_label", "TEXT NOT NULL DEFAULT 'trusted'")?;
        self.ensure_column("files", "modified_unix_ms", "INTEGER NOT NULL DEFAULT 0")?;
        self.ensure_column(
            "chunks_meta",
            "domain_id",
            "TEXT NOT NULL DEFAULT 'default'",
        )?;
        self.ensure_column(
            "chunks_meta",
            "trust_label",
            "TEXT NOT NULL DEFAULT 'trusted'",
        )?;
        Ok(())
    }

    pub fn create_staging_version(&self) -> Result<u64> {
        let next = self.next_version()?;
        self.conn.execute(
            "INSERT INTO index_versions(version, state, created_at) VALUES(?1, 'staging', datetime('now'))",
            params![next],
        )?;
        Ok(next)
    }

    pub fn publish_staging_version(&self, version: u64) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        tx.execute(
            "UPDATE index_versions SET state='obsolete' WHERE state='active'",
            [],
        )?;
        tx.execute(
            "UPDATE index_versions SET state='active', published_at=datetime('now') WHERE version=?1",
            params![version],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn active_version(&self) -> Result<u64> {
        let mut stmt = self.conn.prepare(
            "SELECT version FROM index_versions WHERE state='active' ORDER BY version DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            return Ok(row.get(0)?);
        }
        Ok(0)
    }

    pub fn upsert_file_record(
        &self,
        path: &str,
        file_hash: &str,
        file_type: &str,
        parse_status: &str,
        domain_id: &str,
        trust_label: &str,
        modified_unix_ms: i64,
        version: u64,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO files(path, file_hash, file_type, parse_status, domain_id, trust_label, modified_unix_ms, index_version, updated_at)
            VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))
            ON CONFLICT(path, index_version)
            DO UPDATE SET
                file_hash = excluded.file_hash,
                file_type = excluded.file_type,
                parse_status = excluded.parse_status,
                domain_id = excluded.domain_id,
                trust_label = excluded.trust_label,
                modified_unix_ms = excluded.modified_unix_ms,
                updated_at = excluded.updated_at
            "#,
            params![
                path,
                file_hash,
                file_type,
                parse_status,
                domain_id,
                trust_label,
                modified_unix_ms,
                version
            ],
        )?;
        Ok(())
    }

    pub fn delete_chunks_for_path(&self, path: &str, version: u64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM chunks_meta WHERE path=?1 AND index_version=?2",
            params![path, version],
        )?;
        self.conn
            .execute("DELETE FROM chunks_fts WHERE path=?1", params![path])?;
        Ok(())
    }

    pub fn delete_vectors_for_path(&self, path: &str, version: u64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM chunks_vec WHERE path=?1 AND index_version=?2",
            params![path, version],
        )?;
        Ok(())
    }

    pub fn delete_symbols_for_path(&self, path: &str, version: u64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM symbols WHERE path=?1 AND index_version=?2",
            params![path, version],
        )?;
        Ok(())
    }

    pub fn upsert_chunk(
        &self,
        chunk: &ChunkRecord,
        path: &str,
        file_hash: &str,
        domain_id: &str,
        trust_label: &str,
        version: u64,
    ) -> Result<()> {
        let trust_level = normalized_trust_level(trust_label);
        self.conn.execute(
            r#"
            INSERT INTO chunks_meta(
                chunk_id, path, start_line, end_line, language, symbol, content, file_hash, index_version, trust_level, domain_id, trust_label, updated_at
            ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now'))
            "#,
            params![
                chunk.chunk_id,
                path,
                chunk.start_line,
                chunk.end_line,
                chunk.language,
                chunk.symbol,
                chunk.content,
                file_hash,
                version,
                trust_level,
                domain_id,
                trust_label
            ],
        )?;

        self.conn.execute(
            "INSERT INTO chunks_fts(chunk_id, path, content) VALUES(?1, ?2, ?3)",
            params![chunk.chunk_id, path, chunk.content],
        )?;

        Ok(())
    }

    pub fn upsert_vector(
        &self,
        chunk_id: &str,
        path: &str,
        embedding: &[f32],
        version: u64,
    ) -> Result<()> {
        let embedding_json = serde_json::to_string(embedding)?;
        self.conn.execute(
            r#"
            INSERT INTO chunks_vec(chunk_id, path, embedding_json, index_version, updated_at)
            VALUES(?1, ?2, ?3, ?4, datetime('now'))
            ON CONFLICT(chunk_id, path, index_version)
            DO UPDATE SET embedding_json=excluded.embedding_json, updated_at=excluded.updated_at
            "#,
            params![chunk_id, path, embedding_json, version],
        )?;
        Ok(())
    }

    pub fn upsert_symbol(
        &self,
        symbol: &SymbolRecord,
        file_hash: &str,
        version: u64,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO symbols(
                symbol_name, symbol_kind, path, line_start, line_end, language, exported, scope, file_hash, index_version, updated_at
            ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))
            "#,
            params![
                symbol.symbol_name,
                symbol.symbol_kind,
                symbol.path,
                symbol.line_start,
                symbol.line_end,
                symbol.language,
                if symbol.exported { 1 } else { 0 },
                symbol.scope,
                file_hash,
                version
            ],
        )?;
        Ok(())
    }

    pub fn compute_directory_summaries(&self, version: u64) -> Result<Vec<DirectorySummary>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT path
            FROM files
            WHERE index_version=?1 AND parse_status='indexed'
            ORDER BY path
            "#,
        )?;

        let rows = stmt.query_map(params![version], |row| row.get::<_, String>(0))?;
        let mut counts = BTreeMap::<String, u32>::new();
        for path in rows.filter_map(|r| r.ok()) {
            for dir in directory_ancestors_for_path(&path) {
                *counts.entry(dir).or_insert(0) += 1;
            }
        }

        let mut out = Vec::with_capacity(counts.len());
        for (dir, file_count) in counts {
            let label = if dir == "." {
                "/".to_string()
            } else {
                dir.clone()
            };
            out.push(DirectorySummary {
                dir_path: dir,
                summary_markdown: format!(
                    "# Directory Overview: `{}`\n\n- Indexed files: {}\n- Snapshot version: {}\n- Summary mode: deterministic_precompute\n",
                    label,
                    file_count,
                    version
                ),
            });
        }
        Ok(out)
    }

    pub fn upsert_map_summary(&self, summary: &DirectorySummary, version: u64) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO map_summaries(dir_path, summary_markdown, index_version, updated_at)
            VALUES(?1, ?2, ?3, datetime('now'))
            ON CONFLICT(dir_path, index_version)
            DO UPDATE SET summary_markdown=excluded.summary_markdown, updated_at=excluded.updated_at
            "#,
            params![summary.dir_path, summary.summary_markdown, version],
        )?;
        Ok(())
    }

    pub fn upsert_map_enrichment(
        &self,
        dir_path: &str,
        enrichment_markdown: &str,
        version: u64,
        status: &str,
        model_info: &str,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO map_enrichments(dir_path, enrichment_markdown, index_version, status, model_info, updated_at)
            VALUES(?1, ?2, ?3, ?4, ?5, datetime('now'))
            ON CONFLICT(dir_path, index_version)
            DO UPDATE SET
                enrichment_markdown=excluded.enrichment_markdown,
                status=excluded.status,
                model_info=excluded.model_info,
                updated_at=excluded.updated_at
            "#,
            params![dir_path, enrichment_markdown, version, status, model_info],
        )?;
        Ok(())
    }

    pub fn list_map_summaries(&self, version: u64) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT dir_path, summary_markdown
            FROM map_summaries
            WHERE index_version=?1
            ORDER BY dir_path
            "#,
        )?;
        let rows = stmt.query_map(params![version], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn file_type_counts_for_dir(
        &self,
        dir: &str,
        version: u64,
        limit: usize,
    ) -> Result<Vec<(String, u32)>> {
        let normalized = normalize_dir_key(dir);
        let (exact, prefix) = dir_match_patterns(&normalized);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT file_type, COUNT(*) AS c
            FROM files
            WHERE index_version=?1
              AND parse_status='indexed'
              AND (?2 = '.' OR path=?2 OR path LIKE ?3)
            GROUP BY file_type
            ORDER BY c DESC
            LIMIT ?4
            "#,
        )?;

        let rows = stmt.query_map(params![version, exact, prefix, limit as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u32))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn top_symbols_for_dir(
        &self,
        dir: &str,
        version: u64,
        limit: usize,
    ) -> Result<Vec<(String, String, u32)>> {
        let normalized = normalize_dir_key(dir);
        let (exact, prefix) = dir_match_patterns(&normalized);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT symbol_name, symbol_kind, COUNT(*) AS c
            FROM symbols
            WHERE index_version=?1
              AND (?2 = '.' OR path=?2 OR path LIKE ?3)
            GROUP BY symbol_name, symbol_kind
            ORDER BY c DESC, symbol_name ASC
            LIMIT ?4
            "#,
        )?;

        let rows = stmt.query_map(params![version, exact, prefix, limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)? as u32,
            ))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn child_directories_for_dir(
        &self,
        dir: &str,
        version: u64,
        limit: usize,
    ) -> Result<Vec<(String, u32)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT dir_path
            FROM map_summaries
            WHERE index_version=?1
            ORDER BY dir_path
            "#,
        )?;
        let rows = stmt.query_map(params![version], |row| row.get::<_, String>(0))?;

        let parent = normalize_dir_key(dir);
        let mut counts = BTreeMap::<String, u32>::new();
        for path in rows.filter_map(|r| r.ok()) {
            if path == "." || path == parent {
                continue;
            }

            let child = if parent == "." {
                path.split('/')
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string()
            } else {
                let prefix = format!("{parent}/");
                let Some(rest) = path.strip_prefix(&prefix) else {
                    continue;
                };
                rest.split('/')
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string()
            };

            if child.is_empty() {
                continue;
            }
            *counts.entry(child).or_insert(0) += 1;
        }

        let mut out = counts.into_iter().collect::<Vec<_>>();
        out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        out.truncate(limit);
        Ok(out)
    }

    pub fn trust_label_counts_for_dir(
        &self,
        dir: &str,
        version: u64,
    ) -> Result<Vec<(String, u32)>> {
        let normalized = normalize_dir_key(dir);
        let (exact, prefix) = dir_match_patterns(&normalized);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT trust_label, COUNT(*) AS c
            FROM files
            WHERE index_version=?1
              AND parse_status='indexed'
              AND (?2 = '.' OR path=?2 OR path LIKE ?3)
            GROUP BY trust_label
            ORDER BY c DESC, trust_label ASC
            "#,
        )?;

        let rows = stmt.query_map(params![version, exact, prefix], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u32))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn fetch_vectors_for_version(&self, version: u64) -> Result<Vec<VectorRow>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                m.chunk_id,
                m.path,
                m.domain_id,
                m.start_line,
                m.end_line,
                m.file_hash,
                m.trust_label,
                v.embedding_json
            FROM chunks_vec v
            JOIN chunks_meta m
              ON m.chunk_id = v.chunk_id
             AND m.path = v.path
             AND m.index_version = v.index_version
            WHERE v.index_version = ?1
            "#,
        )?;

        let rows = stmt.query_map(params![version], |row| {
            let embedding_json: String = row.get(7)?;
            let embedding: Vec<f32> = serde_json::from_str(&embedding_json).unwrap_or_default();
            Ok(VectorRow {
                chunk_id: row.get(0)?,
                path: row.get(1)?,
                domain_id: row.get(2)?,
                start_line: row.get::<_, i64>(3)? as u32,
                end_line: row.get::<_, i64>(4)? as u32,
                file_hash: row.get(5)?,
                trust_label: row.get(6)?,
                embedding,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn next_version(&self) -> Result<u64> {
        let mut stmt = self
            .conn
            .prepare("SELECT COALESCE(MAX(version), 0) + 1 FROM index_versions")?;
        let next: u64 = stmt.query_row([], |row| row.get(0))?;
        Ok(next)
    }

    pub fn set_runtime_state(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO runtime_state(key, value, updated_at)
            VALUES(?1, ?2, datetime('now'))
            ON CONFLICT(key)
            DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at
            "#,
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_runtime_state(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM runtime_state WHERE key=?1 LIMIT 1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            let value: String = row.get(0)?;
            return Ok(Some(value));
        }
        Ok(None)
    }

    pub fn clear_runtime_state(&self, key: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM runtime_state WHERE key=?1", params![key])?;
        Ok(())
    }

    pub fn set_indexing_status(&self, status: &IndexingStatus) -> Result<()> {
        let value = serde_json::to_string(status)?;
        self.set_runtime_state("indexing_status", &value)
    }

    pub fn clear_indexing_status(&self) -> Result<()> {
        self.clear_runtime_state("indexing_status")
    }

    fn ensure_column(&self, table: &str, column: &str, definition: &str) -> Result<()> {
        let pragma = format!("PRAGMA table_info({table})");
        let mut stmt = self.conn.prepare(&pragma)?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        let has_column = rows.filter_map(|r| r.ok()).any(|name| name == column);
        if has_column {
            return Ok(());
        }

        let alter = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
        self.conn.execute(&alter, [])?;
        Ok(())
    }
}

fn normalized_trust_level(trust_label: &str) -> &'static str {
    if trust_label.eq_ignore_ascii_case("untrusted") {
        "untrusted"
    } else {
        "trusted"
    }
}

fn directory_ancestors_for_path(path: &str) -> Vec<String> {
    let normalized = path.trim_matches('/');
    if normalized.is_empty() {
        return vec![".".to_string()];
    }

    let parts = normalized.split('/').collect::<Vec<_>>();
    let dir_parts = if parts.len() > 1 {
        &parts[..parts.len() - 1]
    } else {
        &[][..]
    };

    let mut out = vec![".".to_string()];
    let mut current = String::new();
    for part in dir_parts {
        if current.is_empty() {
            current.push_str(part);
        } else {
            current.push('/');
            current.push_str(part);
        }
        out.push(current.clone());
    }

    out
}

fn normalize_dir_key(dir: &str) -> String {
    let trimmed = dir.trim_matches('/');
    if trimmed.is_empty() {
        ".".to_string()
    } else {
        trimmed.to_string()
    }
}

fn dir_match_patterns(dir: &str) -> (String, String) {
    if dir == "." {
        (".".to_string(), "%".to_string())
    } else {
        (dir.to_string(), format!("{dir}/%"))
    }
}
