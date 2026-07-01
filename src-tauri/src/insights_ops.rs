use config::AppConfig;
use insights::{extract_tasks_from_source, summarize_source};
use llm::ChatModel;
use store::{IndexStatus, Store};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightsReport {
    pub summarized: usize,
    pub tasks_extracted: usize,
    pub failed: usize,
}

pub async fn maybe_run_insights_for_source(
    store: &Store,
    chat: &dyn ChatModel,
    cfg: &AppConfig,
    source_id: &str,
) {
    if cfg.sync.auto_summarize_on_index {
        let _ = summarize_source(store, chat, source_id).await;
    }
    if cfg.sync.auto_extract_tasks_on_index {
        let _ = extract_tasks_from_source(store, chat, source_id).await;
    }
}

pub async fn run_insights_for_all_indexed(
    store: &Store,
    chat: &dyn ChatModel,
    summarize: bool,
    extract_tasks: bool,
) -> Result<InsightsReport, store::StoreError> {
    let mut summarized = 0;
    let mut tasks_extracted = 0;
    let mut failed = 0;

    let sources: Vec<_> = store
        .list_sources()?
        .into_iter()
        .filter(|s| s.status == IndexStatus::Indexed)
        .map(|s| s.id)
        .collect();

    for source_id in sources {
        if summarize {
            match summarize_source(store, chat, &source_id).await {
                Ok(_) => summarized += 1,
                Err(_) => failed += 1,
            }
        }
        if extract_tasks {
            match extract_tasks_from_source(store, chat, &source_id).await {
                Ok(tasks) => tasks_extracted += tasks.len(),
                Err(_) => failed += 1,
            }
        }
    }

    Ok(InsightsReport {
        summarized,
        tasks_extracted,
        failed,
    })
}
