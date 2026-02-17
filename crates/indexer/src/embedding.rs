use anyhow::{anyhow, Context, Result};
use semanticfs_common::{embed_text_hash, EmbeddingConfig};
use serde_json::json;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex, OnceLock,
};
use std::time::Instant;

pub struct Embedder {
    backend: Backend,
    dim: usize,
    batch_size: usize,
}

enum Backend {
    Hash,
    Onnx(OnnxSidecar),
}

pub struct OnnxSidecar {
    process: Mutex<SidecarProcess>,
}

struct SidecarProcess {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

#[derive(Debug, Clone)]
pub struct OnnxMetricsSnapshot {
    pub requests_total: u64,
    pub batches_total: u64,
    pub texts_total: u64,
    pub failures_total: u64,
    pub health_checks_total: u64,
    pub health_check_failures_total: u64,
    pub queue_depth_current: u64,
    pub queue_depth_max: u64,
    pub latency_count: u64,
    pub latency_sum_ms: u64,
    pub latency_max_ms: u64,
}

struct OnnxTelemetry {
    requests_total: AtomicU64,
    batches_total: AtomicU64,
    texts_total: AtomicU64,
    failures_total: AtomicU64,
    health_checks_total: AtomicU64,
    health_check_failures_total: AtomicU64,
    queue_depth_current: AtomicU64,
    queue_depth_max: AtomicU64,
    latency_count: AtomicU64,
    latency_sum_ms: AtomicU64,
    latency_max_ms: AtomicU64,
}

impl OnnxTelemetry {
    fn new() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            batches_total: AtomicU64::new(0),
            texts_total: AtomicU64::new(0),
            failures_total: AtomicU64::new(0),
            health_checks_total: AtomicU64::new(0),
            health_check_failures_total: AtomicU64::new(0),
            queue_depth_current: AtomicU64::new(0),
            queue_depth_max: AtomicU64::new(0),
            latency_count: AtomicU64::new(0),
            latency_sum_ms: AtomicU64::new(0),
            latency_max_ms: AtomicU64::new(0),
        }
    }

    fn snapshot(&self) -> OnnxMetricsSnapshot {
        OnnxMetricsSnapshot {
            requests_total: self.requests_total.load(Ordering::Relaxed),
            batches_total: self.batches_total.load(Ordering::Relaxed),
            texts_total: self.texts_total.load(Ordering::Relaxed),
            failures_total: self.failures_total.load(Ordering::Relaxed),
            health_checks_total: self.health_checks_total.load(Ordering::Relaxed),
            health_check_failures_total: self.health_check_failures_total.load(Ordering::Relaxed),
            queue_depth_current: self.queue_depth_current.load(Ordering::Relaxed),
            queue_depth_max: self.queue_depth_max.load(Ordering::Relaxed),
            latency_count: self.latency_count.load(Ordering::Relaxed),
            latency_sum_ms: self.latency_sum_ms.load(Ordering::Relaxed),
            latency_max_ms: self.latency_max_ms.load(Ordering::Relaxed),
        }
    }

    fn reset(&self) {
        self.requests_total.store(0, Ordering::Relaxed);
        self.batches_total.store(0, Ordering::Relaxed);
        self.texts_total.store(0, Ordering::Relaxed);
        self.failures_total.store(0, Ordering::Relaxed);
        self.health_checks_total.store(0, Ordering::Relaxed);
        self.health_check_failures_total.store(0, Ordering::Relaxed);
        self.queue_depth_current.store(0, Ordering::Relaxed);
        self.queue_depth_max.store(0, Ordering::Relaxed);
        self.latency_count.store(0, Ordering::Relaxed);
        self.latency_sum_ms.store(0, Ordering::Relaxed);
        self.latency_max_ms.store(0, Ordering::Relaxed);
    }

    fn observe_latency(&self, elapsed_ms: u64) {
        self.latency_count.fetch_add(1, Ordering::Relaxed);
        self.latency_sum_ms.fetch_add(elapsed_ms, Ordering::Relaxed);
        update_atomic_max(&self.latency_max_ms, elapsed_ms);
    }

    fn enter_queue(&self) -> QueueDepthGuard<'_> {
        let depth = self.queue_depth_current.fetch_add(1, Ordering::Relaxed) + 1;
        update_atomic_max(&self.queue_depth_max, depth);
        QueueDepthGuard { telemetry: self }
    }
}

struct QueueDepthGuard<'a> {
    telemetry: &'a OnnxTelemetry,
}

impl Drop for QueueDepthGuard<'_> {
    fn drop(&mut self) {
        self.telemetry
            .queue_depth_current
            .fetch_sub(1, Ordering::Relaxed);
    }
}

static ONNX_TELEMETRY: OnceLock<OnnxTelemetry> = OnceLock::new();

fn onnx_telemetry() -> &'static OnnxTelemetry {
    ONNX_TELEMETRY.get_or_init(OnnxTelemetry::new)
}

pub fn onnx_metrics_snapshot() -> OnnxMetricsSnapshot {
    onnx_telemetry().snapshot()
}

pub fn reset_onnx_metrics() {
    onnx_telemetry().reset();
}

impl Embedder {
    pub fn from_config(cfg: &EmbeddingConfig) -> Result<Self> {
        let dim = cfg.dimension.max(1);
        let requested = cfg.runtime.to_ascii_lowercase();

        if requested == "onnx" {
            let sidecar = OnnxSidecar::from_env(dim)?;
            sidecar.health_check()?;
            tracing::info!("using onnx sidecar embedding backend");
            return Ok(Self {
                backend: Backend::Onnx(sidecar),
                dim,
                batch_size: cfg.batch_size.max(1),
            });
        }

        Ok(Self {
            backend: Backend::Hash,
            dim,
            batch_size: cfg.batch_size.max(1),
        })
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match &self.backend {
            Backend::Hash => Ok(embed_text_hash(text, self.dim)),
            Backend::Onnx(sidecar) => sidecar.embed(text),
        }
    }

    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        match &self.backend {
            Backend::Hash => Ok(texts
                .iter()
                .map(|t| embed_text_hash(t, self.dim))
                .collect::<Vec<_>>()),
            Backend::Onnx(sidecar) => {
                let mut out = Vec::with_capacity(texts.len());
                for chunk in texts.chunks(self.batch_size) {
                    let part = sidecar.embed_many(chunk)?;
                    out.extend(part);
                }
                Ok(out)
            }
        }
    }
}

impl OnnxSidecar {
    fn from_env(dim: usize) -> Result<Self> {
        let model_path = std::env::var("SEMANTICFS_ONNX_MODEL")
            .context("SEMANTICFS_ONNX_MODEL is required when embedding.runtime=onnx")?;
        if !Path::new(&model_path).exists() {
            return Err(anyhow!("onnx model path does not exist: {}", model_path));
        }

        let tokenizer_path = std::env::var("SEMANTICFS_ONNX_TOKENIZER")
            .unwrap_or_else(|_| infer_tokenizer_path(&model_path).display().to_string());

        let python_bin =
            std::env::var("SEMANTICFS_ONNX_PYTHON").unwrap_or_else(|_| "python".to_string());
        let script_path = std::env::var("SEMANTICFS_ONNX_SCRIPT")
            .unwrap_or_else(|_| "scripts/onnx_embed.py".to_string());
        if !Path::new(&script_path).exists() {
            return Err(anyhow!(
                "onnx sidecar script not found: {} (set SEMANTICFS_ONNX_SCRIPT)",
                script_path
            ));
        }

        let max_length = std::env::var("SEMANTICFS_ONNX_MAX_LENGTH")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(512);
        let provider = std::env::var("SEMANTICFS_ONNX_PROVIDER")
            .unwrap_or_else(|_| "CPUExecutionProvider".to_string());
        let intra_threads = std::env::var("SEMANTICFS_ONNX_INTRA_THREADS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);
        let inter_threads = std::env::var("SEMANTICFS_ONNX_INTER_THREADS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let mut child = Command::new(python_bin)
            .arg(script_path)
            .arg("--serve")
            .arg("--model")
            .arg(model_path)
            .arg("--tokenizer")
            .arg(tokenizer_path)
            .arg("--dim")
            .arg(dim.to_string())
            .arg("--max-length")
            .arg(max_length.to_string())
            .arg("--provider")
            .arg(provider)
            .arg("--intra-threads")
            .arg(intra_threads.to_string())
            .arg("--inter-threads")
            .arg(inter_threads.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("failed to spawn onnx sidecar process")?;

        let stdin = child
            .stdin
            .take()
            .context("onnx sidecar stdin unavailable")?;
        let stdout = child
            .stdout
            .take()
            .context("onnx sidecar stdout unavailable")?;

        Ok(Self {
            process: Mutex::new(SidecarProcess {
                child,
                stdin: BufWriter::new(stdin),
                stdout: BufReader::new(stdout),
            }),
        })
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let mut many = self.embed_many(&[text.to_string()])?;
        many.pop()
            .ok_or_else(|| anyhow!("onnx sidecar returned no embedding for single input"))
    }

    fn embed_many(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let telemetry = onnx_telemetry();
        telemetry.requests_total.fetch_add(1, Ordering::Relaxed);
        telemetry.batches_total.fetch_add(1, Ordering::Relaxed);
        telemetry
            .texts_total
            .fetch_add(texts.len() as u64, Ordering::Relaxed);

        let _queue_guard = telemetry.enter_queue();
        let started = Instant::now();

        let result = (|| {
            let mut proc = self
                .process
                .lock()
                .map_err(|_| anyhow!("onnx sidecar mutex poisoned"))?;

            ensure_child_alive(&mut proc.child)?;

            let req = json!({ "texts": texts });
            let line = serde_json::to_string(&req)?;
            proc.stdin.write_all(line.as_bytes())?;
            proc.stdin.write_all(b"\n")?;
            proc.stdin.flush()?;

            let mut out_line = String::new();
            let read = proc.stdout.read_line(&mut out_line)?;
            if read == 0 {
                return Err(anyhow!("onnx sidecar closed stdout"));
            }

            let value: serde_json::Value = serde_json::from_str(out_line.trim())?;
            if let Some(err) = value.get("error").and_then(|v| v.as_str()) {
                return Err(anyhow!("onnx sidecar error: {}", err));
            }

            let embeddings = value
                .get("embeddings")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow!("onnx sidecar response missing embeddings"))?;

            let mut out = Vec::with_capacity(embeddings.len());
            for emb in embeddings {
                let v = emb
                    .as_array()
                    .ok_or_else(|| anyhow!("onnx sidecar returned malformed embedding vector"))?
                    .iter()
                    .filter_map(|x| x.as_f64())
                    .map(|x| x as f32)
                    .collect::<Vec<_>>();
                if v.is_empty() {
                    return Err(anyhow!("onnx sidecar returned empty embedding"));
                }
                out.push(v);
            }

            if out.len() != texts.len() {
                return Err(anyhow!(
                    "onnx sidecar returned mismatched embedding count: expected {}, got {}",
                    texts.len(),
                    out.len()
                ));
            }

            Ok(out)
        })();

        let elapsed_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        telemetry.observe_latency(elapsed_ms);

        if result.is_err() {
            telemetry.failures_total.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    fn health_check(&self) -> Result<()> {
        let telemetry = onnx_telemetry();
        telemetry
            .health_checks_total
            .fetch_add(1, Ordering::Relaxed);

        let result = (|| {
            let mut proc = self
                .process
                .lock()
                .map_err(|_| anyhow!("onnx sidecar mutex poisoned"))?;

            ensure_child_alive(&mut proc.child)?;

            let req = json!({ "op": "ping" });
            let line = serde_json::to_string(&req)?;
            proc.stdin.write_all(line.as_bytes())?;
            proc.stdin.write_all(b"\n")?;
            proc.stdin.flush()?;

            let mut out_line = String::new();
            let read = proc.stdout.read_line(&mut out_line)?;
            if read == 0 {
                return Err(anyhow!("onnx sidecar closed stdout during health check"));
            }
            let value: serde_json::Value = serde_json::from_str(out_line.trim())?;
            if value.get("ok").and_then(|v| v.as_bool()) != Some(true) {
                return Err(anyhow!("onnx sidecar ping failed: {}", value));
            }
            Ok(())
        })();

        if result.is_err() {
            telemetry
                .health_check_failures_total
                .fetch_add(1, Ordering::Relaxed);
        }

        result
    }
}

fn infer_tokenizer_path(model_path: &str) -> PathBuf {
    let p = Path::new(model_path);
    p.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
}

fn ensure_child_alive(child: &mut Child) -> Result<()> {
    if let Some(status) = child.try_wait()? {
        return Err(anyhow!("onnx sidecar exited unexpectedly: {}", status));
    }
    Ok(())
}

fn update_atomic_max(atom: &AtomicU64, value: u64) {
    let mut current = atom.load(Ordering::Relaxed);
    while value > current {
        match atom.compare_exchange(current, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(next) => current = next,
        }
    }
}
