use crate::error::{LarkError, Result};
use crate::runner::CommandRunner;

pub fn check_auth(runner: &dyn CommandRunner, cli_bin: &str) -> Result<String> {
    let out = runner.run(cli_bin, &["auth", "+whoami"])?;
    let trimmed = out.trim();
    if trimmed.is_empty() {
        return Err(LarkError::Command("empty auth response".into()));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::FakeRunner;

    #[test]
    fn parses_whoami_output() {
        let runner = FakeRunner::new();
        runner.insert("lark-cli auth +whoami", "user@example.com");
        let who = check_auth(&runner, "lark-cli").unwrap();
        assert_eq!(who, "user@example.com");
    }
}
