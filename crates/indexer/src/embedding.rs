use semanticfs_common::{embed_text_hash, EmbeddingConfig};

pub struct Embedder {
    backend: Backend,
    dim: usize,
}

enum Backend {
    Hash,
    #[cfg(feature = "onnx")]
    Onnx(OnnxBackend),
}

impl Embedder {
    pub fn from_config(cfg: &EmbeddingConfig) -> Self {
        let dim = cfg.dimension.max(1);
        let requested = cfg.runtime.to_ascii_lowercase();

        if requested == "onnx" {
            #[cfg(feature = "onnx")]
            {
                match OnnxBackend::from_env(dim) {
                    Ok(model) => {
                        tracing::info!("using onnx embedding backend");
                        return Self {
                            backend: Backend::Onnx(model),
                            dim,
                        };
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "failed to initialize onnx backend; falling back to hash backend");
                    }
                }
            }

            #[cfg(not(feature = "onnx"))]
            {
                tracing::warn!("onnx runtime requested but binary was built without `onnx` feature; using hash backend");
            }
        }

        Self {
            backend: Backend::Hash,
            dim,
        }
    }

    pub fn embed(&self, text: &str) -> Vec<f32> {
        let base = embed_text_hash(text, self.dim);
        match &self.backend {
            Backend::Hash => base,
            #[cfg(feature = "onnx")]
            Backend::Onnx(model) => model.project(&base).unwrap_or(base),
        }
    }
}

#[cfg(feature = "onnx")]
struct OnnxBackend {
    session: std::sync::Mutex<ort::session::Session>,
    dim: usize,
}

#[cfg(feature = "onnx")]
impl OnnxBackend {
    fn from_env(dim: usize) -> anyhow::Result<Self> {
        use anyhow::{bail, Context};
        use ort::session::Session;

        let path = std::env::var("SEMANTICFS_ONNX_MODEL")
            .context("SEMANTICFS_ONNX_MODEL is required for onnx backend")?;

        if !std::path::Path::new(&path).exists() {
            bail!("onnx model path does not exist: {}", path);
        }

        let session = Session::builder()?.commit_from_file(path)?;
        Ok(Self {
            session: std::sync::Mutex::new(session),
            dim,
        })
    }

    fn project(&self, base: &[f32]) -> anyhow::Result<Vec<f32>> {
        use anyhow::Context;
        use ort::value::Tensor;

        let mut session = self
            .session
            .lock()
            .map_err(|_| anyhow::anyhow!("onnx session mutex poisoned"))?;
        let outputs = session.run(ort::inputs![Tensor::<f32>::from_array((
            [1, self.dim],
            base.to_vec(),
        ))?])?;

        let first = outputs
            .iter()
            .next()
            .context("onnx model produced no outputs")?;

        let tensor = first.1.try_extract_array::<f32>()?;
        let mut projected = tensor.iter().copied().collect::<Vec<f32>>();
        if projected.len() != self.dim {
            projected.resize(self.dim, 0.0);
        }
        Ok(projected)
    }
}
