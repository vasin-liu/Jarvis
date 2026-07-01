use std::path::PathBuf;

pub(crate) fn seed_skills_dir(dir: &PathBuf) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let example = dir.join("concise-answers.md");
    if !example.exists() {
        std::fs::write(
            &example,
            "---\nname: 简洁回答\ndescription: 回答尽量简短、分点列出\n---\n回答时使用要点列表，避免冗长铺垫。",
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub(crate) fn seed_hooks_dir(dir: &PathBuf) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let example = dir.join("log-tool-calls.json");
    if !example.exists() {
        let hook = serde_json::json!({
            "id": "log-tool-calls",
            "name": "记录工具调用",
            "description": "在 after_tool_call 时输出工具名（示例 Hook）",
            "event": "after_tool_call",
            "command": "echo tool called: %JARVIS_TOOL_NAME%"
        });
        std::fs::write(&example, serde_json::to_string_pretty(&hook).unwrap())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub(crate) fn seed_plugins_dir(dir: &PathBuf) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let plugin_dir = dir.join("datetime");
    std::fs::create_dir_all(&plugin_dir).map_err(|e| e.to_string())?;
    let manifest = plugin_dir.join("plugin.json");
    if !manifest.exists() {
        #[cfg(windows)]
        let command = r#"powershell -NoProfile -Command "Get-Date -Format 'yyyy-MM-dd HH:mm:ss'""#;
        #[cfg(not(windows))]
        let command = "date '+%Y-%m-%d %H:%M:%S'";
        let plugin = serde_json::json!({
            "id": "datetime",
            "name": "日期时间",
            "description": "提供当前本地日期时间",
            "permissions": ["shell_exec"],
            "tools": [{
                "name": "current_time",
                "description": "返回当前本地日期时间",
                "command": command
            }]
        });
        std::fs::write(&manifest, serde_json::to_string_pretty(&plugin).unwrap())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
