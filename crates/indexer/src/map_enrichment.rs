use anyhow::Result;

use crate::db::IndexerDb;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnrichmentMode {
    Disabled,
    AsyncOptional,
}

impl EnrichmentMode {
    pub fn from_config(raw: &str) -> Self {
        if raw.eq_ignore_ascii_case("disabled") || raw.eq_ignore_ascii_case("off") {
            EnrichmentMode::Disabled
        } else {
            EnrichmentMode::AsyncOptional
        }
    }
}

pub fn run_enrichment_job(db_path: std::path::PathBuf, version: u64) {
    if let Err(err) = run_enrichment_job_blocking(&db_path, version) {
        tracing::warn!(version, error = %err, "map enrichment worker failed");
    }
}

pub fn run_enrichment_job_blocking(db_path: &std::path::Path, version: u64) -> Result<()> {
    let db = IndexerDb::open(db_path)?;
    let summaries = db.list_map_summaries(version)?;
    if summaries.is_empty() {
        return Ok(());
    }

    for (dir, base_summary) in summaries {
        let enrichment = build_enrichment_for_dir(&db, &dir, &base_summary, version)?;
        db.upsert_map_enrichment(&dir, &enrichment, version, "ready", "heuristic_async_v1")?;
    }

    tracing::info!(version, "map enrichment worker completed");
    Ok(())
}

fn build_enrichment_for_dir(
    db: &IndexerDb,
    dir: &str,
    base_summary: &str,
    version: u64,
) -> Result<String> {
    let types = db.file_type_counts_for_dir(dir, version, 5)?;
    let symbols = db.top_symbols_for_dir(dir, version, 8)?;
    let child_dirs = db.child_directories_for_dir(dir, version, 6)?;
    let trust_labels = db.trust_label_counts_for_dir(dir, version)?;
    let dir_label = if dir == "." { "/" } else { dir };

    let mut out = String::new();
    out.push_str(&format!("Directory: `{}`\n", dir_label));
    out.push_str("This section is generated asynchronously and never blocks `/map` reads.\n\n");

    if !child_dirs.is_empty() {
        out.push_str("Immediate child directories in this indexed subtree:\n");
        for (child, nested_count) in child_dirs {
            out.push_str(&format!(
                "- `{}` ({} indexed descendant summaries)\n",
                child, nested_count
            ));
        }
        out.push('\n');
    }

    if !trust_labels.is_empty() {
        out.push_str("Observed trust labels in this subtree:\n");
        for (label, count) in trust_labels {
            out.push_str(&format!("- `{}`: {} indexed files\n", label, count));
        }
        out.push('\n');
    }

    if !types.is_empty() {
        out.push_str("Likely dominant file types:\n");
        for (file_type, count) in types {
            out.push_str(&format!("- `{}`: {} files\n", file_type, count));
        }
        out.push('\n');
    }

    if !symbols.is_empty() {
        out.push_str("Likely key symbols:\n");
        for (name, kind, count) in symbols {
            out.push_str(&format!(
                "- `{}` ({}) appears {} times\n",
                name, kind, count
            ));
        }
        out.push('\n');
    }

    out.push_str("Guidance:\n");
    out.push_str("- Use `/search/<intent>.md` to locate grounded files/lines.\n");
    out.push_str("- Verify exact bytes via `/raw/<path>` before edits.\n");
    out.push_str("- Traverse `/map/<domain-or-dir>/directory_overview.md` to move one directory level at a time across domains.\n");
    out.push_str("- If results are stale, re-run search on latest snapshot.\n\n");

    let high_level = infer_focus_hint(base_summary);
    out.push_str(&format!("Focus hint: {}\n", high_level));

    Ok(out)
}

fn infer_focus_hint(base_summary: &str) -> &'static str {
    let l = base_summary.to_ascii_lowercase();
    if l.contains("test") {
        "Prioritize understanding test harnesses and fixtures before modifying implementation."
    } else if l.contains("docs") {
        "Prefer map/search first; this area appears documentation-heavy and likely not executable paths."
    } else if l.contains("service") || l.contains("api") {
        "Trace entrypoints and exported symbols before touching shared dependencies."
    } else {
        "Start from top symbols and follow imports/callers depth-first."
    }
}
