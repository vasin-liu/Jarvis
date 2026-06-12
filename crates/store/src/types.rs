use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceKind {
    LocalFile,
    LarkDoc,
    LarkMsg,
    LarkSheet,
    LarkMail,
    CursorTranscript,
}

impl SourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceKind::LocalFile => "local_file",
            SourceKind::LarkDoc => "lark_doc",
            SourceKind::LarkMsg => "lark_msg",
            SourceKind::LarkSheet => "lark_sheet",
            SourceKind::LarkMail => "lark_mail",
            SourceKind::CursorTranscript => "cursor_transcript",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "local_file" => SourceKind::LocalFile,
            "lark_doc" => SourceKind::LarkDoc,
            "lark_msg" => SourceKind::LarkMsg,
            "lark_sheet" => SourceKind::LarkSheet,
            "lark_mail" => SourceKind::LarkMail,
            "cursor_transcript" => SourceKind::CursorTranscript,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexStatus {
    Pending,
    Indexed,
    Failed,
}

impl IndexStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            IndexStatus::Pending => "pending",
            IndexStatus::Indexed => "indexed",
            IndexStatus::Failed => "failed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "pending" => IndexStatus::Pending,
            "indexed" => IndexStatus::Indexed,
            "failed" => IndexStatus::Failed,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub kind: SourceKind,
    pub uri: String,
    pub title: String,
    pub content_hash: String,
    pub indexed_at: Option<i64>,
    pub status: IndexStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

impl ChatRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
            ChatRole::System => "system",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "user" => ChatRole::User,
            "assistant" => ChatRole::Assistant,
            "system" => ChatRole::System,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i64,
    pub session_id: String,
    pub role: ChatRole,
    pub content: String,
    pub citations_json: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewChunk {
    pub ord: i64,
    pub text: String,
    pub loc: String,
    pub token_count: i64,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkHit {
    pub chunk_id: i64,
    pub source_id: String,
    pub text: String,
    pub loc: String,
    pub score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_kind_roundtrips() {
        for k in [
            SourceKind::LocalFile,
            SourceKind::LarkDoc,
            SourceKind::LarkMsg,
            SourceKind::LarkSheet,
            SourceKind::LarkMail,
            SourceKind::CursorTranscript,
        ] {
            assert_eq!(SourceKind::parse(k.as_str()), Some(k));
        }
        assert_eq!(SourceKind::parse("bogus"), None);
    }

    #[test]
    fn index_status_roundtrips() {
        for s in [IndexStatus::Pending, IndexStatus::Indexed, IndexStatus::Failed] {
            assert_eq!(IndexStatus::parse(s.as_str()), Some(s));
        }
    }
}
