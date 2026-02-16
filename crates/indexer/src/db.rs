use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::{chunking::ChunkRecord, map_summary::DirectorySummary, symbols::SymbolRecord};

pub struct IndexerDb {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct VectorRow {
    pub chunk_id: String,
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub file_hash: String,
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
            "#,
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
        version: u64,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO files(path, file_hash, file_type, parse_status, index_version, updated_at)
            VALUES(?1, ?2, ?3, ?4, ?5, datetime('now'))
            ON CONFLICT(path, index_version)
            DO UPDATE SET
                file_hash = excluded.file_hash,
                file_type = excluded.file_type,
                parse_status = excluded.parse_status,
                updated_at = excluded.updated_at
            "#,
            params![path, file_hash, file_type, parse_status, version],
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
        version: u64,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO chunks_meta(
                chunk_id, path, start_line, end_line, language, symbol, content, file_hash, index_version, trust_level, updated_at
            ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'trusted', datetime('now'))
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
                version
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
            SELECT
                COALESCE(substr(path, 1, instr(path || '/', '/') - 1), '.') AS dir,
                COUNT(*) AS file_count
            FROM files
            WHERE index_version=?1 AND parse_status='indexed'
            GROUP BY dir
            ORDER BY file_count DESC
            "#,
        )?;

        let mut rows = stmt.query(params![version])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let dir: String = row.get(0)?;
            let file_count: i64 = row.get(1)?;
            out.push(DirectorySummary {
                dir_path: dir.clone(),
                summary_markdown: format!(
                    "# Directory Overview: `{}`\n\n- Indexed files: {}\n- Snapshot version: {}\n- Summary mode: deterministic_precompute\n",
                    dir,
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

    pub fn fetch_vectors_for_version(&self, version: u64) -> Result<Vec<VectorRow>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                m.chunk_id,
                m.path,
                m.start_line,
                m.end_line,
                m.file_hash,
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
            let embedding_json: String = row.get(5)?;
            let embedding: Vec<f32> = serde_json::from_str(&embedding_json).unwrap_or_default();
            Ok(VectorRow {
                chunk_id: row.get(0)?,
                path: row.get(1)?,
                start_line: row.get::<_, i64>(2)? as u32,
                end_line: row.get::<_, i64>(3)? as u32,
                file_hash: row.get(4)?,
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
}
