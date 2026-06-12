use std::path::{Path, PathBuf};

use crate::error::Result;

/// Discover Cursor agent transcript JSONL files under `{root}/{project}/agent-transcripts/{id}/{id}.jsonl`.
pub fn discover_transcripts(projects_root: &Path) -> Result<Vec<PathBuf>> {
    if !projects_root.is_dir() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    for project_entry in std::fs::read_dir(projects_root)? {
        let project_entry = project_entry?;
        let transcripts_dir = project_entry.path().join("agent-transcripts");
        if !transcripts_dir.is_dir() {
            continue;
        }

        for session_entry in std::fs::read_dir(&transcripts_dir)? {
            let session_entry = session_entry?;
            let session_dir = session_entry.path();
            if !session_dir.is_dir() {
                continue;
            }
            let Some(session_id) = session_dir.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let file = session_dir.join(format!("{session_id}.jsonl"));
            if file.is_file() {
                out.push(file);
            }
        }
    }

    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_transcript(root: &Path, project: &str, session_id: &str) -> PathBuf {
        let dir = root
            .join(project)
            .join("agent-transcripts")
            .join(session_id);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join(format!("{session_id}.jsonl"));
        fs::write(
            &file,
            r#"{"role":"user","message":{"content":"ping"}}"#,
        )
        .unwrap();
        file
    }

    #[test]
    fn discovers_nested_transcripts() {
        let root = tempfile::tempdir().unwrap();
        let a = write_transcript(root.path(), "proj-a", "sess-a");
        let b = write_transcript(root.path(), "proj-b", "sess-b");

        let found = discover_transcripts(root.path()).unwrap();
        assert_eq!(found.len(), 2);
        assert!(found.contains(&a));
        assert!(found.contains(&b));
    }

    #[test]
    fn empty_when_root_missing() {
        let found = discover_transcripts(Path::new("/nonexistent/jarvis-cursor-root")).unwrap();
        assert!(found.is_empty());
    }
}
