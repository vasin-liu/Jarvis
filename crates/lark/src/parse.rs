use serde_json::Value;

use crate::error::{LarkError, Result};

/// Normalize user input: accept raw token/ID or full Feishu URL.
pub fn normalize_lark_ref(input: &str) -> String {
    input.trim().to_string()
}

pub fn strip_url_query(url: &str) -> String {
    url.split('?').next().unwrap_or(url).trim().to_string()
}

pub fn parse_cli_json(raw: &str) -> Result<Value> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(LarkError::Command("empty lark-cli response".into()));
    }
    let value: Value = serde_json::from_str(trimmed)?;
    if value.get("ok") == Some(&Value::Bool(false)) {
        let message = value
            .pointer("/error/message")
            .and_then(Value::as_str)
            .or_else(|| value.get("message").and_then(Value::as_str))
            .unwrap_or("lark-cli returned ok=false");
        let hint = value
            .pointer("/error/hint")
            .and_then(Value::as_str)
            .unwrap_or("");
        let detail = if hint.is_empty() {
            message.to_string()
        } else {
            format!("{message} ({hint})")
        };
        return Err(LarkError::Command(detail));
    }
    Ok(value)
}

pub fn extract_title_and_text(value: &Value, fallback_id: &str) -> Result<(String, String)> {
    let data = value.get("data").unwrap_or(value);

    if let Some(doc) = data.pointer("/document") {
        if let Some(pair) = title_text_from_object(doc, fallback_id) {
            return Ok(pair);
        }
    }

    if let Some(pair) = title_text_from_object(data, fallback_id) {
        return Ok(pair);
    }

    if let Some(messages) = data.get("messages").and_then(Value::as_array) {
        let text = format_im_messages(messages);
        if !text.trim().is_empty() {
            let title = data
                .get("chat_name")
                .or_else(|| data.get("chat_id"))
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("lark-im-{fallback_id}"));
            return Ok((title, text));
        }
    }

    for key in ["body_plain_text", "text", "markdown", "content"] {
        if let Some(text) = data.get(key).and_then(Value::as_str) {
            if !text.trim().is_empty() {
                let title = data
                    .get("subject")
                    .or_else(|| data.get("title"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("lark-{fallback_id}"));
                return Ok((title, text.to_string()));
            }
        }
    }

    Err(LarkError::MissingField(
        "title/text in lark-cli json response".into(),
    ))
}

fn title_text_from_object(obj: &Value, fallback_id: &str) -> Option<(String, String)> {
    let text = obj
        .get("content")
        .or_else(|| obj.get("text"))
        .or_else(|| obj.get("markdown"))
        .or_else(|| obj.get("body_plain_text"))
        .and_then(Value::as_str)
        .filter(|t| !t.trim().is_empty())?;

    let title = obj
        .get("title")
        .or_else(|| obj.get("subject"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| extract_xml_title(text))
        .or_else(|| {
            obj.get("document_id")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| format!("lark-{fallback_id}"));

    Some((title, text.to_string()))
}

fn extract_xml_title(content: &str) -> Option<String> {
    let start = content.find("<title>")? + "<title>".len();
    let end = content[start..].find("</title>")? + start;
    let title = content[start..end].trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

fn format_im_messages(messages: &[Value]) -> String {
    let mut lines = Vec::new();
    for msg in messages {
        let sender = msg
            .get("sender_name")
            .or_else(|| msg.pointer("/sender/name"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let body = msg
            .get("content")
            .or_else(|| msg.pointer("/body/content"))
            .or_else(|| msg.get("text"))
            .and_then(Value::as_str)
            .unwrap_or("");
        if body.trim().is_empty() {
            continue;
        }
        let time = msg
            .get("create_time_formatted")
            .or_else(|| msg.get("create_time"))
            .and_then(Value::as_str)
            .unwrap_or("");
        if time.is_empty() {
            lines.push(format!("{sender}: {body}"));
        } else {
            lines.push(format!("[{time}] {sender}: {body}"));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_doc_v2_envelope() {
        let raw = r#"{"ok":true,"data":{"document":{"document_id":"dox1","content":"<title>Spec</title><p>Hello</p>"}}}"#;
        let value = parse_cli_json(raw).unwrap();
        let (title, text) = extract_title_and_text(&value, "dox1").unwrap();
        assert_eq!(title, "Spec");
        assert!(text.contains("Hello"));
    }

    #[test]
    fn parses_mail_envelope() {
        let raw = r#"{"ok":true,"data":{"subject":"Weekly","body_plain_text":"mail body"}}"#;
        let value = parse_cli_json(raw).unwrap();
        let (title, text) = extract_title_and_text(&value, "m1").unwrap();
        assert_eq!(title, "Weekly");
        assert_eq!(text, "mail body");
    }

    #[test]
    fn parses_im_messages() {
        let raw = r#"{"ok":true,"data":{"chat_id":"oc_1","messages":[{"sender_name":"Alice","content":"hi"}]}}"#;
        let value = parse_cli_json(raw).unwrap();
        let (title, text) = extract_title_and_text(&value, "oc_1").unwrap();
        assert_eq!(title, "oc_1");
        assert!(text.contains("Alice"));
        assert!(text.contains("hi"));
    }

    #[test]
    fn surfaces_cli_error_envelope() {
        let raw = r#"{"ok":false,"error":{"message":"permission denied","hint":"run auth login"}}"#;
        let err = parse_cli_json(raw).unwrap_err().to_string();
        assert!(err.contains("permission denied"));
    }
}
