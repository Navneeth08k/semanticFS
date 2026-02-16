use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use fuse_bridge::FuseBridge;
use indexer::Indexer;
use mcp::McpServer;
use semanticfs_common::SemanticFsConfig;
use std::{fs, path::PathBuf};
use tracing::info;

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
    Benchmark,
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
}

#[derive(Subcommand, Debug)]
enum ServeCommand {
    Fuse,
    Mcp,
}

#[derive(Subcommand, Debug)]
enum RecoverCommand {
    Mount {
        #[arg(long)]
        force_unmount: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Init(args) => init_command(args, &cli.config),
        Commands::Index { command } => index_command(command, &cli.config),
        Commands::Serve { command } => serve_command(command, &cli.config).await,
        Commands::Health => health_command(&cli.config),
        Commands::Benchmark => benchmark_command(),
        Commands::Recover { command } => recover_command(command),
    }
}

fn init_command(args: InitArgs, target_path: &PathBuf) -> Result<()> {
    let sample = format!(
        "[workspace]\nrepo_root = \"{}\"\nmount_point = \"{}\"\n\n# copy remaining defaults from config/semanticfs.sample.toml\n",
        args.repo, args.mount
    );
    fs::write(target_path, sample)?;
    info!(path = %target_path.display(), "initialized config");
    Ok(())
}

fn index_command(cmd: IndexCommand, config_path: &PathBuf) -> Result<()> {
    let cfg = SemanticFsConfig::load(config_path)
        .with_context(|| format!("load config from {}", config_path.display()))?;

    let db_path = PathBuf::from("semanticfs.db");
    let indexer = Indexer::new(cfg, &db_path)?;

    match cmd {
        IndexCommand::Build => {
            let version = indexer.build_full_index()?;
            println!("index built and published: version={}", version);
        }
        IndexCommand::Watch => {
            println!("starting watch mode; press Ctrl+C to stop");
            indexer.watch()?;
        }
    }
    Ok(())
}

async fn serve_command(cmd: ServeCommand, config_path: &PathBuf) -> Result<()> {
    let cfg = SemanticFsConfig::load(config_path)?;
    let db_path = PathBuf::from("semanticfs.db");
    let bridge = FuseBridge::new(cfg.clone(), &db_path)?;

    match cmd {
        ServeCommand::Fuse => {
            println!("starting fuse mount at {}", cfg.workspace.mount_point);
            bridge.mount()?;
        }
        ServeCommand::Mcp => {
            let bind = cfg.observability.metrics_bind.clone();
            let server = McpServer::new(bridge);
            server.serve(&bind).await?;
        }
    }

    Ok(())
}

fn health_command(config_path: &PathBuf) -> Result<()> {
    let cfg = SemanticFsConfig::load(config_path)?;
    println!("repo_root={}", cfg.workspace.repo_root);
    println!("mount_point={}", cfg.workspace.mount_point);
    println!("status=healthy (static scaffold)");
    Ok(())
}

fn benchmark_command() -> Result<()> {
    println!("benchmark harness placeholder: add fixture task runs here");
    Ok(())
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
