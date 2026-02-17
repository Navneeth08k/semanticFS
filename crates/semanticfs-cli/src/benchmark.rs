use anyhow::{Context, Result};
use fuse_bridge::FuseBridge;
use indexer::embedding::{onnx_metrics_snapshot, reset_onnx_metrics, Embedder};
use indexer::Indexer;
use semanticfs_common::SemanticFsConfig;
use serde_json::json;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessesToUpdate, System};
use walkdir::WalkDir;

pub struct BenchmarkRunOptions {
    pub config_path: PathBuf,
    pub soak_seconds: u64,
    pub skip_reindex: bool,
    pub fixture_repo: Option<PathBuf>,
}

pub struct LanceDbTuneOptions {
    pub config_path: PathBuf,
    pub fixture_repo: Option<PathBuf>,
    pub soak_seconds: u64,
}

pub struct ReleaseGateOptions {
    pub refresh: bool,
    pub config_path: PathBuf,
    pub fixture_repo: Option<PathBuf>,
    pub soak_seconds: u64,
    pub max_query_p95_ms: f64,
    pub max_soak_p95_ms: f64,
    pub max_rss_mb: u64,
}

pub struct SoakOptions {
    pub config_path: PathBuf,
    pub duration_seconds: u64,
    pub skip_reindex: bool,
    pub fixture_repo: Option<PathBuf>,
    pub max_soak_p95_ms: f64,
    pub max_errors: u64,
    pub max_rss_mb: u64,
}

pub struct OnnxTuneOptions {
    pub config_path: PathBuf,
    pub fixture_repo: Option<PathBuf>,
    pub samples: usize,
    pub rounds: usize,
    pub batch_sizes: Vec<usize>,
    pub max_lengths: Vec<usize>,
    pub providers: Vec<String>,
}

pub fn run(options: BenchmarkRunOptions) -> Result<()> {
    reset_onnx_metrics();

    let mut cfg = SemanticFsConfig::load(&options.config_path)
        .with_context(|| format!("load config from {}", options.config_path.display()))?;

    if let Some(fixture) = options.fixture_repo {
        cfg.workspace.repo_root = fixture.to_string_lossy().to_string();
    }

    let db_path = PathBuf::from("semanticfs.db");
    let indexer = Indexer::new(cfg.clone(), &db_path)?;

    let active = if options.skip_reindex {
        indexer.active_version()?
    } else {
        let version = indexer.build_full_index()?;
        if cfg
            .map
            .llm_enrichment
            .eq_ignore_ascii_case("async_optional")
        {
            let _ = indexer.enrich_map_for_version(version);
        }
        version
    };

    if active == 0 {
        anyhow::bail!("no active index version found; run `semanticfs index build` first");
    }

    let bridge = FuseBridge::new(cfg, &db_path)?;

    let e2e = run_e2e_checks(&bridge, active)?;
    let soak = run_soak(&bridge, active, options.soak_seconds.max(1));

    let rss_mb = current_process_rss_mb();
    let onnx = onnx_metrics_snapshot();

    let report = json!({
        "active_version": active,
        "e2e": {
            "passed": e2e.passed,
            "checks_total": e2e.total,
            "checks_passed": e2e.passed_count,
            "failures": e2e.failures,
        },
        "soak": {
            "duration_sec": options.soak_seconds.max(1),
            "operations": soak.operations,
            "errors": soak.errors,
            "latency_ms": {
                "p50": soak.p50_ms,
                "p95": soak.p95_ms,
                "max": soak.max_ms
            }
        },
        "process": {
            "rss_mb": rss_mb
        },
        "onnx": {
            "requests_total": onnx.requests_total,
            "batches_total": onnx.batches_total,
            "texts_total": onnx.texts_total,
            "failures_total": onnx.failures_total,
            "health_checks_total": onnx.health_checks_total,
            "health_check_failures_total": onnx.health_check_failures_total,
            "queue_depth_current": onnx.queue_depth_current,
            "queue_depth_max": onnx.queue_depth_max,
            "latency_ms": {
                "count": onnx.latency_count,
                "sum": onnx.latency_sum_ms,
                "max": onnx.latency_max_ms
            }
        }
    });

    let out_dir = PathBuf::from(".semanticfs").join("bench");
    std::fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join("latest.json");
    std::fs::write(&out_path, serde_json::to_string_pretty(&report)?)?;

    println!("{}", serde_json::to_string_pretty(&report)?);
    println!("saved benchmark report: {}", out_path.display());
    Ok(())
}

pub fn tune_lancedb(options: LanceDbTuneOptions) -> Result<()> {
    reset_onnx_metrics();

    let mut base_cfg = SemanticFsConfig::load(&options.config_path)
        .with_context(|| format!("load config from {}", options.config_path.display()))?;
    if let Some(fixture) = options.fixture_repo {
        base_cfg.workspace.repo_root = fixture.to_string_lossy().to_string();
    }

    let old_backend = std::env::var("SEMANTICFS_VECTOR_BACKEND").ok();
    let old_uri = std::env::var("SEMANTICFS_LANCEDB_URI").ok();

    let db_path = PathBuf::from("semanticfs.db");
    let queries = fixed_query_set();

    let mut passes = Vec::new();
    for backend in ["sqlite", "lancedb"] {
        for topn in [10usize, 20usize, 40usize] {
            let mut cfg = base_cfg.clone();
            cfg.retrieval.topn_vector = topn;
            cfg.retrieval.topn_final = cfg.retrieval.topn_final.max(5);

            std::env::set_var("SEMANTICFS_VECTOR_BACKEND", backend);
            if backend == "lancedb" {
                let uri = format!("./.semanticfs/lancedb_tuning/topn_{}", topn);
                std::env::set_var("SEMANTICFS_LANCEDB_URI", &uri);
            }

            let pass = run_backend_pass(
                &cfg,
                &db_path,
                backend,
                topn,
                &queries,
                options.soak_seconds,
            )?;
            passes.push(json!({
                "backend": pass.backend,
                "topn_vector": pass.topn_vector,
                "active_version": pass.active_version,
                "query_bench": {
                    "iterations": pass.query_bench.iterations,
                    "errors": pass.query_bench.errors,
                    "p50_ms": pass.query_bench.p50_ms,
                    "p95_ms": pass.query_bench.p95_ms,
                    "max_ms": pass.query_bench.max_ms
                },
                "soak": {
                    "operations": pass.soak.operations,
                    "errors": pass.soak.errors,
                    "p50_ms": pass.soak.p50_ms,
                    "p95_ms": pass.soak.p95_ms,
                    "max_ms": pass.soak.max_ms
                },
                "rss_mb": pass.rss_mb,
                "onnx": pass.onnx
            }));
        }
    }

    restore_env("SEMANTICFS_VECTOR_BACKEND", old_backend);
    restore_env("SEMANTICFS_LANCEDB_URI", old_uri);

    let out = json!({
        "scenario": "lancedb_tuning",
        "repo_root": base_cfg.workspace.repo_root,
        "passes": passes
    });

    let out_dir = PathBuf::from(".semanticfs").join("bench");
    std::fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join("lancedb_tuning.json");
    std::fs::write(&out_path, serde_json::to_string_pretty(&out)?)?;

    println!("{}", serde_json::to_string_pretty(&out)?);
    println!("saved tuning report: {}", out_path.display());
    Ok(())
}

pub fn soak(options: SoakOptions) -> Result<()> {
    reset_onnx_metrics();

    let mut cfg = SemanticFsConfig::load(&options.config_path)
        .with_context(|| format!("load config from {}", options.config_path.display()))?;
    if let Some(fixture) = options.fixture_repo {
        cfg.workspace.repo_root = fixture.to_string_lossy().to_string();
    }

    let db_path = PathBuf::from("semanticfs.db");
    let indexer = Indexer::new(cfg.clone(), &db_path)?;

    let active = if options.skip_reindex {
        indexer.active_version()?
    } else {
        let version = indexer.build_full_index()?;
        if cfg
            .map
            .llm_enrichment
            .eq_ignore_ascii_case("async_optional")
        {
            let _ = indexer.enrich_map_for_version(version);
        }
        version
    };

    if active == 0 {
        anyhow::bail!("no active index version found; run `semanticfs index build` first");
    }

    let bridge = FuseBridge::new(cfg, &db_path)?;
    let soak = run_soak(&bridge, active, options.duration_seconds.max(1));
    let rss_mb = current_process_rss_mb();
    let onnx = onnx_metrics_snapshot();

    let mut checks = Vec::new();
    let mut passed = true;
    record_check(
        &mut checks,
        "soak_errors_threshold",
        soak.errors <= options.max_errors,
        format!("errors={} <= {}", soak.errors, options.max_errors),
        &mut passed,
    );
    record_check(
        &mut checks,
        "soak_p95_threshold",
        soak.p95_ms <= options.max_soak_p95_ms,
        format!(
            "soak.p95_ms={:.3} <= {:.3}",
            soak.p95_ms, options.max_soak_p95_ms
        ),
        &mut passed,
    );
    record_check(
        &mut checks,
        "rss_threshold",
        rss_mb <= options.max_rss_mb,
        format!("rss_mb={} <= {}", rss_mb, options.max_rss_mb),
        &mut passed,
    );

    let report = json!({
        "scenario": "long_soak",
        "active_version": active,
        "duration_sec": options.duration_seconds.max(1),
        "soak": {
            "operations": soak.operations,
            "errors": soak.errors,
            "latency_ms": {
                "p50": soak.p50_ms,
                "p95": soak.p95_ms,
                "max": soak.max_ms
            }
        },
        "process": {
            "rss_mb": rss_mb
        },
        "onnx": {
            "requests_total": onnx.requests_total,
            "batches_total": onnx.batches_total,
            "texts_total": onnx.texts_total,
            "failures_total": onnx.failures_total,
            "queue_depth_current": onnx.queue_depth_current,
            "queue_depth_max": onnx.queue_depth_max,
            "latency_ms": {
                "count": onnx.latency_count,
                "sum": onnx.latency_sum_ms,
                "max": onnx.latency_max_ms
            }
        },
        "thresholds": {
            "max_soak_p95_ms": options.max_soak_p95_ms,
            "max_errors": options.max_errors,
            "max_rss_mb": options.max_rss_mb
        },
        "checks": checks,
        "passed": passed
    });

    let out_dir = PathBuf::from(".semanticfs").join("bench");
    std::fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join("soak_latest.json");
    std::fs::write(&out_path, serde_json::to_string_pretty(&report)?)?;

    println!("{}", serde_json::to_string_pretty(&report)?);
    println!("saved soak report: {}", out_path.display());

    if !passed {
        anyhow::bail!("long soak gate failed");
    }
    Ok(())
}

pub fn tune_onnx(options: OnnxTuneOptions) -> Result<()> {
    if options.batch_sizes.is_empty()
        || options.max_lengths.is_empty()
        || options.providers.is_empty()
    {
        anyhow::bail!("batch_sizes, max_lengths, and providers must be non-empty");
    }

    let mut cfg = SemanticFsConfig::load(&options.config_path)
        .with_context(|| format!("load config from {}", options.config_path.display()))?;
    if let Some(fixture) = options.fixture_repo {
        cfg.workspace.repo_root = fixture.to_string_lossy().to_string();
    }
    cfg.embedding.runtime = "onnx".to_string();

    let samples = collect_text_samples(&cfg.workspace.repo_root, options.samples.max(1))?;
    if samples.is_empty() {
        anyhow::bail!(
            "no text samples collected from repo_root={} for ONNX tuning",
            cfg.workspace.repo_root
        );
    }

    let old_max_length = std::env::var("SEMANTICFS_ONNX_MAX_LENGTH").ok();
    let old_provider = std::env::var("SEMANTICFS_ONNX_PROVIDER").ok();

    let mut passes = Vec::new();
    for provider in &options.providers {
        for &max_length in &options.max_lengths {
            for &batch_size in &options.batch_sizes {
                reset_onnx_metrics();
                std::env::set_var("SEMANTICFS_ONNX_PROVIDER", provider);
                std::env::set_var("SEMANTICFS_ONNX_MAX_LENGTH", max_length.to_string());

                let mut pass_cfg = cfg.clone();
                pass_cfg.embedding.batch_size = batch_size.max(1);

                let embedder = Embedder::from_config(&pass_cfg.embedding)?;
                let started = Instant::now();
                let mut embedded = 0usize;

                for _ in 0..options.rounds.max(1) {
                    let vecs = embedder.embed_batch(&samples)?;
                    embedded += vecs.len();
                }

                let elapsed = started.elapsed().as_secs_f64().max(0.000_001);
                let throughput = (embedded as f64) / elapsed;
                let onnx = onnx_metrics_snapshot();

                passes.push(json!({
                    "provider": provider,
                    "max_length": max_length,
                    "batch_size": batch_size,
                    "rounds": options.rounds.max(1),
                    "samples_per_round": samples.len(),
                    "total_texts": embedded,
                    "elapsed_sec": elapsed,
                    "throughput_texts_per_sec": throughput,
                    "onnx": {
                        "requests_total": onnx.requests_total,
                        "batches_total": onnx.batches_total,
                        "texts_total": onnx.texts_total,
                        "failures_total": onnx.failures_total,
                        "queue_depth_max": onnx.queue_depth_max,
                        "latency_count": onnx.latency_count,
                        "latency_sum_ms": onnx.latency_sum_ms,
                        "latency_max_ms": onnx.latency_max_ms
                    }
                }));
            }
        }
    }

    restore_env("SEMANTICFS_ONNX_MAX_LENGTH", old_max_length);
    restore_env("SEMANTICFS_ONNX_PROVIDER", old_provider);

    let best = passes
        .iter()
        .max_by(|a, b| {
            let a_tp = a
                .get("throughput_texts_per_sec")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let b_tp = b
                .get("throughput_texts_per_sec")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            a_tp.partial_cmp(&b_tp).unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned();

    let out = json!({
        "scenario": "onnx_tuning",
        "repo_root": cfg.workspace.repo_root,
        "samples": samples.len(),
        "passes": passes,
        "best": best
    });

    let out_dir = PathBuf::from(".semanticfs").join("bench");
    std::fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join("onnx_tuning.json");
    std::fs::write(&out_path, serde_json::to_string_pretty(&out)?)?;

    println!("{}", serde_json::to_string_pretty(&out)?);
    println!("saved onnx tuning report: {}", out_path.display());
    Ok(())
}

pub fn release_gate(options: ReleaseGateOptions) -> Result<()> {
    if options.refresh {
        run(BenchmarkRunOptions {
            config_path: options.config_path.clone(),
            soak_seconds: options.soak_seconds.max(1),
            skip_reindex: false,
            fixture_repo: options.fixture_repo.clone(),
        })?;
        tune_lancedb(LanceDbTuneOptions {
            config_path: options.config_path.clone(),
            fixture_repo: options.fixture_repo.clone(),
            soak_seconds: options.soak_seconds.max(1),
        })?;
    }

    let latest_path = PathBuf::from(".semanticfs/bench/latest.json");
    let tuning_path = PathBuf::from(".semanticfs/bench/lancedb_tuning.json");

    let latest = read_json(&latest_path)?;
    let tuning = read_json(&tuning_path)?;

    let mut checks = Vec::new();
    let mut passed = true;

    let e2e_passed = latest
        .get("e2e")
        .and_then(|v| v.get("passed"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    record_check(
        &mut checks,
        "e2e_passed",
        e2e_passed,
        format!("e2e.passed={}", e2e_passed),
        &mut passed,
    );

    let latest_soak_errors = as_u64(&latest, &["soak", "errors"]).unwrap_or(u64::MAX);
    record_check(
        &mut checks,
        "latest_soak_errors_zero",
        latest_soak_errors == 0,
        format!("soak.errors={}", latest_soak_errors),
        &mut passed,
    );

    let latest_soak_p95 = as_f64(&latest, &["soak", "latency_ms", "p95"]).unwrap_or(f64::MAX);
    record_check(
        &mut checks,
        "latest_soak_p95_threshold",
        latest_soak_p95 <= options.max_soak_p95_ms,
        format!(
            "soak.p95_ms={:.3} <= {:.3}",
            latest_soak_p95, options.max_soak_p95_ms
        ),
        &mut passed,
    );

    let latest_rss = as_u64(&latest, &["process", "rss_mb"]).unwrap_or(u64::MAX);
    record_check(
        &mut checks,
        "latest_rss_threshold",
        latest_rss <= options.max_rss_mb,
        format!("rss_mb={} <= {}", latest_rss, options.max_rss_mb),
        &mut passed,
    );

    let passes = tuning
        .get("passes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    record_check(
        &mut checks,
        "tuning_pass_count",
        passes.len() >= 6,
        format!("passes={}", passes.len()),
        &mut passed,
    );

    let has_sqlite = passes
        .iter()
        .any(|p| p.get("backend").and_then(|v| v.as_str()) == Some("sqlite"));
    let has_lancedb = passes
        .iter()
        .any(|p| p.get("backend").and_then(|v| v.as_str()) == Some("lancedb"));
    record_check(
        &mut checks,
        "tuning_backend_coverage",
        has_sqlite && has_lancedb,
        format!("sqlite={} lancedb={}", has_sqlite, has_lancedb),
        &mut passed,
    );

    let mut pass_query_errors = 0u64;
    let mut pass_soak_errors = 0u64;
    let mut worst_query_p95 = 0.0f64;
    let mut worst_soak_p95 = 0.0f64;
    for p in &passes {
        pass_query_errors += as_u64(p, &["query_bench", "errors"]).unwrap_or(u64::MAX / 4);
        pass_soak_errors += as_u64(p, &["soak", "errors"]).unwrap_or(u64::MAX / 4);
        worst_query_p95 = worst_query_p95.max(as_f64(p, &["query_bench", "p95_ms"]).unwrap_or(0.0));
        worst_soak_p95 = worst_soak_p95.max(as_f64(p, &["soak", "p95_ms"]).unwrap_or(0.0));
    }

    record_check(
        &mut checks,
        "tuning_query_errors_zero",
        pass_query_errors == 0,
        format!("query_errors_total={}", pass_query_errors),
        &mut passed,
    );
    record_check(
        &mut checks,
        "tuning_soak_errors_zero",
        pass_soak_errors == 0,
        format!("soak_errors_total={}", pass_soak_errors),
        &mut passed,
    );
    record_check(
        &mut checks,
        "tuning_query_p95_threshold",
        worst_query_p95 <= options.max_query_p95_ms,
        format!(
            "worst_query_p95_ms={:.3} <= {:.3}",
            worst_query_p95, options.max_query_p95_ms
        ),
        &mut passed,
    );
    record_check(
        &mut checks,
        "tuning_soak_p95_threshold",
        worst_soak_p95 <= options.max_soak_p95_ms,
        format!(
            "worst_soak_p95_ms={:.3} <= {:.3}",
            worst_soak_p95, options.max_soak_p95_ms
        ),
        &mut passed,
    );

    let report = json!({
        "release_gate": {
            "passed": passed,
            "thresholds": {
                "max_query_p95_ms": options.max_query_p95_ms,
                "max_soak_p95_ms": options.max_soak_p95_ms,
                "max_rss_mb": options.max_rss_mb
            },
            "checks": checks
        }
    });

    let out_dir = PathBuf::from(".semanticfs/bench");
    std::fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join("release_gate.json");
    std::fs::write(&out_path, serde_json::to_string_pretty(&report)?)?;

    println!("{}", serde_json::to_string_pretty(&report)?);
    println!("saved release gate report: {}", out_path.display());

    if !passed {
        anyhow::bail!("release gate failed");
    }
    Ok(())
}

struct E2eResult {
    passed: bool,
    total: usize,
    passed_count: usize,
    failures: Vec<String>,
}

fn run_e2e_checks(bridge: &FuseBridge, version: u64) -> Result<E2eResult> {
    let mut failures = Vec::new();
    let mut passed_count = 0usize;
    let checks = 4usize;

    let search = bridge.read_virtual("/search/policy_guard.md", version, version)?;
    let search_str = String::from_utf8_lossy(&search);
    if search_str.contains("# Search Results") {
        passed_count += 1;
    } else {
        failures.push("search did not render expected markdown header".to_string());
    }

    let map = bridge.read_virtual("/map/docs/directory_overview.md", version, version)?;
    let map_str = String::from_utf8_lossy(&map);
    if map_str.to_ascii_lowercase().contains("directory overview") {
        passed_count += 1;
    } else {
        failures.push("map overview missing expected content".to_string());
    }

    let raw_target = extract_first_hit_path(&search_str);
    match raw_target {
        Some(path) => {
            let raw_path = format!("/raw/{}", path);
            match bridge.read_virtual(&raw_path, version, version) {
                Ok(bytes) if !bytes.is_empty() => {
                    passed_count += 1;
                }
                Ok(_) => failures.push("raw verify returned empty bytes".to_string()),
                Err(err) => failures.push(format!("raw verify failed: {}", err)),
            }
        }
        None => match bridge.read_virtual("/raw/src/main.rs", version, version) {
            Ok(bytes) if !bytes.is_empty() => {
                passed_count += 1;
            }
            Ok(_) => failures.push("raw fallback read returned empty bytes".to_string()),
            Err(err) => failures.push(format!("raw fallback verify failed: {}", err)),
        },
    }

    match bridge.read_virtual("/.well-known/health.json", version, version) {
        Ok(bytes) => {
            let text = String::from_utf8_lossy(&bytes);
            if text.contains("\"live\":true") {
                passed_count += 1;
            } else {
                failures.push("health payload missing live=true".to_string());
            }
        }
        Err(err) => failures.push(format!("health virtual file failed: {}", err)),
    }

    Ok(E2eResult {
        passed: failures.is_empty(),
        total: checks,
        passed_count,
        failures,
    })
}

struct SoakResult {
    operations: u64,
    errors: u64,
    p50_ms: f64,
    p95_ms: f64,
    max_ms: f64,
}

struct QueryBenchResult {
    iterations: u64,
    errors: u64,
    p50_ms: f64,
    p95_ms: f64,
    max_ms: f64,
}

struct BackendPassResult {
    backend: String,
    topn_vector: usize,
    active_version: u64,
    query_bench: QueryBenchResult,
    soak: SoakResult,
    rss_mb: u64,
    onnx: serde_json::Value,
}

fn run_backend_pass(
    cfg: &SemanticFsConfig,
    db_path: &PathBuf,
    backend: &str,
    topn_vector: usize,
    queries: &[&str],
    soak_seconds: u64,
) -> Result<BackendPassResult> {
    let indexer = Indexer::new(cfg.clone(), db_path)?;
    let version = indexer.build_full_index()?;
    if cfg
        .map
        .llm_enrichment
        .eq_ignore_ascii_case("async_optional")
    {
        let _ = indexer.enrich_map_for_version(version);
    }

    let bridge = FuseBridge::new(cfg.clone(), db_path)?;
    let query_bench = run_query_bench(&bridge, version, queries, 200);
    let soak = run_soak(&bridge, version, soak_seconds.max(1));
    let rss_mb = current_process_rss_mb();
    let onnx = onnx_metrics_snapshot();

    Ok(BackendPassResult {
        backend: backend.to_string(),
        topn_vector,
        active_version: version,
        query_bench,
        soak,
        rss_mb,
        onnx: json!({
            "requests_total": onnx.requests_total,
            "batches_total": onnx.batches_total,
            "texts_total": onnx.texts_total,
            "failures_total": onnx.failures_total,
            "queue_depth_max": onnx.queue_depth_max,
            "latency_ms_max": onnx.latency_max_ms
        }),
    })
}

fn run_query_bench(
    bridge: &FuseBridge,
    version: u64,
    queries: &[&str],
    rounds: usize,
) -> QueryBenchResult {
    let mut latencies = Vec::with_capacity(rounds);
    let mut errors = 0u64;
    for i in 0..rounds {
        let q = queries[i % queries.len()];
        let path = format!("/search/{}.md", q.replace(' ', "_"));
        let t0 = Instant::now();
        if bridge.read_virtual(&path, version, version).is_err() {
            errors += 1;
        }
        latencies.push(t0.elapsed().as_micros() as u64);
    }

    latencies.sort_unstable();
    QueryBenchResult {
        iterations: rounds as u64,
        errors,
        p50_ms: micros_to_ms(percentile(&latencies, 0.50)),
        p95_ms: micros_to_ms(percentile(&latencies, 0.95)),
        max_ms: micros_to_ms(latencies.last().copied().unwrap_or(0)),
    }
}

fn run_soak(bridge: &FuseBridge, version: u64, duration_sec: u64) -> SoakResult {
    let mut latencies = Vec::new();
    let mut errors = 0u64;
    let mut ops = 0u64;
    let start = Instant::now();
    let duration = Duration::from_secs(duration_sec);
    let paths = [
        "/search/indexer.md",
        "/search/map_enrichment.md",
        "/map/docs/directory_overview.md",
        "/.well-known/health.json",
    ];

    while start.elapsed() < duration {
        let idx = (ops as usize) % paths.len();
        let p = paths[idx];
        let t0 = Instant::now();
        if bridge.read_virtual(p, version, version).is_err() {
            errors += 1;
        }
        ops += 1;
        latencies.push(t0.elapsed().as_micros() as u64);
    }

    latencies.sort_unstable();
    let p50 = micros_to_ms(percentile(&latencies, 0.50));
    let p95 = micros_to_ms(percentile(&latencies, 0.95));
    let max = micros_to_ms(latencies.last().copied().unwrap_or(0));

    SoakResult {
        operations: ops,
        errors,
        p50_ms: p50,
        p95_ms: p95,
        max_ms: max,
    }
}

fn percentile(samples: &[u64], p: f64) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let idx = ((samples.len() as f64 - 1.0) * p).round() as usize;
    samples[idx.min(samples.len() - 1)]
}

fn micros_to_ms(us: u64) -> f64 {
    (us as f64) / 1000.0
}

fn extract_first_hit_path(markdown: &str) -> Option<String> {
    for line in markdown.lines() {
        if !line.starts_with("## ") {
            continue;
        }
        let first_tick = line.find('`')?;
        let rest = &line[first_tick + 1..];
        let second_tick = rest.find('`')?;
        let path = &rest[..second_tick];
        if !path.is_empty() {
            return Some(path.to_string());
        }
    }
    None
}

fn current_process_rss_mb() -> u64 {
    let mut sys = System::new_all();
    let pid = Pid::from_u32(std::process::id());
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    sys.process(pid)
        .map(|p| p.memory() / (1024 * 1024))
        .unwrap_or(0)
}

fn fixed_query_set() -> Vec<&'static str> {
    vec![
        "auth token validation",
        "directory overview logic",
        "map enrichment",
        "policy guard deny",
        "index version publish",
        "search codebase tool",
    ]
}

fn restore_env(name: &str, value: Option<String>) {
    match value {
        Some(v) => std::env::set_var(name, v),
        None => std::env::remove_var(name),
    }
}

fn collect_text_samples(repo_root: &str, limit: usize) -> Result<Vec<String>> {
    let mut out = Vec::with_capacity(limit);
    let root = PathBuf::from(repo_root);
    for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        if out.len() >= limit {
            break;
        }
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let allowed = matches!(
            ext.as_str(),
            "rs" | "py"
                | "ts"
                | "tsx"
                | "js"
                | "jsx"
                | "java"
                | "go"
                | "md"
                | "txt"
                | "toml"
                | "yaml"
                | "yml"
                | "json"
        );
        if !allowed {
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(path) else {
            continue;
        };
        for chunk in raw.split("\n\n") {
            let s = chunk.trim();
            if s.len() < 20 {
                continue;
            }
            out.push(s.chars().take(1200).collect::<String>());
            if out.len() >= limit {
                break;
            }
        }
    }
    Ok(out)
}

fn read_json(path: &PathBuf) -> Result<serde_json::Value> {
    let raw =
        std::fs::read_to_string(path).with_context(|| format!("read json {}", path.display()))?;
    let value = serde_json::from_str::<serde_json::Value>(&raw)
        .with_context(|| format!("parse json {}", path.display()))?;
    Ok(value)
}

fn as_u64(v: &serde_json::Value, path: &[&str]) -> Option<u64> {
    let mut cur = v;
    for k in path {
        cur = cur.get(*k)?;
    }
    cur.as_u64()
}

fn as_f64(v: &serde_json::Value, path: &[&str]) -> Option<f64> {
    let mut cur = v;
    for k in path {
        cur = cur.get(*k)?;
    }
    cur.as_f64()
}

fn record_check(
    checks: &mut Vec<serde_json::Value>,
    id: &str,
    ok: bool,
    detail: String,
    overall: &mut bool,
) {
    if !ok {
        *overall = false;
    }
    checks.push(json!({
        "id": id,
        "ok": ok,
        "detail": detail
    }));
}
