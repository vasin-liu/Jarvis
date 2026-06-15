use std::fs;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::{AgentError, Result};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginTool {
    pub name: String,
    pub description: String,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub tools: Vec<PluginTool>,
}

pub fn load_plugins_from_dir(dir: &Path) -> Vec<PluginManifest> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut out = Vec::new();
    collect_plugin_manifests(dir, &mut out);
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

fn collect_plugin_manifests(dir: &Path, out: &mut Vec<PluginManifest>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let manifest = path.join("plugin.json");
            if manifest.is_file() {
                if let Ok(raw) = fs::read_to_string(&manifest) {
                    if let Ok(plugin) = serde_json::from_str::<PluginManifest>(&raw) {
                        if !plugin.id.is_empty() && !plugin.tools.is_empty() {
                            out.push(plugin);
                        }
                    }
                }
            }
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(raw) = fs::read_to_string(&path) {
                if let Ok(plugin) = serde_json::from_str::<PluginManifest>(&raw) {
                    if !plugin.id.is_empty() && !plugin.tools.is_empty() {
                        out.push(plugin);
                    }
                }
            }
        }
    }
}

pub fn enabled_plugin_tools<'a>(
    plugins: &'a [PluginManifest],
    enabled_ids: &[String],
) -> Vec<(&'a PluginManifest, &'a PluginTool)> {
    let mut out = Vec::new();
    for plugin in plugins {
        if !enabled_ids.iter().any(|id| id == &plugin.id) {
            continue;
        }
        for tool in &plugin.tools {
            out.push((plugin, tool));
        }
    }
    out
}

pub fn find_plugin_tool<'a>(
    plugins: &'a [PluginManifest],
    enabled_ids: &[String],
    name: &str,
) -> Option<(&'a PluginManifest, &'a PluginTool)> {
    enabled_plugin_tools(plugins, enabled_ids)
        .into_iter()
        .find(|(_, tool)| tool.name == name)
}

pub fn execute_plugin_tool(
    tool: &PluginTool,
    args: &serde_json::Value,
) -> Result<String> {
    let mut cmd = shell_command(&tool.command);
    cmd.env("JARVIS_TOOL_NAME", &tool.name);
    cmd.env("JARVIS_TOOL_ARGS", args.to_string());

    let output = cmd
        .output()
        .map_err(|e| AgentError::PluginExec(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AgentError::PluginExec(format!(
            "plugin tool `{}` failed: {}",
            tool.name,
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        Ok("(插件工具无输出)".into())
    } else {
        Ok(stdout)
    }
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
    fn loads_nested_plugin_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let plugin_dir = dir.path().join("demo");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(
            plugin_dir.join("plugin.json"),
            r#"{"id":"demo","name":"Demo","tools":[{"name":"ping","description":"ping","command":"echo pong"}]}"#,
        )
        .unwrap();
        let plugins = load_plugins_from_dir(dir.path());
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].tools[0].name, "ping");
    }
}
