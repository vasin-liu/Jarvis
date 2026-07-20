use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

use crate::error::{LarkError, Result};

pub trait CommandRunner: Send + Sync {
    fn run(&self, program: &str, args: &[&str]) -> Result<String> {
        self.run_in(program, args, None)
    }

    fn run_in(&self, program: &str, args: &[&str], cwd: Option<&Path>) -> Result<String>;
}

pub struct ProcessRunner;

impl CommandRunner for ProcessRunner {
    fn run_in(&self, program: &str, args: &[&str], cwd: Option<&Path>) -> Result<String> {
        let output = spawn_command(program, args, cwd)?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let message = format_cli_failure(&stdout, &stderr);
            return Err(LarkError::Command(format!(
                "{program} {} exited {}: {message}",
                args.join(" "),
                output.status,
            )));
        }

        Ok(stdout)
    }
}

fn spawn_command(program: &str, args: &[&str], cwd: Option<&Path>) -> Result<std::process::Output> {
    let mut command = build_command(program, args);
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    command
        .output()
        .or_else(|_| {
            if cfg!(windows) && !program.ends_with(".cmd") {
                let mut fallback = build_command(&format!("{program}.cmd"), args);
                if let Some(dir) = cwd {
                    fallback.current_dir(dir);
                }
                fallback.output()
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("spawn {program}"),
                ))
            }
        })
        .map_err(|e| LarkError::Command(format!("spawn {program}: {e}")))
}

fn build_command(program: &str, args: &[&str]) -> Command {
    let mut command = Command::new(program);
    command.args(args);
    command
}

pub struct FakeRunner {
    responses: Mutex<HashMap<String, String>>,
}

impl FakeRunner {
    pub fn new() -> Self {
        Self {
            responses: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, key: impl Into<String>, body: impl Into<String>) {
        self.responses.lock().unwrap().insert(key.into(), body.into());
    }
}

impl Default for FakeRunner {
    fn default() -> Self {
        Self::new()
    }
}

fn format_cli_failure(stdout: &str, stderr: &str) -> String {
    for raw in [stdout, stderr] {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw.trim()) {
            if let Some(msg) = value.pointer("/error/message").and_then(|v| v.as_str()) {
                let hint = value
                    .pointer("/error/hint")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if hint.is_empty() {
                    return msg.to_string();
                }
                return format!("{msg} ({hint})");
            }
        }
    }
    let stderr = stderr.trim();
    if !stderr.is_empty() {
        return stderr.to_string();
    }
    stdout.trim().to_string()
}

impl CommandRunner for FakeRunner {
    fn run_in(&self, program: &str, args: &[&str], _cwd: Option<&Path>) -> Result<String> {
        let key = format!("{program} {}", args.join(" "));
        self.responses
            .lock()
            .unwrap()
            .get(&key)
            .cloned()
            .ok_or_else(|| LarkError::Command(format!("no fake response for {key}")))
    }
}
