use serde::Deserialize;

use crate::tools::ToolCallPayload;

#[derive(Debug, Clone, PartialEq)]
pub enum ToolCallParseOutcome {
    Found(ToolCallPayload),
    NotFound,
    Failed { error: String, raw_snippet: String },
}

pub trait ToolCallParser {
    fn id(&self) -> &str;
    fn parse(&self, text: &str) -> ToolCallParseOutcome;
}

pub struct JsonToolCallParser;

impl ToolCallParser for JsonToolCallParser {
    fn id(&self) -> &str {
        "json"
    }

    fn parse(&self, text: &str) -> ToolCallParseOutcome {
        if text.contains("```") {
            return ToolCallParseOutcome::Failed {
                error: "markdown fence not allowed".into(),
                raw_snippet: truncate_snippet(text),
            };
        }

        let trimmed = text.trim();
        if trimmed.is_empty() {
            return ToolCallParseOutcome::NotFound;
        }

        let non_empty_lines: Vec<&str> = trimmed
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();

        let json_lines: Vec<&str> = non_empty_lines
            .iter()
            .copied()
            .filter(|l| l.starts_with("{\"name\""))
            .collect();

        if json_lines.len() > 1 {
            return ToolCallParseOutcome::Failed {
                error: "multiple JSON tool call objects in reply".into(),
                raw_snippet: truncate_snippet(text),
            };
        }

        let parse_target = if non_empty_lines.len() == 1 {
            non_empty_lines[0]
        } else if trimmed.contains('\n') {
            if let Some(&line) = json_lines.first() {
                line
            } else if non_empty_lines
                .iter()
                .any(|l| l.starts_with("{\"name\""))
            {
                return ToolCallParseOutcome::Failed {
                    error: "could not locate JSON tool call line".into(),
                    raw_snippet: truncate_snippet(text),
                };
            } else {
                return ToolCallParseOutcome::NotFound;
            }
        } else {
            trimmed
        };

        if !parse_target.starts_with("{\"name\"") {
            if text.contains("```") {
                return ToolCallParseOutcome::Failed {
                    error: "markdown fence not allowed".into(),
                    raw_snippet: truncate_snippet(text),
                };
            }
            if non_empty_lines
                .iter()
                .any(|l| l.starts_with("{\"name\""))
            {
                return ToolCallParseOutcome::Failed {
                    error: "tool intent detected but JSON parse failed".into(),
                    raw_snippet: truncate_snippet(parse_target),
                };
            }
            return ToolCallParseOutcome::NotFound;
        }

        match serde_json::from_str::<RawToolCall>(parse_target) {
            Ok(raw) => match raw.into_payload() {
                Ok(payload) => ToolCallParseOutcome::Found(payload),
                Err(err) => ToolCallParseOutcome::Failed {
                    error: err,
                    raw_snippet: truncate_snippet(parse_target),
                },
            },
            Err(err) => ToolCallParseOutcome::Failed {
                error: format!("invalid JSON: {err}"),
                raw_snippet: truncate_snippet(parse_target),
            },
        }
    }
}

pub struct XmlToolCallParser;

impl ToolCallParser for XmlToolCallParser {
    fn id(&self) -> &str {
        "xml"
    }

    fn parse(&self, text: &str) -> ToolCallParseOutcome {
        let start = match text.find("<tool_call>") {
            Some(s) => s,
            None => return ToolCallParseOutcome::NotFound,
        };
        let end = match text.find("</tool_call>") {
            Some(e) => e,
            None => {
                return ToolCallParseOutcome::Failed {
                    error: "unclosed tool_call tag".into(),
                    raw_snippet: truncate_snippet(text),
                };
            }
        };

        let json = text[start + 11..end].trim();
        match serde_json::from_str::<RawToolCall>(json) {
            Ok(raw) => match raw.into_payload() {
                Ok(payload) => {
                    eprintln!("[jarvis] XML tool_call format deprecated; use JSON line");
                    ToolCallParseOutcome::Found(payload)
                }
                Err(err) => ToolCallParseOutcome::Failed {
                    error: err,
                    raw_snippet: truncate_snippet(json),
                },
            },
            Err(err) => ToolCallParseOutcome::Failed {
                error: format!("invalid JSON inside tool_call tags: {err}"),
                raw_snippet: truncate_snippet(json),
            },
        }
    }
}

pub struct CompositeToolCallParser {
    json: JsonToolCallParser,
    xml: XmlToolCallParser,
}

impl Default for CompositeToolCallParser {
    fn default() -> Self {
        Self {
            json: JsonToolCallParser,
            xml: XmlToolCallParser,
        }
    }
}

impl CompositeToolCallParser {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ToolCallParser for CompositeToolCallParser {
    fn id(&self) -> &str {
        "composite"
    }

    fn parse(&self, text: &str) -> ToolCallParseOutcome {
        match self.json.parse(text) {
            ToolCallParseOutcome::Found(payload) => ToolCallParseOutcome::Found(payload),
            ToolCallParseOutcome::Failed { error, raw_snippet } => {
                ToolCallParseOutcome::Failed { error, raw_snippet }
            }
            ToolCallParseOutcome::NotFound => self.xml.parse(text),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawToolCall {
    name: serde_json::Value,
    arguments: Option<serde_json::Value>,
}

impl RawToolCall {
    fn into_payload(self) -> Result<ToolCallPayload, String> {
        let name = match self.name {
            serde_json::Value::String(s) => s,
            _ => return Err("name must be a string".into()),
        };
        let arguments = match self.arguments {
            Some(serde_json::Value::Object(obj)) => serde_json::Value::Object(obj),
            Some(_) => return Err("arguments must be an object".into()),
            None => return Err("arguments required".into()),
        };
        Ok(ToolCallPayload { name, arguments })
    }
}

fn has_tool_intent(text: &str) -> bool {
    text.contains("<tool_call>")
        || text.contains("```")
        || text.lines().any(|l| l.trim().starts_with("{\"name\""))
}

fn truncate_snippet(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= 120 {
        trimmed.to_string()
    } else {
        let truncated: String = trimmed.chars().take(120).collect();
        format!("{truncated}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn composite() -> CompositeToolCallParser {
        CompositeToolCallParser::new()
    }

    #[test]
    fn bare_json_line_found() {
        let text = r#"{"name":"search_knowledge","arguments":{"query":"test"}}"#;
        let outcome = composite().parse(text);
        match outcome {
            ToolCallParseOutcome::Found(p) => {
                assert_eq!(p.name, "search_knowledge");
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn xml_fallback_found() {
        let text = r#"<tool_call>{"name":"list_tasks","arguments":{}}</tool_call>"#;
        let outcome = composite().parse(text);
        match outcome {
            ToolCallParseOutcome::Found(p) => assert_eq!(p.name, "list_tasks"),
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn fence_rejected() {
        let text = r#"```json
{"name":"search_knowledge","arguments":{"query":"x"}}
```"#;
        let outcome = composite().parse(text);
        match outcome {
            ToolCallParseOutcome::Failed { error, .. } => {
                assert!(error.contains("markdown fence"));
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn two_json_lines_failed() {
        let text = r#"{"name":"a","arguments":{}}
{"name":"b","arguments":{}}"#;
        let outcome = composite().parse(text);
        match outcome {
            ToolCallParseOutcome::Failed { error, .. } => {
                assert!(error.contains("multiple"));
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn missing_arguments_failed() {
        let text = r#"{"name":"search_knowledge"}"#;
        let outcome = composite().parse(text);
        match outcome {
            ToolCallParseOutcome::Failed { error, .. } => {
                assert!(error.contains("arguments"));
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn prose_not_found() {
        let text = "这是普通的中文回答，没有工具调用。";
        let outcome = composite().parse(text);
        assert_eq!(outcome, ToolCallParseOutcome::NotFound);
    }

    #[test]
    fn broken_json_after_name_marker_failed() {
        let text = r#"{"name":"broken"#;
        let outcome = composite().parse(text);
        match outcome {
            ToolCallParseOutcome::Failed { .. } => {}
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn preamble_plus_one_json_line_found() {
        let text = "思考中…\n{\"name\":\"list_sources\",\"arguments\":{}}";
        let outcome = composite().parse(text);
        match outcome {
            ToolCallParseOutcome::Found(p) => assert_eq!(p.name, "list_sources"),
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn invalid_json_inside_xml_tags_failed() {
        let text = r#"<tool_call>{"name":"x"</tool_call>"#;
        let outcome = XmlToolCallParser.parse(text);
        match outcome {
            ToolCallParseOutcome::Failed { .. } => {}
            other => panic!("expected Failed, got {other:?}"),
        }
    }
}
