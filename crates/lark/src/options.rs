#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LarkIdentity {
    User,
    Bot,
    Auto,
}

impl LarkIdentity {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "bot" => Self::Bot,
            "auto" => Self::Auto,
            _ => Self::User,
        }
    }

    pub fn as_cli_flag(self) -> Option<&'static str> {
        match self {
            Self::User => Some("user"),
            Self::Bot => Some("bot"),
            Self::Auto => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LarkCliOptions<'a> {
    pub bin: &'a str,
    pub identity: LarkIdentity,
}

impl<'a> LarkCliOptions<'a> {
    pub fn new(bin: &'a str, identity: LarkIdentity) -> Self {
        Self { bin, identity }
    }

    pub fn push_identity<'b>(&self, args: &'b mut Vec<String>) {
        if let Some(as_flag) = self.identity.as_cli_flag() {
            args.push("--as".into());
            args.push(as_flag.into());
        }
    }
}
