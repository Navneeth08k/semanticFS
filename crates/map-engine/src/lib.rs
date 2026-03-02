use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub struct MapEngine {
    db_path: PathBuf,
}

impl MapEngine {
    pub fn open(db_path: &Path) -> Result<Self> {
        Ok(Self {
            db_path: db_path.to_path_buf(),
        })
    }

    pub fn get_directory_overview(
        &self,
        dir: &str,
        snapshot_version: u64,
    ) -> Result<Option<String>> {
        let dir_key = normalize_dir_key(dir);
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT summary_markdown FROM map_summaries WHERE dir_path=?1 AND index_version=?2",
        )?;

        let mut rows = stmt.query(params![&dir_key, snapshot_version])?;
        if let Some(row) = rows.next()? {
            let mut base: String = row.get(0)?;
            if let Some(enriched) = self.get_enrichment(&dir_key, snapshot_version)? {
                base.push_str("\n\n---\n\n## Optional Enrichment\n");
                base.push_str(&enriched);
            }
            return Ok(Some(base));
        }

        Ok(None)
    }

    pub fn has_directory_overview(&self, dir: &str, snapshot_version: u64) -> Result<bool> {
        Ok(self
            .get_directory_overview(dir, snapshot_version)?
            .is_some())
    }

    pub fn list_child_dirs(&self, dir: &str, snapshot_version: u64) -> Result<Vec<String>> {
        let parent = normalize_dir_key(dir);
        let paths = self.list_directory_paths(snapshot_version)?;
        let mut children = BTreeSet::new();

        for path in paths {
            if path == "." {
                continue;
            }

            if parent == "." {
                if let Some(child) = path.split('/').next() {
                    if !child.is_empty() {
                        children.insert(child.to_string());
                    }
                }
                continue;
            }

            if path == parent {
                continue;
            }

            let prefix = format!("{parent}/");
            let Some(rest) = path.strip_prefix(&prefix) else {
                continue;
            };
            let child = rest.split('/').next().unwrap_or_default().trim();
            if !child.is_empty() {
                children.insert(child.to_string());
            }
        }

        Ok(children.into_iter().collect())
    }

    pub fn directory_exists(&self, dir: &str, snapshot_version: u64) -> Result<bool> {
        let dir_key = normalize_dir_key(dir);
        if dir_key == "." {
            return Ok(true);
        }
        if self.has_directory_overview(&dir_key, snapshot_version)? {
            return Ok(true);
        }
        Ok(!self.list_child_dirs(&dir_key, snapshot_version)?.is_empty())
    }

    fn get_enrichment(&self, dir: &str, snapshot_version: u64) -> Result<Option<String>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT enrichment_markdown FROM map_enrichments WHERE dir_path=?1 AND index_version=?2 AND status='ready'",
        )?;

        let mut rows = stmt.query(params![dir, snapshot_version])?;
        if let Some(row) = rows.next()? {
            let md: String = row.get(0)?;
            return Ok(Some(md));
        }

        Ok(None)
    }

    fn list_directory_paths(&self, snapshot_version: u64) -> Result<Vec<String>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT dir_path FROM map_summaries WHERE index_version=?1 ORDER BY dir_path",
        )?;
        let rows = stmt.query_map(params![snapshot_version], |row| row.get::<_, String>(0))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

fn normalize_dir_key(dir: &str) -> String {
    let trimmed = dir.trim_matches('/');
    if trimmed.is_empty() {
        ".".to_string()
    } else {
        trimmed.to_string()
    }
}
