//! Download and configure a real embedding model for SemanticFS.
//!
//! `semanticfs model setup` downloads the bge-small-en-v1.5 ONNX model and
//! tokenizer from Hugging Face and places them in `~/.semanticfs/models/`.
//! After setup, SemanticFS auto-detects the model on startup and uses ONNX
//! embeddings instead of the default hash backend.

use anyhow::{Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Known model configurations.
struct ModelSpec {
    name: &'static str,
    onnx_url: &'static str,
    tokenizer_url: &'static str,
    onnx_filename: &'static str,
}

const MODELS: &[ModelSpec] = &[ModelSpec {
    name: "bge-small-en-v1.5",
    onnx_url: "https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/main/onnx/model_quantized.onnx",
    tokenizer_url: "https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/main/tokenizer.json",
    onnx_filename: "model_quantized.onnx",
}];

pub fn run(model_name: &str, model_dir: &Path) -> Result<()> {
    let spec = MODELS
        .iter()
        .find(|m| m.name == model_name)
        .with_context(|| {
            format!(
                "Unknown model '{}'. Supported: {}",
                model_name,
                MODELS.iter().map(|m| m.name).collect::<Vec<_>>().join(", ")
            )
        })?;

    std::fs::create_dir_all(model_dir)
        .with_context(|| format!("create model dir: {}", model_dir.display()))?;

    let onnx_path = model_dir.join(spec.onnx_filename);
    let tokenizer_path = model_dir.join("tokenizer.json");

    println!("Model directory: {}", model_dir.display());
    println!();

    download_if_missing(&onnx_path, spec.onnx_url, "ONNX model")?;
    download_if_missing(&tokenizer_path, spec.tokenizer_url, "tokenizer")?;

    println!();
    println!("Setup complete.");
    println!();
    println!("SemanticFS will auto-detect the model at:");
    println!("  {}", onnx_path.display());
    println!();
    println!("To use a custom model path, set:");
    println!(
        "  export SEMANTICFS_ONNX_MODEL={}",
        onnx_path.display()
    );
    println!(
        "  export SEMANTICFS_ONNX_TOKENIZER={}",
        tokenizer_path.display()
    );
    println!();
    println!("Then rebuild the index: semanticfs --config <config> index build");

    Ok(())
}

/// Returns the default model directory: `~/.semanticfs/models`.
pub fn default_model_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("SEMANTICFS_HOME") {
        return Some(PathBuf::from(dir).join("models"));
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;
    Some(PathBuf::from(home).join(".semanticfs/models"))
}

fn download_if_missing(dest: &Path, url: &str, label: &str) -> Result<()> {
    if dest.exists() {
        let bytes = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
        println!("  [skip] {} already present ({} bytes)", label, bytes);
        return Ok(());
    }

    print!("  Downloading {} ...", label);
    std::io::stdout().flush().ok();

    let response = ureq::get(url)
        .call()
        .with_context(|| format!("HTTP GET {} failed", url))?;

    let mut data = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut data)
        .with_context(|| format!("reading response body for {}", url))?;

    std::fs::write(dest, &data)
        .with_context(|| format!("writing {} to {}", label, dest.display()))?;

    println!(" {} bytes saved.", data.len());
    Ok(())
}

// Need Read trait for read_to_end
use std::io::Read;
