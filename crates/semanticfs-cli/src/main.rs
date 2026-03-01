use anyhow::{Context, Result};
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use clap::{Args, Parser, Subcommand};
use fuse_bridge::FuseBridge;
use indexer::embedding::onnx_metrics_snapshot;
use indexer::Indexer;
use mcp::McpServer;
use semanticfs_common::SemanticFsConfig;
use serde_json::json;
use std::{fs, net::SocketAddr, path::PathBuf, process::Command, sync::Arc, time::Instant};
use sysinfo::{Pid, ProcessesToUpdate, System};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod benchmark;

#[derive(Parser, Debug)]
#[command(name = "semanticfs")]
struct Cli {
    #[arg(long, default_value = "config/semanticfs.sample.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init(InitArgs),
    Index {
        #[command(subcommand)]
        command: IndexCommand,
    },
    Serve {
        #[command(subcommand)]
        command: ServeCommand,
    },
    Health,
    Benchmark {
        #[command(subcommand)]
        command: BenchmarkCommand,
    },
    Recover {
        #[command(subcommand)]
        command: RecoverCommand,
    },
}

#[derive(Args, Debug)]
struct InitArgs {
    #[arg(long)]
    repo: String,
    #[arg(long)]
    mount: String,
}

#[derive(Subcommand, Debug)]
enum IndexCommand {
    Build,
    Watch,
    Enrich {
        #[arg(long)]
        version: Option<u64>,
    },
}

#[derive(Subcommand, Debug)]
enum ServeCommand {
    Fuse,
    Mcp,
    Observability,
}

#[derive(Subcommand, Debug)]
enum RecoverCommand {
    Mount {
        #[arg(long)]
        force_unmount: bool,
    },
}

#[derive(Subcommand, Debug)]
enum BenchmarkCommand {
    Run {
        #[arg(long, default_value_t = 60)]
        soak_seconds: u64,
        #[arg(long, default_value_t = false)]
        skip_reindex: bool,
        #[arg(long)]
        fixture_repo: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        history: bool,
    },
    TuneLancedb {
        #[arg(long, default_value_t = 10)]
        soak_seconds: u64,
        #[arg(long)]
        fixture_repo: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        history: bool,
    },
    TuneOnnx {
        #[arg(long)]
        fixture_repo: Option<PathBuf>,
        #[arg(long, default_value_t = 1000)]
        samples: usize,
        #[arg(long, default_value_t = 5)]
        rounds: usize,
        #[arg(long, default_value = "16,32,64")]
        batch_sizes: String,
        #[arg(long, default_value = "128,256,384,512")]
        max_lengths: String,
        #[arg(long, default_value = "CPUExecutionProvider")]
        providers: String,
        #[arg(long, default_value_t = false)]
        history: bool,
    },
    Soak {
        #[arg(long, default_value_t = 3600)]
        duration_seconds: u64,
        #[arg(long, default_value_t = false)]
        skip_reindex: bool,
        #[arg(long)]
        fixture_repo: Option<PathBuf>,
        #[arg(long, default_value_t = 250.0)]
        max_soak_p95_ms: f64,
        #[arg(long, default_value_t = 0)]
        max_errors: u64,
        #[arg(long, default_value_t = 2048)]
        max_rss_mb: u64,
        #[arg(long, default_value_t = false)]
        history: bool,
    },
    ReleaseGate {
        #[arg(long, default_value_t = false)]
        refresh: bool,
        #[arg(long, default_value_t = 10)]
        soak_seconds: u64,
        #[arg(long)]
        fixture_repo: Option<PathBuf>,
        #[arg(long, default_value_t = 250.0)]
        max_query_p95_ms: f64,
        #[arg(long, default_value_t = 250.0)]
        max_soak_p95_ms: f64,
        #[arg(long, default_value_t = 2048)]
        max_rss_mb: u64,
        #[arg(long, default_value_t = false)]
        enforce_relevance: bool,
        #[arg(long, default_value_t = 20)]
        min_relevance_queries: u64,
        #[arg(long, default_value_t = 0.90)]
        min_recall_at_5: f64,
        #[arg(long, default_value_t = 0.99)]
        min_symbol_hit_rate: f64,
        #[arg(long, default_value_t = 0.80)]
        min_mrr: f64,
    },
    Relevance {
        #[arg(long)]
        fixture_repo: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        skip_reindex: bool,
        #[arg(long)]
        golden: Option<PathBuf>,
        #[arg(long)]
        golden_dir: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        history: bool,
    },
    HeadToHead {
        #[arg(long)]
        fixture_repo: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        skip_reindex: bool,
        #[arg(long)]
        golden: Option<PathBuf>,
        #[arg(long)]
        golden_dir: Option<PathBuf>,
        #[arg(long, default_value_t = 5)]
        baseline_topn: usize,
        #[arg(long, default_value_t = false)]
        history: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,lance=warn,lancedb=warn"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Init(args) => init_command(args, &cli.config),
        Commands::Index { command } => index_command(command, &cli.config),
        Commands::Serve { command } => serve_command(command, &cli.config).await,
        Commands::Health => health_command(&cli.config),
        Commands::Benchmark { command } => benchmark_command(command, &cli.config),
        Commands::Recover { command } => recover_command(command),
    }
}

fn init_command(args: InitArgs, target_path: &PathBuf) -> Result<()> {
    let sample = format!(
        "[workspace]\nrepo_root = \"{}\"\nmount_point = \"{}\"\n\n# Optional Phase 3 multi-root example:\n# [[workspace.domains]]\n# id = \"primary\"\n# root = \"{}\"\n# trust_label = \"trusted\"\n# allow_roots = [\"**\"]\n# deny_globs = []\n\n# copy remaining defaults from config/semanticfs.sample.toml\n",
        args.repo, args.mount, args.repo
    );
    fs::write(target_path, sample)?;
    info!(path = %target_path.display(), "initialized config");
    Ok(())
}

fn index_command(cmd: IndexCommand, config_path: &PathBuf) -> Result<()> {
    let cfg = SemanticFsConfig::load(config_path)
        .with_context(|| format!("load config from {}", config_path.display()))?;
    let enable_async_map_enrichment = cfg
        .map
        .llm_enrichment
        .eq_ignore_ascii_case("async_optional");

    let db_path = resolve_db_path();
    let indexer = Indexer::new(cfg, &db_path)?;

    match cmd {
        IndexCommand::Build => {
            let version = indexer.build_full_index()?;
            println!("index built and published: version={}", version);
            if enable_async_map_enrichment {
                spawn_async_map_enrichment(config_path, &db_path, version)?;
            }
        }
        IndexCommand::Watch => {
            println!("starting watch mode; press Ctrl+C to stop");
            indexer.watch()?;
        }
        IndexCommand::Enrich { version } => {
            let target = match version {
                Some(v) => v,
                None => indexer.active_version()?,
            };
            indexer.enrich_map_for_version(target)?;
            println!("map enrichment complete: version={}", target);
        }
    }
    Ok(())
}

async fn serve_command(cmd: ServeCommand, config_path: &PathBuf) -> Result<()> {
    let cfg = SemanticFsConfig::load(config_path)?;
    let db_path = resolve_db_path();

    match cmd {
        ServeCommand::Fuse => {
            let bridge = FuseBridge::new(cfg.clone(), &db_path)?;
            println!("starting fuse mount at {}", cfg.workspace.mount_point);
            bridge.mount()?;
        }
        ServeCommand::Mcp => {
            let bridge = FuseBridge::new(cfg.clone(), &db_path)?;
            let bind = cfg.observability.metrics_bind.clone();
            let server = McpServer::new(bridge);
            server.serve(&bind).await?;
        }
        ServeCommand::Observability => {
            let bridge = FuseBridge::new(cfg.clone(), &db_path)?;
            let bind = cfg.observability.health_bind.clone();
            serve_observability(bridge, &db_path, &bind).await?;
        }
    }

    Ok(())
}

fn health_command(config_path: &PathBuf) -> Result<()> {
    let cfg = SemanticFsConfig::load(config_path)?;
    println!("repo_root={}", cfg.workspace.repo_root);
    println!("mount_point={}", cfg.workspace.mount_point);
    let domains = cfg.effective_workspace_domains();
    println!("workspace_domain_count={}", domains.len());
    for domain in domains {
        let allow = if domain.allow_roots.is_empty() {
            "-".to_string()
        } else {
            domain.allow_roots.join(",")
        };
        let deny = if domain.deny_globs.is_empty() {
            "-".to_string()
        } else {
            domain.deny_globs.join(",")
        };
        println!(
            "workspace_domain={} root={} trust_label={} allow_roots={} deny_globs={}",
            domain.id, domain.root, domain.trust_label, allow, deny
        );
    }
    println!("status=healthy (static scaffold)");
    Ok(())
}

fn benchmark_command(cmd: BenchmarkCommand, config_path: &PathBuf) -> Result<()> {
    match cmd {
        BenchmarkCommand::Run {
            soak_seconds,
            skip_reindex,
            fixture_repo,
            history,
        } => benchmark::run(benchmark::BenchmarkRunOptions {
            config_path: config_path.clone(),
            soak_seconds,
            skip_reindex,
            fixture_repo,
            history,
        }),
        BenchmarkCommand::TuneLancedb {
            soak_seconds,
            fixture_repo,
            history,
        } => benchmark::tune_lancedb(benchmark::LanceDbTuneOptions {
            config_path: config_path.clone(),
            fixture_repo,
            soak_seconds,
            history,
        }),
        BenchmarkCommand::TuneOnnx {
            fixture_repo,
            samples,
            rounds,
            batch_sizes,
            max_lengths,
            providers,
            history,
        } => benchmark::tune_onnx(benchmark::OnnxTuneOptions {
            config_path: config_path.clone(),
            fixture_repo,
            samples,
            rounds,
            batch_sizes: parse_usize_csv(&batch_sizes)?,
            max_lengths: parse_usize_csv(&max_lengths)?,
            providers: parse_string_csv(&providers),
            history,
        }),
        BenchmarkCommand::Soak {
            duration_seconds,
            skip_reindex,
            fixture_repo,
            max_soak_p95_ms,
            max_errors,
            max_rss_mb,
            history,
        } => benchmark::soak(benchmark::SoakOptions {
            config_path: config_path.clone(),
            duration_seconds,
            skip_reindex,
            fixture_repo,
            max_soak_p95_ms,
            max_errors,
            max_rss_mb,
            history,
        }),
        BenchmarkCommand::ReleaseGate {
            refresh,
            soak_seconds,
            fixture_repo,
            max_query_p95_ms,
            max_soak_p95_ms,
            max_rss_mb,
            enforce_relevance,
            min_relevance_queries,
            min_recall_at_5,
            min_symbol_hit_rate,
            min_mrr,
        } => benchmark::release_gate(benchmark::ReleaseGateOptions {
            refresh,
            config_path: config_path.clone(),
            fixture_repo,
            soak_seconds,
            max_query_p95_ms,
            max_soak_p95_ms,
            max_rss_mb,
            enforce_relevance,
            min_relevance_queries,
            min_recall_at_5,
            min_symbol_hit_rate,
            min_mrr,
        }),
        BenchmarkCommand::Relevance {
            fixture_repo,
            skip_reindex,
            golden,
            golden_dir,
            history,
        } => benchmark::relevance(benchmark::RelevanceOptions {
            config_path: config_path.clone(),
            fixture_repo,
            skip_reindex,
            golden_path: golden,
            golden_dir,
            history,
        }),
        BenchmarkCommand::HeadToHead {
            fixture_repo,
            skip_reindex,
            golden,
            golden_dir,
            baseline_topn,
            history,
        } => benchmark::head_to_head(benchmark::HeadToHeadOptions {
            config_path: config_path.clone(),
            fixture_repo,
            skip_reindex,
            golden_path: golden,
            golden_dir,
            baseline_topn,
            history,
        }),
    }
}

fn parse_usize_csv(raw: &str) -> Result<Vec<usize>> {
    let vals = raw
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<usize>()
                .with_context(|| format!("invalid usize value in csv list: {}", s))
        })
        .collect::<Result<Vec<_>>>()?;
    if vals.is_empty() {
        anyhow::bail!("csv list must contain at least one value");
    }
    Ok(vals)
}

fn parse_string_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn recover_command(cmd: RecoverCommand) -> Result<()> {
    match cmd {
        RecoverCommand::Mount { force_unmount } => {
            if force_unmount {
                println!("Run: fusermount -uz /mnt/ai (Linux) to clear stale mount");
            } else {
                println!("Use --force-unmount for stale mount recovery guidance");
            }
        }
    }
    Ok(())
}

fn spawn_async_map_enrichment(
    config_path: &PathBuf,
    db_path: &PathBuf,
    version: u64,
) -> Result<()> {
    let exe = std::env::current_exe()?;
    let child = Command::new(exe)
        .arg("--config")
        .arg(config_path)
        .arg("index")
        .arg("enrich")
        .arg("--version")
        .arg(version.to_string())
        .spawn()?;

    info!(
        version,
        pid = child.id(),
        db_path = %db_path.display(),
        "spawned async map enrichment worker process"
    );
    Ok(())
}

fn resolve_db_path() -> PathBuf {
    std::env::var("SEMANTICFS_DB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("semanticfs.db"))
}

#[derive(Clone)]
struct ObservabilityState {
    bridge: Arc<FuseBridge>,
    db_path: PathBuf,
    started_at: Instant,
}

async fn serve_observability(bridge: FuseBridge, db_path: &PathBuf, bind: &str) -> Result<()> {
    let state = ObservabilityState {
        bridge: Arc::new(bridge),
        db_path: db_path.clone(),
        started_at: Instant::now(),
    };

    let app = Router::new()
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .route("/health/index", get(health_index))
        .route("/metrics", get(metrics_prometheus))
        .with_state(state);

    let addr: SocketAddr = bind.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_live(State(state): State<ObservabilityState>) -> Json<serde_json::Value> {
    Json(json!({
        "live": true,
        "uptime_seconds": state.started_at.elapsed().as_secs()
    }))
}

async fn health_ready(State(state): State<ObservabilityState>) -> Json<serde_json::Value> {
    let db_exists = state.db_path.exists();
    let active_version = state.bridge.active_version().ok();
    let ready = db_exists && active_version.is_some();
    Json(json!({
        "ready": ready,
        "db_exists": db_exists,
        "active_version": active_version
    }))
}

async fn health_index(State(state): State<ObservabilityState>) -> Json<serde_json::Value> {
    let active_version = state.bridge.active_version().unwrap_or(0);
    let (inode_entries, content_entries) = state.bridge.cache_stats();
    let onnx = onnx_metrics_snapshot();
    Json(json!({
        "active_version": active_version,
        "staging_version": serde_json::Value::Null,
        "queue_depth": onnx.queue_depth_current,
        "lag_ms": 0,
        "cache": {
            "inode_entries": inode_entries,
            "content_entries": content_entries
        },
        "onnx": {
            "requests_total": onnx.requests_total,
            "failures_total": onnx.failures_total,
            "queue_depth_max": onnx.queue_depth_max
        }
    }))
}

async fn metrics_prometheus(State(state): State<ObservabilityState>) -> impl IntoResponse {
    let stats = state.bridge.stats_snapshot();
    let (inode_entries, _) = state.bridge.cache_stats();

    let mut system = System::new_all();
    let pid = Pid::from_u32(std::process::id());
    system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    let rss_mb = system
        .process(pid)
        .map(|p| p.memory() / (1024 * 1024))
        .unwrap_or(0);

    let inode_total = stats.inode_cache_hits + stats.inode_cache_misses;
    let content_total = stats.content_cache_hits + stats.content_cache_misses;
    let inode_hit_ratio = if inode_total == 0 {
        0.0
    } else {
        stats.inode_cache_hits as f64 / inode_total as f64
    };
    let content_hit_ratio = if content_total == 0 {
        0.0
    } else {
        stats.content_cache_hits as f64 / content_total as f64
    };

    let mut out = String::new();
    out.push_str("# HELP semanticfs_query_latency_ms SemanticFS virtual read latency in ms.\n");
    out.push_str("# TYPE semanticfs_query_latency_ms histogram\n");

    let mut cumulative: u64 = 0;
    for (bound, count) in &stats.latency_buckets {
        cumulative += *count;
        out.push_str(&format!(
            "semanticfs_query_latency_ms_bucket{{le=\"{}\"}} {}\n",
            bound, cumulative
        ));
    }
    out.push_str(&format!(
        "semanticfs_query_latency_ms_bucket{{le=\"+Inf\"}} {}\n",
        stats.latency_count
    ));
    out.push_str(&format!(
        "semanticfs_query_latency_ms_sum {}\n",
        stats.latency_sum_ms
    ));
    out.push_str(&format!(
        "semanticfs_query_latency_ms_count {}\n",
        stats.latency_count
    ));

    out.push_str("# TYPE semanticfs_index_lag_ms gauge\n");
    out.push_str("semanticfs_index_lag_ms 0\n");

    out.push_str("# TYPE semanticfs_cache_hit_ratio gauge\n");
    out.push_str(&format!(
        "semanticfs_cache_hit_ratio{{cache=\"inode\"}} {:.6}\n",
        inode_hit_ratio
    ));
    out.push_str(&format!(
        "semanticfs_cache_hit_ratio{{cache=\"content\"}} {:.6}\n",
        content_hit_ratio
    ));

    out.push_str("# TYPE semanticfs_virtual_inode_count gauge\n");
    out.push_str(&format!(
        "semanticfs_virtual_inode_count {}\n",
        inode_entries
    ));

    out.push_str("# TYPE semanticfs_rss_mb gauge\n");
    out.push_str(&format!("semanticfs_rss_mb {}\n", rss_mb));

    out.push_str("# TYPE semanticfs_policy_denies_total counter\n");
    out.push_str(&format!(
        "semanticfs_policy_denies_total {}\n",
        stats.policy_denies_total
    ));

    out.push_str("# TYPE semanticfs_stale_hits_total counter\n");
    out.push_str(&format!(
        "semanticfs_stale_hits_total {}\n",
        stats.stale_hits_total
    ));

    out.push_str("# TYPE semanticfs_read_errors_total counter\n");
    out.push_str(&format!(
        "semanticfs_read_errors_total {}\n",
        stats.read_errors
    ));

    let onnx = onnx_metrics_snapshot();
    out.push_str("# TYPE semanticfs_onnx_requests_total counter\n");
    out.push_str(&format!(
        "semanticfs_onnx_requests_total {}\n",
        onnx.requests_total
    ));
    out.push_str("# TYPE semanticfs_onnx_batches_total counter\n");
    out.push_str(&format!(
        "semanticfs_onnx_batches_total {}\n",
        onnx.batches_total
    ));
    out.push_str("# TYPE semanticfs_onnx_texts_total counter\n");
    out.push_str(&format!(
        "semanticfs_onnx_texts_total {}\n",
        onnx.texts_total
    ));
    out.push_str("# TYPE semanticfs_onnx_failures_total counter\n");
    out.push_str(&format!(
        "semanticfs_onnx_failures_total {}\n",
        onnx.failures_total
    ));
    out.push_str("# TYPE semanticfs_onnx_health_checks_total counter\n");
    out.push_str(&format!(
        "semanticfs_onnx_health_checks_total {}\n",
        onnx.health_checks_total
    ));
    out.push_str("# TYPE semanticfs_onnx_health_check_failures_total counter\n");
    out.push_str(&format!(
        "semanticfs_onnx_health_check_failures_total {}\n",
        onnx.health_check_failures_total
    ));
    out.push_str("# TYPE semanticfs_onnx_queue_depth gauge\n");
    out.push_str(&format!(
        "semanticfs_onnx_queue_depth {}\n",
        onnx.queue_depth_current
    ));
    out.push_str("# TYPE semanticfs_onnx_queue_depth_max gauge\n");
    out.push_str(&format!(
        "semanticfs_onnx_queue_depth_max {}\n",
        onnx.queue_depth_max
    ));
    out.push_str("# TYPE semanticfs_onnx_latency_ms_sum counter\n");
    out.push_str(&format!(
        "semanticfs_onnx_latency_ms_sum {}\n",
        onnx.latency_sum_ms
    ));
    out.push_str("# TYPE semanticfs_onnx_latency_ms_count counter\n");
    out.push_str(&format!(
        "semanticfs_onnx_latency_ms_count {}\n",
        onnx.latency_count
    ));
    out.push_str("# TYPE semanticfs_onnx_latency_ms_max gauge\n");
    out.push_str(&format!(
        "semanticfs_onnx_latency_ms_max {}\n",
        onnx.latency_max_ms
    ));

    (
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        out,
    )
}
