use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiAnalysis {
    pub summary: String,
    pub entities: Vec<WikiEntity>,
    pub concepts: Vec<WikiConcept>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiEntity {
    pub name: String,
    pub blurb: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiConcept {
    pub name: String,
    pub blurb: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiPageDraft {
    pub slug: String,
    pub title: String,
    pub page_type: WikiPageType,
    pub body_markdown: String,
    pub source_uris: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiCompileResult {
    pub pages: Vec<WikiPageDraft>,
    pub index_markdown: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WikiPageType {
    SourceSummary,
    Entity,
    Concept,
}

impl WikiPageType {
    pub fn as_str(self) -> &'static str {
        match self {
            WikiPageType::SourceSummary => "source_summary",
            WikiPageType::Entity => "entity",
            WikiPageType::Concept => "concept",
        }
    }
}

/// Deterministic: structured analysis → markdown drafts. No I/O.
/// Stub until Plan 08-02 implements page assembly.
pub fn render_wiki_pages(
    _analysis: &WikiAnalysis,
    _source_uri: &str,
    _source_title: &str,
) -> WikiCompileResult {
    WikiCompileResult {
        pages: Vec::new(),
        index_markdown: String::new(),
    }
}

// Called by render_wiki_pages in Plan 08-02; unit-tested now.
#[allow(dead_code)]
fn hash6(name: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(name.as_bytes());
    let full = hex::encode(digest);
    full[..6].to_string()
}

fn slugify(name: &str) -> String {
    let mut out = String::new();
    for ch in name.to_lowercase().chars() {
        match ch {
            'a'..='z' | '0'..='9' => out.push(ch),
            _ if ch.is_whitespace() || ch == '-' || ch == '_' => {
                if !out.ends_with('-') && !out.is_empty() {
                    out.push('-');
                }
            }
            _ => {}
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        format!("e-{}", hash6(name))
    } else {
        trimmed
    }
}

fn uniquify_slug(base: &str, seen: &mut std::collections::HashMap<String, usize>) -> String {
    let count = seen.entry(base.to_string()).or_insert(0);
    *count += 1;
    if *count == 1 {
        base.to_string()
    } else {
        format!("{base}-{count}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiki_page_type_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&WikiPageType::SourceSummary).unwrap(),
            "\"source_summary\""
        );
        assert_eq!(
            serde_json::to_string(&WikiPageType::Entity).unwrap(),
            "\"entity\""
        );
        assert_eq!(
            serde_json::to_string(&WikiPageType::Concept).unwrap(),
            "\"concept\""
        );
        assert_eq!(WikiPageType::SourceSummary.as_str(), "source_summary");
        assert_eq!(WikiPageType::Entity.as_str(), "entity");
        assert_eq!(WikiPageType::Concept.as_str(), "concept");
    }

    #[test]
    fn render_wiki_pages_stub_returns_empty() {
        let analysis = WikiAnalysis {
            summary: "要点".into(),
            entities: vec![],
            concepts: vec![],
        };
        let out = render_wiki_pages(&analysis, "file:///a.md", "Doc A");
        assert!(out.pages.is_empty());
        assert!(out.index_markdown.is_empty());
    }

    #[test]
    fn slugify_ascii_name() {
        assert_eq!(slugify("Acme Corp"), "acme-corp");
        assert_eq!(slugify("Acme__Corp--Inc"), "acme-corp-inc");
        assert_eq!(slugify("  Hello World  "), "hello-world");
    }

    #[test]
    fn cjk_name_uses_hash_slug() {
        let s = slugify("张三");
        assert!(s.starts_with("e-"), "expected e- prefix, got {s}");
        let hex = &s[2..];
        assert_eq!(hex.len(), 6, "expected 6 hex chars, got {s}");
        assert!(
            hex.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')),
            "expected lowercase hex, got {s}"
        );
        assert_eq!(slugify("张三"), s, "slugify must be deterministic");
    }

    #[test]
    fn slug_collision_suffix() {
        let mut entities = std::collections::HashMap::new();
        let a = uniquify_slug(&slugify("Acme"), &mut entities);
        let b = uniquify_slug(&slugify("Acme"), &mut entities);
        assert_eq!(a, "acme");
        assert_eq!(b, "acme-2");

        let mut concepts = std::collections::HashMap::new();
        let c = uniquify_slug(&slugify("Acme"), &mut concepts);
        assert_eq!(c, "acme", "cross-directory names must not collide");
    }

    #[test]
    fn unsafe_path_chars_stripped() {
        let s = slugify("Acme/Corp\\Division");
        assert!(!s.contains('/'), "slash must not appear in slug: {s}");
        assert!(!s.contains('\\'), "backslash must not appear in slug: {s}");
        assert_eq!(s, "acmecorpdivision");
    }
}
