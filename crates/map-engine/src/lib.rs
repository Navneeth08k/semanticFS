use anyhow::Result;
use rusqlite::{params, Connection};
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
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT summary_markdown FROM map_summaries WHERE dir_path=?1 AND index_version=?2",
        )?;

        let mut rows = stmt.query(params![dir, snapshot_version])?;
        if let Some(row) = rows.next()? {
            let mut base: String = row.get(0)?;
            if let Some(enriched) = self.get_enrichment(dir, snapshot_version)? {
                base.push_str("\n\n---\n\n## Optional Enrichment\n");
                base.push_str(&enriched);
            }
            return Ok(Some(base));
        }

        Ok(None)
    }

    fn get_enrichment(&self, _dir: &str, _snapshot_version: u64) -> Result<Option<String>> {
        Ok(None)
    }
}
