use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::options::LarkCliOptions;
use crate::parse::parse_cli_json;
use crate::runner::CommandRunner;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LarkAuthStatus {
    pub ok: bool,
    pub identity: String,
    pub user_name: Option<String>,
    pub user_open_id: Option<String>,
    pub token_status: Option<String>,
    pub user_available: bool,
    pub bot_available: bool,
    pub note: Option<String>,
    pub hint: Option<String>,
    pub app_id: Option<String>,
    pub brand: Option<String>,
}

pub fn check_auth(runner: &dyn CommandRunner, opts: &LarkCliOptions<'_>) -> Result<LarkAuthStatus> {
    let raw = runner.run(opts.bin, &["auth", "status"])?;
    let value = parse_cli_json(&raw)?;

    let user_available = value
        .pointer("/identities/user/available")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let bot_available = value
        .pointer("/identities/bot/available")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let ok = user_available || bot_available;
    let hint = value
        .pointer("/identities/user/hint")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            if !ok {
                Some("运行 lark-cli config init 与 lark-cli auth login".into())
            } else if !user_available && bot_available {
                Some("用户身份未就绪；访问个人文档/邮箱请执行 lark-cli auth login".into())
            } else {
                None
            }
        });

    Ok(LarkAuthStatus {
        ok,
        identity: value
            .get("identity")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        user_name: value
            .get("userName")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        user_open_id: value
            .get("userOpenId")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        token_status: value
            .get("tokenStatus")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        user_available,
        bot_available,
        note: value
            .get("note")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        hint,
        app_id: value
            .get("appId")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        brand: value
            .get("brand")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
    })
}

pub fn detect_cli_bin(runner: &dyn CommandRunner) -> Option<String> {
    for candidate in ["lark-cli", "lark"] {
        if runner.run(candidate, &["--version"]).is_ok() {
            return Some(candidate.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{LarkCliOptions, LarkIdentity};
    use crate::runner::FakeRunner;

    #[test]
    fn parses_auth_status_json() {
        let runner = FakeRunner::new();
        runner.insert(
            "lark-cli auth status",
            r#"{
              "identity":"bot",
              "userName":"Alice",
              "userOpenId":"ou_1",
              "tokenStatus":"ready",
              "identities":{
                "user":{"available":true,"status":"ready"},
                "bot":{"available":true,"status":"ready"}
              },
              "note":"ok"
            }"#,
        );
        let opts = LarkCliOptions::new("lark-cli", LarkIdentity::User);
        let status = check_auth(&runner, &opts).unwrap();
        assert!(status.ok);
        assert_eq!(status.user_name.as_deref(), Some("Alice"));
        assert!(status.user_available);
        assert!(status.bot_available);
    }

    #[test]
    fn detects_missing_user_identity() {
        let runner = FakeRunner::new();
        runner.insert(
            "lark-cli auth status",
            r#"{
              "identity":"bot",
              "identities":{
                "user":{"available":false,"hint":"run auth login"},
                "bot":{"available":true}
              }
            }"#,
        );
        let opts = LarkCliOptions::new("lark-cli", LarkIdentity::Auto);
        let status = check_auth(&runner, &opts).unwrap();
        assert!(status.ok);
        assert!(!status.user_available);
        assert!(status.bot_available);
        assert!(status.hint.is_some());
    }
}
