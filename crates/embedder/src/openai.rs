use async_trait::async_trait;

use crate::{EmbedError, Embedder, Result};

pub struct OpenAiEmbedder {
    base_url: String,
    api_key: String,
    model: String,
    dim: usize,
    id: String,
    client: reqwest::Client,
}

impl OpenAiEmbedder {
    pub fn new(base_url: String, api_key: String, model: String, dim: usize) -> Self {
        let id = format!("cloud:embed:{model}");
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model,
            dim,
            id,
            client: reqwest::Client::new(),
        }
    }

    fn embeddings_url(&self) -> String {
        format!("{}/embeddings", self.base_url)
    }
}

#[derive(Debug, serde::Deserialize)]
struct EmbedData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Debug, serde::Deserialize)]
struct EmbedResponse {
    data: Vec<EmbedData>,
}

#[async_trait]
impl Embedder for OpenAiEmbedder {
    fn id(&self) -> &str {
        &self.id
    }

    fn dim(&self) -> usize {
        self.dim
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        if self.api_key.is_empty() {
            return Err(EmbedError::Http("cloud api key is empty".into()));
        }

        let resp = self
            .client
            .post(self.embeddings_url())
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({
                "model": self.model,
                "input": texts,
            }))
            .send()
            .await
            .map_err(|e| EmbedError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(EmbedError::Http(format!(
                "cloud embeddings status {status}: {body}"
            )));
        }

        let body: EmbedResponse = resp
            .json()
            .await
            .map_err(|e| EmbedError::BadResponse(e.to_string()))?;

        if body.data.len() != texts.len() {
            return Err(EmbedError::BadResponse(format!(
                "expected {} embeddings, got {}",
                texts.len(),
                body.data.len()
            )));
        }

        let mut out = vec![Vec::new(); texts.len()];
        for item in body.data {
            if item.index >= texts.len() {
                return Err(EmbedError::BadResponse(format!(
                    "embedding index {} out of range",
                    item.index
                )));
            }
            if item.embedding.len() != self.dim {
                return Err(EmbedError::DimMismatch {
                    expected: self.dim,
                    got: item.embedding.len(),
                });
            }
            out[item.index] = item.embedding;
        }

        Ok(out)
    }
}
