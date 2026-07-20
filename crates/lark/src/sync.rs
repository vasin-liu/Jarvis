use ingest::Document;
use ingest::hash_text;

use crate::error::{LarkError, Result};
use crate::options::LarkCliOptions;
use crate::parse::{extract_title_and_text, normalize_lark_ref, parse_cli_json};
use crate::runner::CommandRunner;

pub fn fetch_doc(
    runner: &dyn CommandRunner,
    opts: &LarkCliOptions<'_>,
    doc_ref: &str,
) -> Result<Document> {
    let doc_ref = normalize_lark_ref(doc_ref);
    let mut args = vec![
        "docs".into(),
        "+fetch".into(),
        "--api-version".into(),
        "v2".into(),
        "--doc".into(),
        doc_ref.clone(),
        "--doc-format".into(),
        "markdown".into(),
        "--format".into(),
        "json".into(),
    ];
    opts.push_identity(&mut args);
    let raw = runner.run(opts.bin, &args.iter().map(String::as_str).collect::<Vec<_>>())?;
    let id = doc_id_from_ref(&doc_ref);
    parse_fetch_json(&raw, &format!("lark://doc/{id}"), &id)
}

pub fn fetch_sheet(
    runner: &dyn CommandRunner,
    opts: &LarkCliOptions<'_>,
    spreadsheet_ref: &str,
) -> Result<Document> {
    let spreadsheet_ref = normalize_lark_ref(spreadsheet_ref);
    let mut args = vec!["sheets".into(), "+read".into(), "--format".into(), "json".into()];
    if spreadsheet_ref.starts_with("http://") || spreadsheet_ref.starts_with("https://") {
        args.push("--url".into());
        args.push(spreadsheet_ref.clone());
    } else {
        args.push("--spreadsheet-token".into());
        args.push(spreadsheet_ref.clone());
    }
    opts.push_identity(&mut args);
    let raw = runner.run(opts.bin, &args.iter().map(String::as_str).collect::<Vec<_>>())?;
    let id = sheet_id_from_ref(&spreadsheet_ref);
    parse_fetch_json(&raw, &format!("lark://sheet/{id}"), &id)
}

pub fn fetch_mail(
    runner: &dyn CommandRunner,
    opts: &LarkCliOptions<'_>,
    message_id: &str,
) -> Result<Document> {
    let message_id = normalize_lark_ref(message_id);
    let mut args = vec![
        "mail".into(),
        "+message".into(),
        "--message-id".into(),
        message_id.clone(),
        "--html".into(),
        "false".into(),
        "--format".into(),
        "json".into(),
    ];
    opts.push_identity(&mut args);
    let raw = runner.run(opts.bin, &args.iter().map(String::as_str).collect::<Vec<_>>())?;
    parse_fetch_json(&raw, &format!("lark://mail/{message_id}"), &message_id)
}

pub fn fetch_im_chat(
    runner: &dyn CommandRunner,
    opts: &LarkCliOptions<'_>,
    chat_ref: &str,
) -> Result<Document> {
    let chat_ref = normalize_lark_ref(chat_ref);
    let mut args = vec![
        "im".into(),
        "+chat-messages-list".into(),
        "--sort".into(),
        "asc".into(),
        "--page-size".into(),
        "50".into(),
        "--format".into(),
        "json".into(),
    ];
    if chat_ref.starts_with("ou_") {
        args.push("--user-id".into());
    } else {
        args.push("--chat-id".into());
    }
    args.push(chat_ref.clone());
    opts.push_identity(&mut args);
    let raw = runner.run(opts.bin, &args.iter().map(String::as_str).collect::<Vec<_>>())?;
    parse_fetch_json(&raw, &format!("lark://im/{chat_ref}"), &chat_ref)
}

#[derive(Debug, Clone)]
struct InspectInfo {
    title: String,
    token: String,
    doc_type: String,
    url: String,
}

/// Resolve a Feishu URL (or token) via drive +inspect, then fetch content.
pub fn fetch_from_url(
    runner: &dyn CommandRunner,
    opts: &LarkCliOptions<'_>,
    url_or_ref: &str,
) -> Result<Document> {
    let trimmed = normalize_lark_ref(url_or_ref);
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        let info = inspect_url(runner, opts, &trimmed)?;
        return fetch_inspected(runner, opts, &info);
    }
    fetch_doc(runner, opts, &trimmed)
}

fn inspect_url(
    runner: &dyn CommandRunner,
    opts: &LarkCliOptions<'_>,
    url: &str,
) -> Result<InspectInfo> {
    let clean = crate::parse::strip_url_query(url);
    let mut args = vec![
        "drive".into(),
        "+inspect".into(),
        "--url".into(),
        clean,
        "--format".into(),
        "json".into(),
    ];
    opts.push_identity(&mut args);
    let raw = runner.run(opts.bin, &args.iter().map(String::as_str).collect::<Vec<_>>())?;
    let value = parse_cli_json(&raw)?;
    let data = value.get("data").ok_or_else(|| {
        LarkError::MissingField("data in drive +inspect response".into())
    })?;
    Ok(InspectInfo {
        title: data
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("lark-file")
            .to_string(),
        token: data
            .get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LarkError::MissingField("token in drive +inspect".into()))?
            .to_string(),
        doc_type: data
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("file")
            .to_string(),
        url: data
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or(url)
            .to_string(),
    })
}

fn fetch_inspected(
    runner: &dyn CommandRunner,
    opts: &LarkCliOptions<'_>,
    info: &InspectInfo,
) -> Result<Document> {
    match info.doc_type.as_str() {
        "doc" | "docx" | "wiki" => fetch_doc(runner, opts, &info.url),
        "sheet" | "bitable" => fetch_sheet(runner, opts, &info.url),
        "file" => fetch_drive_file(runner, opts, info),
        other => Err(LarkError::Command(format!(
            "unsupported feishu resource type: {other}"
        ))),
    }
}

const DRIVE_CACHE_DIR: &str = ".jarvis-lark-cache";

fn fetch_drive_file(
    runner: &dyn CommandRunner,
    opts: &LarkCliOptions<'_>,
    info: &InspectInfo,
) -> Result<Document> {
    std::fs::create_dir_all(DRIVE_CACHE_DIR).map_err(|e| LarkError::Command(e.to_string()))?;
    let filename = cache_filename(&info.title, &info.token);
    let output = format!("{DRIVE_CACHE_DIR}/{filename}");
    let mut args = vec![
        "drive".into(),
        "+download".into(),
        "--file-token".into(),
        info.token.clone(),
        "--output".into(),
        output.clone(),
        "--overwrite".into(),
    ];
    opts.push_identity(&mut args);
    runner.run_in(
        opts.bin,
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
        Some(std::path::Path::new(".")),
    )?;

    let mut doc = ingest::load_path(&output).map_err(|e| LarkError::Command(e.to_string()))?;
    doc.uri = format!("lark://file/{}", info.token);
    doc.title = info.title.clone();
    doc.content_hash = hash_text(&doc.text);
    Ok(doc)
}

fn cache_filename(title: &str, token: &str) -> String {
    let ext = std::path::Path::new(title)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    format!("{token}.{ext}")
}

fn parse_fetch_json(raw: &str, uri: &str, fallback_id: &str) -> Result<Document> {
    let value = parse_cli_json(raw)?;
    let (title, text) = extract_title_and_text(&value, fallback_id)?;

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

fn doc_id_from_ref(doc_ref: &str) -> String {
    if let Some(token) = last_path_segment(doc_ref) {
        return token.to_string();
    }
    doc_ref.to_string()
}

fn sheet_id_from_ref(sheet_ref: &str) -> String {
    if let Some(token) = last_path_segment(sheet_ref) {
        return token.to_string();
    }
    sheet_ref.to_string()
}

fn last_path_segment(input: &str) -> Option<&str> {
    let trimmed = input.trim_end_matches('/');
    trimmed.rsplit('/').next().filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{LarkCliOptions, LarkIdentity};
    use crate::runner::FakeRunner;

    fn opts() -> LarkCliOptions<'static> {
        LarkCliOptions::new("lark-cli", LarkIdentity::User)
    }

    #[test]
    fn parses_doc_v2_command_and_response() {
        let runner = FakeRunner::new();
        runner.insert(
            "lark-cli docs +fetch --api-version v2 --doc tok123 --doc-format markdown --format json --as user",
            r#"{"ok":true,"data":{"document":{"document_id":"tok123","content":"<title>Spec</title>\n\nHello Lark"}}}"#,
        );

        let doc = fetch_doc(&runner, &opts(), "tok123").unwrap();
        assert_eq!(doc.title, "Spec");
        assert_eq!(doc.uri, "lark://doc/tok123");
        assert!(doc.text.contains("Hello Lark"));
    }

    #[test]
    fn parses_sheet_json_response() {
        let runner = FakeRunner::new();
        runner.insert(
            "lark-cli sheets +read --format json --spreadsheet-token sh1 --as user",
            r#"{"ok":true,"data":{"title":"Budget","text":"row1,row2"}}"#,
        );
        let doc = fetch_sheet(&runner, &opts(), "sh1").unwrap();
        assert_eq!(doc.uri, "lark://sheet/sh1");
        assert!(doc.text.contains("row1"));
    }

    #[test]
    fn parses_mail_json_response() {
        let runner = FakeRunner::new();
        runner.insert(
            "lark-cli mail +message --message-id m1 --html false --format json --as user",
            r#"{"ok":true,"data":{"subject":"Weekly","body_plain_text":"mail body"}}"#,
        );
        let doc = fetch_mail(&runner, &opts(), "m1").unwrap();
        assert_eq!(doc.uri, "lark://mail/m1");
        assert_eq!(doc.title, "Weekly");
    }

    #[test]
    fn parses_im_messages_response() {
        let runner = FakeRunner::new();
        runner.insert(
            "lark-cli im +chat-messages-list --sort asc --page-size 50 --format json --chat-id oc_1 --as user",
            r#"{"ok":true,"data":{"chat_id":"oc_1","messages":[{"sender_name":"Bob","content":"hello"}]}}"#,
        );
        let doc = fetch_im_chat(&runner, &opts(), "oc_1").unwrap();
        assert_eq!(doc.uri, "lark://im/oc_1");
        assert!(doc.text.contains("Bob"));
    }

    #[test]
    #[ignore = "requires live lark-cli auth"]
    fn live_fetch_drive_xls() {
        let runner = crate::runner::ProcessRunner;
        let opts = opts();
        let url = "https://icnvvlct5jin.feishu.cn/file/YFHgbUMzKo63gYxSFYTcqxjOn5d";
        let doc = fetch_from_url(&runner, &opts, url).unwrap();
        assert!(doc.uri.starts_with("lark://file/"));
        assert!(!doc.text.trim().is_empty());
    }
}
