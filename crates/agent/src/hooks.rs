use std::fs;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookEvent {
    BeforeToolCall,
    AfterToolCall,
    BeforeAnswer,
}

impl HookEvent {
    pub fn as_str(self) -> &'static str {
        match self {
            HookEvent::BeforeToolCall => "before_tool_call",
            HookEvent::AfterToolCall => "after_tool_call",
            HookEvent::BeforeAnswer => "before_answer",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "before_tool_call" => Some(HookEvent::BeforeToolCall),
            "after_tool_call" => Some(HookEvent::AfterToolCall),
            "before_answer" => Some(HookEvent::BeforeAnswer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hook {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub event: String,
    pub command: String,
}

#[derive(Debug, Clone, Default)]
pub struct HookContext<'a> {
    pub question: Option<&'a str>,
    pub tool_name: Option<&'a str>,
    pub tool_args: Option<&'a serde_json::Value>,
    pub tool_result: Option<&'a str>,
    pub answer: Option<&'a str>,
}

pub fn load_hooks_from_dir(dir: &Path) -> Vec<Hook> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(hook) = serde_json::from_str::<Hook>(&raw) {
                if !hook.id.is_empty() && !hook.command.is_empty() {
                    out.push(hook);
                }
            }
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

pub fn run_hooks(
    hooks: &[Hook],
    enabled_ids: &[String],
    event: HookEvent,
    ctx: &HookContext<'_>,
) {
    let event_name = event.as_str();
    for hook in hooks {
        if !enabled_ids.iter().any(|id| id == &hook.id) {
            continue;
        }
        if HookEvent::parse(&hook.event) != Some(event) {
            continue;
        }
        let _ = run_hook_command(&hook.command, event_name, ctx);
    }
}

fn run_hook_command(command: &str, event: &str, ctx: &HookContext<'_>) -> std::io::Result<String> {
    let args_json = ctx
        .tool_args
        .map(|v| v.to_string())
        .unwrap_or_else(|| "{}".to_string());

    let mut cmd = shell_command(command);
    cmd.env("JARVIS_HOOK_EVENT", event);
    if let Some(q) = ctx.question {
        cmd.env("JARVIS_QUESTION", q);
    }
    if let Some(name) = ctx.tool_name {
        cmd.env("JARVIS_TOOL_NAME", name);
    }
    cmd.env("JARVIS_TOOL_ARGS", &args_json);
    if let Some(result) = ctx.tool_result {
        cmd.env("JARVIS_TOOL_RESULT", result);
    }
    if let Some(answer) = ctx.answer {
        cmd.env("JARVIS_ANSWER", answer);
    }

    let output = cmd.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(stdout)
}

fn shell_command(command: &str) -> Command {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", command]);
        cmd
    }
    #[cfg(not(windows))]
    {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", command]);
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_hook_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.json");
        fs::write(
            &path,
            r#"{"id":"audit","name":"Audit","event":"after_tool_call","command":"echo ok"}"#,
        )
        .unwrap();
        let hooks = load_hooks_from_dir(dir.path());
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].id, "audit");
    }

    #[test]
    fn parses_hook_events() {
        assert_eq!(
            HookEvent::parse("before_tool_call"),
            Some(HookEvent::BeforeToolCall)
        );
        assert_eq!(HookEvent::parse("nope"), None);
    }
}
