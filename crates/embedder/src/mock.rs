use sha2::{Digest, Sha256};

use async_trait::async_trait;

use crate::{Embedder, Result};

/// Deterministic embedder for tests: same text always yields the same vector.
pub struct MockEmbedder {
    dim: usize,
}

impl MockEmbedder {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    fn embed_one(&self, text: &str) -> Vec<f32> {
        let mut out = vec![0.0f32; self.dim];
        let digest = Sha256::digest(text.as_bytes());
        for (i, slot) in out.iter_mut().enumerate() {
            let byte = digest[i % digest.len()];
            *slot = (byte as f32) / 255.0;
        }
        let norm: f32 = out.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut out {
                *v /= norm;
            }
        }
        out
    }
}

#[async_trait]
impl Embedder for MockEmbedder {
    fn id(&self) -> &str {
        "mock:v1"
    }

    fn dim(&self) -> usize {
        self.dim
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|t| self.embed_one(t)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_is_deterministic() {
        let e = MockEmbedder::new(4);
        let a = e.embed(&["hello".into()]).await.unwrap();
        let b = e.embed(&["hello".into()]).await.unwrap();
        assert_eq!(a, b);
        assert_eq!(a[0].len(), 4);
    }

    #[tokio::test]
    async fn different_texts_differ() {
        let e = MockEmbedder::new(4);
        let a = e.embed(&["alpha".into()]).await.unwrap();
        let b = e.embed(&["beta".into()]).await.unwrap();
        assert_ne!(a[0], b[0]);
    }

    #[test]
    fn reports_dim_via_trait() {
        let e = MockEmbedder::new(4);
        assert_eq!(e.dim(), 4);
    }
}
