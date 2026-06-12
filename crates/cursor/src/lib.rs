mod discover;
mod error;
mod parse;

pub use discover::discover_transcripts;
pub use error::{CursorError, Result};
pub use parse::{extract_message_text, load_transcript};

use std::path::Path;

/// Map a transcript file path to the stable source id used in the store.
pub fn source_id_for_path(path: &Path) -> Option<String> {
    let session_id = path.parent()?.file_name()?.to_str()?;
    Some(format!("cursor://transcript/{session_id}"))
}

pub fn resolve_transcript_path(projects_root: &Path, source_uri: &str) -> Option<std::path::PathBuf> {
    let session_id = source_uri.strip_prefix("cursor://transcript/")?;
    let found = discover_transcripts(projects_root).ok()?;
    found.into_iter().find(|p| {
        p.parent()
            .and_then(|d| d.file_name())
            .and_then(|n| n.to_str())
            == Some(session_id)
    })
}

pub fn list_transcript_summaries(
    projects_root: &Path,
) -> Result<Vec<TranscriptSummary>> {
    discover_transcripts(projects_root)?
        .into_iter()
        .map(|path| {
            let session_id = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("session")
                .to_string();
            let project = path
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string();
            let uri = format!("cursor://transcript/{session_id}");
            Ok(TranscriptSummary {
                session_id,
                project,
                path: path.to_string_lossy().into_owned(),
                uri,
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TranscriptSummary {
    pub session_id: String,
    pub project: String,
    pub path: String,
    pub uri: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn source_id_from_path() {
        let dir = tempfile::tempdir().unwrap();
        let session = dir.path().join("uuid-1");
        fs::create_dir_all(&session).unwrap();
        let file = session.join("uuid-1.jsonl");
        fs::write(&file, "{}").unwrap();
        assert_eq!(
            source_id_for_path(&file).as_deref(),
            Some("cursor://transcript/uuid-1")
        );
    }
}
