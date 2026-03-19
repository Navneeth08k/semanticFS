//! `semanticfs doctor` — diagnose the local setup.
//!
//! Checks config validity, index state, embedding backend, HTTP server
//! reachability, and policy guard health. Prints `[OK]`, `[WARN]`, or
//! `[FAIL]` for each check and exits non-zero if any check is `[FAIL]`.

use anyhow::Result;
use indexer::db::IndexerDb;
use policy_guard::PolicyGuard;
use semanticfs_common::SemanticFsConfig;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

pub struct DoctorOptions {
    pub config_path: PathBuf,
    pub db_path: PathBuf,
    pub http_check: bool,
}

enum Status {
    Ok,
    Warn,
    Fail,
}

impl Status {
    fn label(&self) -> &'static str {
        match self {
            Status::Ok => "[OK]  ",
            Status::Warn => "[WARN]",
            Status::Fail => "[FAIL]",
        }
    }
    fn is_fail(&self) -> bool {
        matches!(self, Status::Fail)
    }
}

fn print_check(status: Status, message: &str) -> bool {
    println!("{}  {}", status.label(), message);
    status.is_fail()
}

pub fn run(opts: DoctorOptions) -> Result<()> {
    println!("SemanticFS Doctor");
    println!("{}", "-".repeat(60));

    let mut any_fail = false;

    // ── 1. Config file ──────────────────────────────────────────────────────
    let cfg_result = SemanticFsConfig::load(&opts.config_path);
    let cfg = match cfg_result {
        Ok(cfg) => {
            any_fail |= print_check(
                Status::Ok,
                &format!("Config: {}", opts.config_path.display()),
            );
            Some(cfg)
        }
        Err(e) => {
            any_fail |= print_check(
                Status::Fail,
                &format!(
                    "Config: {} — {}",
                    opts.config_path.display(),
                    e
                ),
            );
            None
        }
    };

    // ── 2. Index DB ─────────────────────────────────────────────────────────
    if !opts.db_path.exists() {
        any_fail |= print_check(
            Status::Warn,
            &format!(
                "Index DB: not found at {} — run `semanticfs index build`",
                opts.db_path.display()
            ),
        );
    } else {
        match IndexerDb::open(&opts.db_path) {
            Err(e) => {
                any_fail |= print_check(
                    Status::Fail,
                    &format!("Index DB: could not open — {}", e),
                );
            }
            Ok(db) => {
                let version = db.active_version().unwrap_or(0);
                if version == 0 {
                    any_fail |= print_check(
                        Status::Warn,
                        "Index DB: exists but no published version — run `semanticfs index build`",
                    );
                } else {
                    // Use DB file mtime as a proxy for last update time
                    let age_str = opts
                        .db_path
                        .metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| {
                            let secs = d.as_secs();
                            let now = std::time::SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(secs);
                            let age = now.saturating_sub(secs);
                            if age < 60 {
                                format!("{age}s ago")
                            } else if age < 3600 {
                                format!("{}m ago", age / 60)
                            } else if age < 86400 {
                                format!("{}h ago", age / 3600)
                            } else {
                                format!("{}d ago", age / 86400)
                            }
                        })
                        .unwrap_or_else(|| "unknown".to_string());
                    any_fail |= print_check(
                        Status::Ok,
                        &format!("Index DB: version={version}, last updated {age_str}"),
                    );
                }
            }
        }
    }

    // ── 3. Embedding backend ────────────────────────────────────────────────
    {
        let onnx_model = default_model_path();
        let env_override = std::env::var("SEMANTICFS_ONNX_MODEL").ok();
        let cfg_runtime = cfg.as_ref().map(|c| c.embedding.runtime.as_str() == "onnx").unwrap_or(false);

        if let Some(ref path) = env_override {
            any_fail |= print_check(
                Status::Ok,
                &format!("Embeddings: ONNX via SEMANTICFS_ONNX_MODEL={path}"),
            );
        } else if cfg_runtime {
            any_fail |= print_check(
                Status::Ok,
                "Embeddings: ONNX (configured in semanticfs.toml)",
            );
        } else if onnx_model.as_ref().map(|p| p.exists()).unwrap_or(false) {
            let path = onnx_model.unwrap();
            any_fail |= print_check(
                Status::Ok,
                &format!("Embeddings: ONNX auto-detected at {}", path.display()),
            );
        } else {
            any_fail |= print_check(
                Status::Warn,
                "Embeddings: hash backend (keyword/symbol queries only) — run `semanticfs model setup` for semantic search",
            );
        }
    }

    // ── 4. HTTP server reachability ─────────────────────────────────────────
    if opts.http_check {
        let bind = cfg
            .as_ref()
            .map(|c| c.observability.metrics_bind.clone())
            .unwrap_or_else(|| "127.0.0.1:9464".to_string());
        let url = format!("http://{}/health/live", bind);
        match ureq::get(&url).timeout(std::time::Duration::from_secs(2)).call() {
            Ok(_) => {
                any_fail |= print_check(
                    Status::Ok,
                    &format!("HTTP server: reachable at {bind}"),
                );
            }
            Err(_) => {
                any_fail |= print_check(
                    Status::Warn,
                    &format!(
                        "HTTP server: not reachable at {bind} — start with `semanticfs serve mcp` (only needed for HTTP/OpenClaw mode)"
                    ),
                );
            }
        }
    }

    // ── 5. stdio MCP transport ──────────────────────────────────────────────
    {
        let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("semanticfs"));
        any_fail |= print_check(
            Status::Ok,
            &format!("stdio MCP: available ({})", exe.display()),
        );
    }

    // ── 6. Policy guard ─────────────────────────────────────────────────────
    if let Some(ref cfg) = cfg {
        match PolicyGuard::from_config(cfg) {
            Err(e) => {
                any_fail |= print_check(
                    Status::Fail,
                    &format!("Policy guard: failed to load — {e}"),
                );
            }
            Ok(guard) => {
                let domains = guard.domain_ids().len();
                let scan = guard.scan_target_count();
                let watch = guard.watch_target_count();
                let label = if domains > 0 {
                    format!("{domains} domain(s), {scan} scan target(s), {watch} watch target(s)")
                } else {
                    format!("{scan} scan target(s), {watch} watch target(s)")
                };
                if scan == 0 {
                    any_fail |= print_check(
                        Status::Warn,
                        &format!("Policy guard: no scan targets configured — {label}"),
                    );
                } else {
                    any_fail |= print_check(Status::Ok, &format!("Policy guard: {label}"));
                }
            }
        }
    }

    println!("{}", "-".repeat(60));

    if any_fail {
        println!("Doctor found issues. Fix the items marked [FAIL] above.");
        std::process::exit(1);
    } else {
        println!("All checks passed.");
    }

    Ok(())
}

/// Mirrors the `find_auto_model()` logic in `crates/indexer/src/embedding.rs`.
fn default_model_path() -> Option<PathBuf> {
    let home = std::env::var("SEMANTICFS_HOME")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map(|h| PathBuf::from(h).join(".semanticfs"))
        })
        .ok()?;
    Some(home.join("models/model_quantized.onnx"))
}
