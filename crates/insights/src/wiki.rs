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
pub fn render_wiki_pages(
    analysis: &WikiAnalysis,
    source_uri: &str,
    source_title: &str,
) -> WikiCompileResult {
    let mut sources_seen = std::collections::HashMap::new();
    let mut entities_seen = std::collections::HashMap::new();
    let mut concepts_seen = std::collections::HashMap::new();

    let source_base = slugify(source_title);
    let source_slug_seg = uniquify_slug(&source_base, &mut sources_seen);
    let source_slug = format!("sources/{source_slug_seg}");

    // Precompute entity/concept paths so the summary can forward-link.
    let mut entity_pages = Vec::with_capacity(analysis.entities.len());
    for entity in &analysis.entities {
        let seg = uniquify_slug(&slugify(&entity.name), &mut entities_seen);
        let slug = format!("entities/{seg}");
        entity_pages.push((slug, entity));
    }
    let mut concept_pages = Vec::with_capacity(analysis.concepts.len());
    for concept in &analysis.concepts {
        let seg = uniquify_slug(&slugify(&concept.name), &mut concepts_seen);
        let slug = format!("concepts/{seg}");
        concept_pages.push((slug, concept));
    }

    let mut summary_body = String::new();
    summary_body.push_str(&build_frontmatter(
        source_title,
        WikiPageType::SourceSummary.as_str(),
        source_uri,
    ));
    summary_body.push_str(&format!("# {source_title}\n\n"));
    summary_body.push_str(&analysis.summary);
    summary_body.push('\n');
    for (slug, entity) in &entity_pages {
        summary_body.push('\n');
        summary_body.push_str(&wikilink(slug, &entity.name));
        summary_body.push('\n');
    }
    for (slug, concept) in &concept_pages {
        summary_body.push('\n');
        summary_body.push_str(&wikilink(slug, &concept.name));
        summary_body.push('\n');
    }

    let mut pages = Vec::with_capacity(1 + entity_pages.len() + concept_pages.len());
    pages.push(WikiPageDraft {
        slug: source_slug.clone(),
        title: source_title.to_string(),
        page_type: WikiPageType::SourceSummary,
        body_markdown: summary_body,
        source_uris: vec![source_uri.to_string()],
    });

    let backlink = wikilink(&source_slug, source_title);
    for (slug, entity) in &entity_pages {
        let mut body = String::new();
        body.push_str(&build_frontmatter(
            &entity.name,
            WikiPageType::Entity.as_str(),
            source_uri,
        ));
        body.push_str(&format!("# {}\n\n", entity.name));
        body.push_str(&entity.blurb);
        body.push_str("\n\n");
        body.push_str(&backlink);
        body.push('\n');
        pages.push(WikiPageDraft {
            slug: slug.clone(),
            title: entity.name.clone(),
            page_type: WikiPageType::Entity,
            body_markdown: body,
            source_uris: vec![source_uri.to_string()],
        });
    }
    for (slug, concept) in &concept_pages {
        let mut body = String::new();
        body.push_str(&build_frontmatter(
            &concept.name,
            WikiPageType::Concept.as_str(),
            source_uri,
        ));
        body.push_str(&format!("# {}\n\n", concept.name));
        body.push_str(&concept.blurb);
        body.push_str("\n\n");
        body.push_str(&backlink);
        body.push('\n');
        pages.push(WikiPageDraft {
            slug: slug.clone(),
            title: concept.name.clone(),
            page_type: WikiPageType::Concept,
            body_markdown: body,
            source_uris: vec![source_uri.to_string()],
        });
    }

    let index_markdown = build_index_markdown(&pages);
    WikiCompileResult {
        pages,
        index_markdown,
    }
}

fn build_index_markdown(pages: &[WikiPageDraft]) -> String {
    let mut out = String::from("# Wiki\n");
    let sources: Vec<_> = pages
        .iter()
        .filter(|p| p.page_type == WikiPageType::SourceSummary)
        .collect();
    let entities: Vec<_> = pages
        .iter()
        .filter(|p| p.page_type == WikiPageType::Entity)
        .collect();
    let concepts: Vec<_> = pages
        .iter()
        .filter(|p| p.page_type == WikiPageType::Concept)
        .collect();

    if !sources.is_empty() {
        out.push_str("\n## Sources\n");
        for page in sources {
            out.push_str(&format!("- {}\n", wikilink(&page.slug, &page.title)));
        }
    }
    if !entities.is_empty() {
        out.push_str("\n## Entities\n");
        for page in entities {
            out.push_str(&format!("- {}\n", wikilink(&page.slug, &page.title)));
        }
    }
    if !concepts.is_empty() {
        out.push_str("\n## Concepts\n");
        for page in concepts {
            out.push_str(&format!("- {}\n", wikilink(&page.slug, &page.title)));
        }
    }
    out
}

// Called by render_wiki_pages in Plan 08-02; unit-tested now.
#[allow(dead_code)]
fn hash6(name: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(name.as_bytes());
    let full = hex::encode(digest);
    full[..6].to_string()
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn uniquify_slug(base: &str, seen: &mut std::collections::HashMap<String, usize>) -> String {
    let count = seen.entry(base.to_string()).or_insert(0);
    *count += 1;
    if *count == 1 {
        base.to_string()
    } else {
        format!("{base}-{count}")
    }
}

#[allow(dead_code)]
fn build_frontmatter(title: &str, page_type_str: &str, source_uri: &str) -> String {
    let escaped_title = title.replace('\\', "\\\\").replace('"', "\\\"");
    let escaped_uri = source_uri.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        "---\ntitle: \"{escaped_title}\"\ntype: {page_type_str}\nsources: [\"{escaped_uri}\"]\ngenerated: true\n---\n"
    )
}

#[allow(dead_code)]
fn sanitize_display(display: &str) -> String {
    display.replace('|', "").replace(']', "")
}

#[allow(dead_code)]
fn wikilink(path: &str, display: &str) -> String {
    format!("[[{path}|{}]]", sanitize_display(display))
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

    #[test]
    fn frontmatter_includes_required_keys() {
        let fm = build_frontmatter("Doc A", WikiPageType::SourceSummary.as_str(), "file:///a.md");
        assert!(fm.contains("title: \"Doc A\""), "missing title: {fm}");
        assert!(fm.contains("type: source_summary"), "missing type: {fm}");
        assert!(
            fm.contains("sources: [\"file:///a.md\"]"),
            "missing sources array: {fm}"
        );
        assert!(fm.contains("generated: true"), "missing generated: {fm}");
        assert!(fm.starts_with("---\n"), "missing opening fence: {fm}");
        assert!(fm.contains("\n---\n"), "missing closing fence: {fm}");
    }

    #[test]
    fn frontmatter_omits_content_hash() {
        let fm = build_frontmatter("Doc A", WikiPageType::SourceSummary.as_str(), "file:///a.md");
        assert!(
            !fm.contains("content_hash"),
            "content_hash must be omitted in Phase 08: {fm}"
        );
    }

    #[test]
    fn frontmatter_escapes_title_quotes() {
        let fm = build_frontmatter(
            r#"Say "hello""#,
            WikiPageType::Entity.as_str(),
            "file:///a.md",
        );
        assert!(
            !fm.contains(r#"title: "Say "hello"""#),
            "raw double quotes must not break YAML title: {fm}"
        );
        assert!(fm.contains("title:"), "title key required: {fm}");
        // Escaped form must still be a single YAML double-quoted scalar line.
        let title_line = fm
            .lines()
            .find(|l| l.starts_with("title:"))
            .expect("title line");
        assert!(
            title_line.starts_with("title: \"") && title_line.ends_with('"'),
            "title must remain double-quoted: {title_line}"
        );
    }

    #[test]
    fn wikilink_uses_path_display_form() {
        let link = wikilink("entities/acme", "Acme");
        assert!(
            link.contains("[[entities/acme|Acme]]"),
            "expected path|display form, got {link}"
        );
    }

    #[test]
    fn wikilink_sanitizes_display() {
        let link = wikilink("entities/acme", "Acme|Corp]");
        assert!(
            link.starts_with("[[entities/acme|") && link.ends_with("]]"),
            "must keep path|display wikilink shape: {link}"
        );
        // Inspect display segment only — full-string contains("Corp]") also matches trailing ]].
        let display = link
            .strip_prefix("[[entities/acme|")
            .and_then(|s| s.strip_suffix("]]"))
            .expect("wikilink shape");
        assert!(
            !display.contains('|'),
            "pipe in display must be sanitized: {link}"
        );
        assert!(
            !display.contains(']'),
            "closing bracket in display must be sanitized: {link}"
        );
    }

    #[test]
    fn render_always_emits_source_summary() {
        let analysis = WikiAnalysis {
            summary: "要点摘要".into(),
            entities: vec![],
            concepts: vec![],
        };
        let out = render_wiki_pages(&analysis, "file:///a.md", "Doc A");
        assert_eq!(out.pages.len(), 1, "expected exactly one source-summary page");
        let page = &out.pages[0];
        assert!(
            page.slug.starts_with("sources/"),
            "slug must be under sources/: {}",
            page.slug
        );
        assert_eq!(page.page_type, WikiPageType::SourceSummary);
        assert!(
            page.body_markdown.contains("sources: [\"file:///a.md\"]"),
            "missing sources frontmatter: {}",
            page.body_markdown
        );
        assert!(
            page.body_markdown.contains("generated: true"),
            "missing generated: {}",
            page.body_markdown
        );
        assert!(
            !page.body_markdown.contains("content_hash"),
            "content_hash must be omitted: {}",
            page.body_markdown
        );
        assert!(
            out.index_markdown.contains("# Wiki"),
            "index missing # Wiki: {}",
            out.index_markdown
        );
        assert!(
            out.index_markdown.contains("## Sources"),
            "index missing ## Sources: {}",
            out.index_markdown
        );
        assert!(
            !out.index_markdown.contains("## Entities"),
            "empty entities must omit ## Entities: {}",
            out.index_markdown
        );
    }

    #[test]
    fn render_links_entities_bidirectional() {
        let analysis = WikiAnalysis {
            summary: "About Acme".into(),
            entities: vec![WikiEntity {
                name: "Acme".into(),
                blurb: "A company".into(),
            }],
            concepts: vec![],
        };
        let out = render_wiki_pages(&analysis, "file:///a.md", "Doc A");
        let summary = out
            .pages
            .iter()
            .find(|p| p.page_type == WikiPageType::SourceSummary)
            .expect("source summary");
        let entity = out
            .pages
            .iter()
            .find(|p| p.page_type == WikiPageType::Entity)
            .expect("entity page");
        assert!(
            summary.body_markdown.contains("[[entities/acme|Acme]]"),
            "summary must forward-link entity: {}",
            summary.body_markdown
        );
        assert!(
            entity.slug == "entities/acme",
            "entity slug: {}",
            entity.slug
        );
        assert!(
            entity.body_markdown.contains(&format!(
                "[[{}|Doc A]]",
                summary.slug
            )),
            "entity must backlink source: {}",
            entity.body_markdown
        );
        for page in &out.pages {
            assert!(
                !page.body_markdown.contains("content_hash"),
                "content_hash omitted: {}",
                page.body_markdown
            );
        }
    }

    #[test]
    fn render_links_concepts_bidirectional() {
        let analysis = WikiAnalysis {
            summary: "About Foo".into(),
            entities: vec![],
            concepts: vec![WikiConcept {
                name: "Foo".into(),
                blurb: "A concept".into(),
            }],
        };
        let out = render_wiki_pages(&analysis, "file:///a.md", "Doc A");
        let summary = out
            .pages
            .iter()
            .find(|p| p.page_type == WikiPageType::SourceSummary)
            .expect("source summary");
        let concept = out
            .pages
            .iter()
            .find(|p| p.page_type == WikiPageType::Concept)
            .expect("concept page");
        assert!(
            summary.body_markdown.contains("[[concepts/foo|Foo]]"),
            "summary must forward-link concept: {}",
            summary.body_markdown
        );
        assert_eq!(concept.slug, "concepts/foo");
        assert!(
            concept
                .body_markdown
                .contains(&format!("[[{}|Doc A]]", summary.slug)),
            "concept must backlink source: {}",
            concept.body_markdown
        );
    }
}
