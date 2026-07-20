use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use crate::{EmbedError, Embedder, Result};

/// Known FastEmbed model names → vector dimension.
pub fn fastembed_model_dim(name: &str) -> Option<usize> {
    match name.to_ascii_lowercase().as_str() {
        "bge-small-en-v1.5" | "bge-small-en-v1.5-q" | "all-minilm-l6-v2" => Some(384),
        "bge-small-zh-v1.5" => Some(512),
        "bge-base-en-v1.5" | "bge-base-en-v1.5-q" => Some(768),
        "bge-large-en-v1.5" => Some(1024),
        "multilingual-e5-small" => Some(384),
        "multilingual-e5-base" => Some(768),
        _ => None,
    }
}

fn parse_model(name: &str) -> Result<EmbeddingModel> {
    match name.to_ascii_lowercase().as_str() {
        "bge-small-en-v1.5" => Ok(EmbeddingModel::BGESmallENV15),
        "bge-small-en-v1.5-q" => Ok(EmbeddingModel::BGESmallENV15Q),
        "bge-small-zh-v1.5" => Ok(EmbeddingModel::BGESmallZHV15),
        "bge-base-en-v1.5" => Ok(EmbeddingModel::BGEBaseENV15),
        "bge-base-en-v1.5-q" => Ok(EmbeddingModel::BGEBaseENV15Q),
        "bge-large-en-v1.5" => Ok(EmbeddingModel::BGELargeENV15),
        "multilingual-e5-small" => Ok(EmbeddingModel::MultilingualE5Small),
        "all-minilm-l6-v2" => Ok(EmbeddingModel::AllMiniLML6V2),
        other => Err(EmbedError::FastEmbed(format!("unknown fastembed model: {other}"))),
    }
}

pub struct FastEmbedder {
    inner: Arc<Mutex<TextEmbedding>>,
    id: String,
    dim: usize,
}

impl FastEmbedder {
    /// Build a FastEmbed model. When `cache_dir` is `Some`, model weights are
    /// downloaded to / loaded from that absolute directory; otherwise fastembed
    /// falls back to its default cwd-relative `.fastembed_cache` (avoid in apps,
    /// as it re-downloads whenever the process is launched from a new cwd).
    pub fn try_new(model_name: &str, cache_dir: Option<PathBuf>) -> Result<Self> {
        let model = parse_model(model_name)?;
        let dim = fastembed_model_dim(model_name).ok_or_else(|| {
            EmbedError::FastEmbed(format!("unknown dimension for model: {model_name}"))
        })?;

        let mut opts = InitOptions::new(model).with_show_download_progress(false);
        if let Some(dir) = cache_dir {
            opts = opts.with_cache_dir(dir);
        }
        let text_model = TextEmbedding::try_new(opts)
            .map_err(|e| EmbedError::FastEmbed(e.to_string()))?;

        Ok(Self {
            inner: Arc::new(Mutex::new(text_model)),
            id: format!("fastembed:{model_name}"),
            dim,
        })
    }
}

#[async_trait::async_trait]
impl Embedder for FastEmbedder {
    fn id(&self) -> &str {
        &self.id
    }

    fn dim(&self) -> usize {
        self.dim
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let texts = texts.to_vec();
        let inner = self.inner.clone();
        let dim = self.dim;

        let out = tokio::task::spawn_blocking(move || {
            let mut model = inner
                .lock()
                .map_err(|e| EmbedError::FastEmbed(format!("lock poisoned: {e}")))?;
            let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
            let embeddings = model
                .embed(refs, None)
                .map_err(|e| EmbedError::FastEmbed(e.to_string()))?;

            for (i, vec) in embeddings.iter().enumerate() {
                if vec.len() != dim {
                    return Err(EmbedError::DimMismatch {
                        expected: dim,
                        got: vec.len(),
                    });
                }
                if vec.is_empty() {
                    return Err(EmbedError::FastEmbed(format!(
                        "empty embedding at index {i}"
                    )));
                }
            }
            Ok(embeddings)
        })
        .await
        .map_err(|e| EmbedError::FastEmbed(format!("task join: {e}")))?;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_dim_lookup() {
        assert_eq!(fastembed_model_dim("bge-small-en-v1.5"), Some(384));
        assert_eq!(fastembed_model_dim("bge-small-zh-v1.5"), Some(512));
    }

    #[tokio::test]
    #[ignore = "downloads ONNX model from network"]
    async fn embeds_text_with_fastembed() {
        let embedder = FastEmbedder::try_new("bge-small-en-v1.5", None).unwrap();
        let out = embedder.embed(&["hello world".into()]).await.unwrap();
        assert_eq!(out[0].len(), embedder.dim());
    }
}
