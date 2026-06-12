use std::path::Path;

use ingest::{hash_text, Document};
use serde::Deserialize;
use serde_json::Value;

use crate::error::{CursorError, Result};

#[derive(Debug, Deserialize)]
struct TranscriptLine {
    role: String,
    message: TranscriptMessage,
}

#[derive(Debug, Deserialize)]
struct TranscriptMessage {
    content: Value,
}

pub fn extract_message_text(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(blocks) => blocks
            .iter()
            .filter_map(|block| {
                let kind = block.get("type")?.as_str()?;
                match kind {
                    "text" => block
                        .get("text")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    "tool_use" => {
                        let name = block
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("tool");
                        Some(format!("[tool: {name}]"))
                    }
                    _ => None,
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

pub fn load_transcript(path: &Path) -> Result<Document> {
    let raw = std::fs::read_to_string(path)?;
    let mut parts = Vec::new();
    let mut first_user_title: Option<String> = None;

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let row: TranscriptLine = serde_json::from_str(line)?;
        let body = extract_message_text(&row.message.content);
        if body.is_empty() {
            continue;
        }
        if first_user_title.is_none() && row.role == "user" {
            first_user_title = Some(
                body.lines()
                    .next()
                    .unwrap_or(&body)
                    .chars()
                    .take(80)
                    .collect(),
            );
        }
        parts.push(format!("[{}]\n{}", row.role, body));
    }

    if parts.is_empty() {
        return Err(CursorError::Invalid(format!(
            "transcript has no indexable content: {}",
            path.display()
        )));
    }

    let session_id = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("session");

    let title = first_user_title
        .filter(|t| !t.is_empty())
        .map(|t| format!("Cursor: {t}"))
        .unwrap_or_else(|| format!("Cursor: {session_id}"));

    let text = parts.join("\n\n");
    let uri = format!("cursor://transcript/{session_id}");
    let content_hash = hash_text(&text);

    Ok(Document {
        uri,
        title,
        text,
        content_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn extracts_text_and_tool_blocks() {
        let content = serde_json::json!([
            {"type": "text", "text": "hello"},
            {"type": "tool_use", "name": "Shell", "input": {}}
        ]);
        let out = extract_message_text(&content);
        assert!(out.contains("hello"));
        assert!(out.contains("[tool: Shell]"));
    }

    #[test]
    fn loads_fixture_transcript() {
        let dir = tempfile::tempdir().unwrap();
        let session = dir.path().join("abc-123");
        fs::create_dir_all(&session).unwrap();
        let file = session.join("abc-123.jsonl");
        fs::write(
            &file,
            r#"{"role":"user","message":{"content":[{"type":"text","text":"How does indexing work?"}]}}
{"role":"assistant","message":{"content":[{"type":"text","text":"Indexing chunks documents."}]}}"#,
        )
        .unwrap();

        let doc = load_transcript(&file).unwrap();
        assert_eq!(doc.uri, "cursor://transcript/abc-123");
        assert!(doc.title.contains("How does indexing"));
        assert!(doc.text.contains("[user]"));
        assert!(doc.text.contains("Indexing chunks"));
        assert_eq!(doc.content_hash.len(), 64);
        let _ = PathBuf::from(file);
    }
}
