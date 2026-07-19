mod error;
mod summarize;
mod tasks;
mod wiki;

pub use error::{InsightsError, Result};
pub use summarize::summarize_source;
pub use tasks::extract_tasks_from_source;
pub use wiki::{
    analyze_source_for_wiki, render_wiki_pages, WikiAnalysis, WikiCompileResult, WikiConcept,
    WikiEntity, WikiPageDraft, WikiPageType,
};

pub(crate) fn truncate_chars(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let taken: String = text.chars().take(max).collect();
    format!("{taken}\n\n[正文已截断]")
}
