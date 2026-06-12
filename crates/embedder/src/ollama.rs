use async_trait::async_trait;
use serde::Deserialize;

use crate::{EmbedError, Embedder, Result};

pub struct OllamaEmbedder {
    base_url: String,
    model: String,
    dim: usize,
    client: reqwest::Client,
}

impl OllamaEmbedder {
    pub fn new(base_url: String, model: String, dim: usize) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
            dim,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

#[async_trait]
impl Embedder for OllamaEmbedder {
    fn id(&self) -> &str {
        "ollama:embed"
    }

    fn dim(&self) -> usize {
        self.dim
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut out = Vec::with_capacity(texts.len());
        for text in texts {
            let url = format!("{}/api/embeddings", self.base_url);
            let resp = self
                .client
                .post(url)
                .json(&serde_json::json!({
                    "model": self.model,
                    "prompt": text,
                }))
                .send()
                .await
                .map_err(|e| EmbedError::Http(e.to_string()))?;

            if !resp.status().is_success() {
                return Err(EmbedError::Http(format!(
                    "ollama embeddings status {}",
                    resp.status()
                )));
            }

            let body: EmbedResponse = resp
                .json()
                .await
                .map_err(|e| EmbedError::BadResponse(e.to_string()))?;

            if body.embedding.len() != self.dim {
                return Err(EmbedError::DimMismatch {
                    expected: self.dim,
                    got: body.embedding.len(),
                });
            }
            out.push(body.embedding);
        }
        Ok(out)
    }
}
