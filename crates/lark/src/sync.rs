use ingest::Document;
use ingest::hash_text;
use serde::Deserialize;

use crate::error::{LarkError, Result};
use crate::runner::CommandRunner;

#[derive(Debug, Deserialize)]
struct FetchPayload {
    title: Option<String>,
    #[serde(alias = "markdown", alias = "body", alias = "content")]
    text: Option<String>,
    data: Option<InnerData>,
}

#[derive(Debug, Deserialize)]
struct InnerData {
    title: Option<String>,
    #[serde(alias = "markdown", alias = "body", alias = "content")]
    text: Option<String>,
    document: Option<Box<InnerData>>,
}

pub fn fetch_doc(
    runner: &dyn CommandRunner,
    cli_bin: &str,
    token: &str,
) -> Result<Document> {
    let args = [
        "docs",
        "+fetch",
        "--api-version",
        "v2",
        "--token",
        token,
        "--format",
        "json",
    ];
    let raw = runner.run(cli_bin, &args)?;
    parse_fetch_json(&raw, token)
}

fn parse_fetch_json(raw: &str, token: &str) -> Result<Document> {
    let payload: FetchPayload = serde_json::from_str(raw)?;
    let (title, text) = extract_fields(&payload)?;

    if text.trim().is_empty() {
        return Err(LarkError::Empty(token.to_string()));
    }

    let uri = format!("lark://doc/{token}");
    Ok(Document {
        uri: uri.clone(),
        title,
        text: text.clone(),
        content_hash: hash_text(&text),
    })
}

fn extract_fields(payload: &FetchPayload) -> Result<(String, String)> {
    if let Some(title) = payload.title.clone().filter(|t| !t.is_empty()) {
        if let Some(text) = payload.text.clone().filter(|t| !t.is_empty()) {
            return Ok((title, text));
        }
    }

    if let Some(data) = &payload.data {
        if let Some(pair) = extract_from_inner(data) {
            return Ok(pair);
        }
    }

    Err(LarkError::MissingField(
        "title/text in lark docs +fetch json".into(),
    ))
}

fn extract_from_inner(data: &InnerData) -> Option<(String, String)> {
    if let Some(doc) = &data.document {
        if let Some(pair) = extract_from_inner(doc) {
            return Some(pair);
        }
    }
    let title = data.title.clone().filter(|t| !t.is_empty())?;
    let text = data.text.clone().filter(|t| !t.is_empty())?;
    Some((title, text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::FakeRunner;

    #[test]
    fn parses_flat_json_response() {
        let runner = FakeRunner::new();
        runner.insert(
            "lark-cli docs +fetch --api-version v2 --token tok123 --format json",
            r#"{"title":"Spec","text":"Hello Lark"}"#,
        );

        let doc = fetch_doc(&runner, "lark-cli", "tok123").unwrap();
        assert_eq!(doc.title, "Spec");
        assert_eq!(doc.uri, "lark://doc/tok123");
        assert!(doc.text.contains("Hello Lark"));
    }

    #[test]
    fn parses_nested_json_response() {
        let runner = FakeRunner::new();
        runner.insert(
            "lark-cli docs +fetch --api-version v2 --token abc --format json",
            r#"{"data":{"document":{"title":"Wiki","markdown":"Body"}}}"#,
        );

        let doc = fetch_doc(&runner, "lark-cli", "abc").unwrap();
        assert_eq!(doc.title, "Wiki");
        assert!(doc.text.contains("Body"));
    }
}
