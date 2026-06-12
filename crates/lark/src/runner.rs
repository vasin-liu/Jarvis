use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;

use crate::error::{LarkError, Result};

pub trait CommandRunner: Send + Sync {
    fn run(&self, program: &str, args: &[&str]) -> Result<String>;
}

pub struct ProcessRunner;

impl CommandRunner for ProcessRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| LarkError::Command(format!("spawn {program}: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LarkError::Command(format!(
                "{program} {} exited {}: {}",
                args.join(" "),
                output.status,
                stderr.trim()
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
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

impl CommandRunner for FakeRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<String> {
        let key = format!("{program} {}", args.join(" "));
        self.responses
            .lock()
            .unwrap()
            .get(&key)
            .cloned()
            .ok_or_else(|| LarkError::Command(format!("no fake response for {key}")))
    }
}
