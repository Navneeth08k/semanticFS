//! Smart project initialization for SemanticFS.
//!
//! `semanticfs init` auto-detects the git root, infers the project type, and
//! writes a ready-to-use `semanticfs.toml` config with appropriate deny_globs.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Profile templates (embedded at compile time from config/profiles/).
static PROFILE_SINGLE_REPO: &str =
    include_str!("../../../config/profiles/single-repo.sample.toml");
static PROFILE_MULTI_ROOT: &str =
    include_str!("../../../config/profiles/multi-root-dev-box.sample.toml");

/// Detected project type — drives which deny_globs to inject.
#[derive(Debug, PartialEq)]
enum ProjectType {
    Rust,
    Node,
    Python,
    Generic,
}

pub struct InitOptions {
    /// Project root directory (defaults to cwd; git root is derived from this).
    pub root: PathBuf,
    /// Profile: "single-repo" or "multi-root".
    pub profile: String,
    /// Output config file path.
    pub output: PathBuf,
}

pub fn run(opts: InitOptions) -> Result<()> {
    let git_root = detect_git_root(&opts.root).unwrap_or_else(|_| opts.root.clone());
    let project_type = detect_project_type(&git_root);
    let extra_deny = project_extra_deny_globs(&project_type);

    // Normalize the path to forward slashes (TOML string value)
    let root_str = git_root
        .to_string_lossy()
        .replace('\\', "/");

    // Choose template
    let template = match opts.profile.as_str() {
        "multi-root" => PROFILE_MULTI_ROOT,
        _ => PROFILE_SINGLE_REPO,
    };

    // Default mount point (FUSE on Linux; placeholder elsewhere)
    let mount_point = if cfg!(target_os = "linux") {
        "/mnt/ai".to_string()
    } else {
        "/mnt/ai".to_string() // placeholder — FUSE is Linux-only
    };

    let mut config = template
        .replace("__REPO_ROOT__", &root_str)
        .replace("__MOUNT_POINT__", &mount_point)
        .replace("__HOME_ROOT__", &root_str)
        .replace("__PROJECTS_ROOT__", &format!("{root_str}/projects"));

    // Inject extra deny_globs for the detected project type
    if !extra_deny.is_empty() {
        config = inject_deny_globs(&config, &extra_deny);
    }

    std::fs::write(&opts.output, &config)
        .with_context(|| format!("writing config to {}", opts.output.display()))?;

    println!("Config written to: {}", opts.output.display());
    println!("Detected project type: {:?}", project_type);
    println!("Repo root: {}", root_str);
    println!();
    println!("Next steps:");
    println!("  1. Review: {}", opts.output.display());
    println!(
        "  2. Build index: semanticfs --config {} index build",
        opts.output.display()
    );
    println!(
        "  3. Start server: semanticfs --config {} serve mcp-stdio",
        opts.output.display()
    );

    Ok(())
}

fn detect_git_root(start: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(start)
        .output()
        .context("running git rev-parse --show-toplevel")?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(path))
    } else {
        // Not a git repo — use the provided directory as-is
        Ok(start.to_path_buf())
    }
}

fn detect_project_type(root: &Path) -> ProjectType {
    if root.join("Cargo.toml").exists() {
        return ProjectType::Rust;
    }
    if root.join("package.json").exists() || root.join("node_modules").exists() {
        return ProjectType::Node;
    }
    if root.join("pyproject.toml").exists()
        || root.join("setup.py").exists()
        || root.join("requirements.txt").exists()
    {
        return ProjectType::Python;
    }
    ProjectType::Generic
}

fn project_extra_deny_globs(pt: &ProjectType) -> Vec<&'static str> {
    match pt {
        ProjectType::Rust => vec!["**/target/**"],
        ProjectType::Node => vec!["**/node_modules/**", "**/.next/**", "**/dist/**"],
        ProjectType::Python => vec!["**/.venv/**", "**/venv/**", "**/__pycache__/**", "**/*.pyc"],
        ProjectType::Generic => vec![],
    }
}

/// Append any globs not already present in the [filter] deny_globs list.
fn inject_deny_globs(config: &str, extra: &[&str]) -> String {
    // Simple approach: append extra globs inside the deny_globs array
    // We look for the closing ']' of the deny_globs value and insert before it.
    // This is safe for well-formed TOML with the array on multiple lines.
    let marker = "**/*.mp4\"";
    if let Some(pos) = config.find(marker) {
        let end = pos + marker.len();
        let new_globs: String = extra
            .iter()
            .filter(|g| !config.contains(*g))
            .map(|g| format!(",\n  \"{g}\""))
            .collect();
        if new_globs.is_empty() {
            return config.to_string();
        }
        let mut out = config.to_string();
        out.insert_str(end, &new_globs);
        return out;
    }
    config.to_string()
}
