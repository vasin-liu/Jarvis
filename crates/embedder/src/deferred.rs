use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use async_trait::async_trait;

use crate::{EmbedError, Embedder, Result};

enum Slot {
    Pending,
    Ready(Arc<dyn Embedder>),
    Failed(String),
}

/// Placeholder embedder that keeps the Tauri setup thread free while a real
/// provider (typically FastEmbed) loads on a background thread.
pub struct DeferredEmbedder {
    id: String,
    dim: usize,
    pair: Arc<(Mutex<Slot>, Condvar)>,
}

impl DeferredEmbedder {
    pub fn new(id: impl Into<String>, dim: usize) -> Self {
        Self {
            id: id.into(),
            dim,
            pair: Arc::new((Mutex::new(Slot::Pending), Condvar::new())),
        }
    }

    pub fn fulfill(&self, embedder: Arc<dyn Embedder>) {
        let (lock, cv) = &*self.pair;
        let mut slot = lock.lock().unwrap();
        *slot = Slot::Ready(embedder);
        cv.notify_all();
    }

    pub fn fail(&self, err: impl Into<String>) {
        let (lock, cv) = &*self.pair;
        let mut slot = lock.lock().unwrap();
        *slot = Slot::Failed(err.into());
        cv.notify_all();
    }

    fn wait_ready(&self) -> Result<Arc<dyn Embedder>> {
        let (lock, cv) = &*self.pair;
        let mut slot = lock.lock().unwrap();
        loop {
            match &*slot {
                Slot::Ready(e) => return Ok(e.clone()),
                Slot::Failed(msg) => {
                    return Err(EmbedError::Init(format!(
                        "deferred embedder failed: {msg}"
                    )));
                }
                Slot::Pending => {
                    let (next, result) = cv
                        .wait_timeout(slot, Duration::from_secs(300))
                        .unwrap();
                    slot = next;
                    if result.timed_out() {
                        if matches!(&*slot, Slot::Pending) {
                            return Err(EmbedError::Init(
                                "timed out waiting for embedder init".into(),
                            ));
                        }
                    }
                }
            }
        }
    }
}

#[async_trait]
impl Embedder for DeferredEmbedder {
    fn id(&self) -> &str {
        &self.id
    }

    fn dim(&self) -> usize {
        self.dim
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let inner = self.wait_ready()?;
        inner.embed(texts).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockEmbedder;

    #[tokio::test]
    async fn fulfill_unblocks_embed() {
        let deferred = Arc::new(DeferredEmbedder::new("deferred:test", 8));
        let d2 = deferred.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            d2.fulfill(Arc::new(MockEmbedder::new(8)));
        });
        let out = deferred.embed(&[String::from("hi")]).await.unwrap();
        assert_eq!(out[0].len(), 8);
    }

    #[tokio::test]
    async fn fail_surfaces_error() {
        let deferred = Arc::new(DeferredEmbedder::new("deferred:test", 8));
        let d2 = deferred.clone();
        std::thread::spawn(move || {
            d2.fail("boom");
        });
        let err = deferred.embed(&[String::from("hi")]).await.unwrap_err();
        assert!(err.to_string().contains("boom"));
    }
}
