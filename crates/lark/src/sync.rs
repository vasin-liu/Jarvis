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
    parse_fetch_json(&raw, &format!("lark://doc/{token}"), token)
}

pub fn fetch_sheet(
    runner: &dyn CommandRunner,
    cli_bin: &str,
    spreadsheet_token: &str,
) -> Result<Document> {
    let args = [
        "sheets",
        "+read",
        "--spreadsheet-token",
        spreadsheet_token,
        "--format",
        "json",
    ];
    let raw = runner.run(cli_bin, &args)?;
    parse_fetch_json(
        &raw,
        &format!("lark://sheet/{spreadsheet_token}"),
        spreadsheet_token,
    )
}

pub fn fetch_mail(
    runner: &dyn CommandRunner,
    cli_bin: &str,
    message_id: &str,
) -> Result<Document> {
    let args = ["mail", "+get", "--message-id", message_id, "--format", "json"];
    let raw = runner.run(cli_bin, &args)?;
    parse_fetch_json(&raw, &format!("lark://mail/{message_id}"), message_id)
}

pub fn fetch_im_chat(
    runner: &dyn CommandRunner,
    cli_bin: &str,
    chat_id: &str,
) -> Result<Document> {
    let args = ["im", "+history", "--chat-id", chat_id, "--format", "json"];
    let raw = runner.run(cli_bin, &args)?;
    parse_fetch_json(&raw, &format!("lark://im/{chat_id}"), chat_id)
}

fn parse_fetch_json(raw: &str, uri: &str, fallback_id: &str) -> Result<Document> {
    let payload: FetchPayload = serde_json::from_str(raw)?;
    let (title, text) = extract_fields(&payload).unwrap_or_else(|_| {
        (
            format!("lark-{fallback_id}"),
            raw.to_string(),
        )
    });

    if text.trim().is_empty() {
        return Err(LarkError::Empty(fallback_id.to_string()));
    }

    Ok(Document {
        uri: uri.to_string(),
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
    fn parses_sheet_json_response() {
        let runner = FakeRunner::new();
        runner.insert(
            "lark-cli sheets +read --spreadsheet-token sh1 --format json",
            r#"{"title":"Budget","text":"row1,row2"}"#,
        );
        let doc = fetch_sheet(&runner, "lark-cli", "sh1").unwrap();
        assert_eq!(doc.uri, "lark://sheet/sh1");
        assert!(doc.text.contains("row1"));
    }

    #[test]
    fn parses_mail_json_response() {
        let runner = FakeRunner::new();
        runner.insert(
            "lark-cli mail +get --message-id m1 --format json",
            r#"{"title":"Weekly","text":"mail body"}"#,
        );
        let doc = fetch_mail(&runner, "lark-cli", "m1").unwrap();
        assert_eq!(doc.uri, "lark://mail/m1");
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
