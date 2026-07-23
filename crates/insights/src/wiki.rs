use std::collections::HashSet;
use std::path::{Path, PathBuf};

use chunker::ChunkerConfig;
use embedder::Embedder;
use llm::{ChatModel, Message, Role};
use serde::{Deserialize, Serialize};
use store::{IndexStatus, SourceKind, Store};

use crate::error::{InsightsError, Result};

const MAX_SOURCE_CHARS: usize = 12_000;

const WIKI_SYSTEM_PROMPT: &str = "\
你是笔记编译助手。根据来源正文提取结构化知识，仅输出裸 JSON（不要 markdown 代码块）。\
字段名使用英文：summary（字符串）、entities（数组，每项 name/blurb）、concepts（数组，每项 name/blurb）。\
质量优先、宁缺毋滥：entities 与 concepts 各自建议不超过 5–8 项；无把握则输出空数组。";

/// Load an indexed source, ask ChatModel for wiki JSON, parse into WikiAnalysis.
/// Does not read or write `sources.summary`.
pub async fn analyze_source_for_wiki(
    store: &Store,
    chat: &dyn ChatModel,
    source_id: &str,
) -> Result<WikiAnalysis> {
    let source = store.get_source(source_id)?;
    if source.status != IndexStatus::Indexed {
        return Err(InsightsError::NotIndexed(source_id.to_string()));
    }

    let text = store.source_chunk_text(source_id)?;
    if text.trim().is_empty() {
        return Err(InsightsError::EmptySource(source_id.to_string()));
    }

    let excerpt = crate::truncate_chars(&text, MAX_SOURCE_CHARS);
    let messages = vec![
        Message {
            role: Role::System,
            content: WIKI_SYSTEM_PROMPT.into(),
        },
        Message {
            role: Role::User,
            content: format!("标题：{}\n\n正文：\n{}", source.title, excerpt),
        },
    ];

    let raw = chat.complete(&messages).await?;
    parse_wiki_analysis(&raw)
}

/// Write `render_wiki_pages` output under `wiki_root` using draft.slug paths + `index.md`.
pub fn write_wiki_pages_to_dir(compiled: &WikiCompileResult, wiki_root: &Path) -> Result<()> {
    std::fs::create_dir_all(wiki_root)?;
    for page in &compiled.pages {
        let path = wiki_root.join(format!("{}.md", page.slug));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &page.body_markdown)?;
    }
    std::fs::write(wiki_root.join("index.md"), &compiled.index_markdown)?;
    Ok(())
}

/// Compact outcome of a successful wiki compile (D-17).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiCompileSummary {
    pub wiki_root: PathBuf,
    pub pages_written: usize,
    pub created: usize,
    pub updated: usize,
    pub skipped_user_edit: usize,
    pub cleaned: usize,
}

/// Orchestrate analyze → write → scan-rebuild index.md → index WikiPage sources (WIKI-04).
pub async fn compile_wiki_for_source(
    store: &Store,
    chat: &dyn ChatModel,
    embedder: &dyn Embedder,
    chunker: &ChunkerConfig,
    source_id: &str,
    wiki_root: &Path,
    wiki_enabled: bool,
) -> Result<WikiCompileSummary> {
    // D-15 / D-16: hard gates before any FS or analyze I/O side effects.
    if !wiki_enabled {
        return Err(InsightsError::WikiDisabled);
    }
    let source = store.get_source(source_id)?;
    if source.kind == SourceKind::WikiPage {
        return Err(InsightsError::WikiPageInput(source_id.to_string()));
    }
    if source.status != IndexStatus::Indexed {
        return Err(InsightsError::NotIndexed(source_id.to_string()));
    }

    let analysis = analyze_source_for_wiki(store, chat, source_id).await?;
    let compiled = render_wiki_pages(&analysis, &source.uri, &source.title);

    std::fs::create_dir_all(wiki_root)?;

    let mut pages_written = 0usize;
    let mut skipped_user_edit = 0usize;
    let mut index_queue: Vec<(String, PathBuf)> = Vec::with_capacity(compiled.pages.len());
    let mut current_slugs: HashSet<String> = HashSet::with_capacity(compiled.pages.len());

    for page in &compiled.pages {
        let path = resolve_wiki_page_path(wiki_root, &page.slug)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let skip_write = if path.is_file() {
            let existing = std::fs::read_to_string(&path)?;
            !frontmatter_is_generated(&existing)
        } else {
            false
        };

        if skip_write {
            skipped_user_edit += 1;
        } else {
            std::fs::write(&path, &page.body_markdown)?;
            pages_written += 1;
        }
        current_slugs.insert(page.slug.clone());
        index_queue.push((page.slug.clone(), path));
    }

    let cleaned =
        cleanup_stale_wiki_pages_for_source(store, wiki_root, &source.uri, &current_slugs)?;
    rebuild_index_md_from_disk(wiki_root)?;

    let mut created = 0usize;
    let mut updated = 0usize;
    for (slug, path) in index_queue {
        let text = std::fs::read_to_string(&path)?;
        let uri = format!("wiki://{slug}");
        let title = title_from_frontmatter_or_slug(&text, &slug);
        let content_hash = ingest::hash_text(&text);

        let prior = store.get_source(&uri).ok();
        let prior_skip = prior
            .as_ref()
            .is_some_and(|s| s.content_hash == content_hash && s.status == IndexStatus::Indexed);
        let had_prior = prior.is_some();

        let doc = ingest::Document {
            uri: uri.clone(),
            title,
            text,
            content_hash,
        };
        indexer::index_document(store, embedder, chunker, doc, SourceKind::WikiPage).await?;

        if prior_skip {
            // Hash skip — neither created nor updated.
        } else if had_prior {
            updated += 1;
        } else {
            created += 1;
        }
    }

    Ok(WikiCompileSummary {
        wiki_root: wiki_root.to_path_buf(),
        pages_written,
        created,
        updated,
        skipped_user_edit,
        cleaned,
    })
}

/// Delete generated pages owned by `source_uri` that are absent from this compile (D-06…D-08).
fn cleanup_stale_wiki_pages_for_source(
    store: &Store,
    wiki_root: &Path,
    source_uri: &str,
    current_slugs: &HashSet<String>,
) -> Result<usize> {
    let mut cleaned = 0usize;
    for dir_name in ["sources", "entities", "concepts"] {
        let dir = wiki_root.join(dir_name);
        if !dir.is_dir() {
            continue;
        }
        let mut stale_paths: Vec<(String, PathBuf)> = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("page");
            if stem.contains("..") || stem.contains('/') || stem.contains('\\') {
                continue;
            }
            let slug = format!("{dir_name}/{stem}");
            if current_slugs.contains(&slug) {
                continue;
            }
            let text = std::fs::read_to_string(&path)?;
            // D-08: never delete user-curated notes lacking generated: true.
            if !frontmatter_is_generated(&text) {
                continue;
            }
            // D-07: only pages whose sources list contains this compile's URI.
            if !frontmatter_sources_contains(&text, source_uri) {
                continue;
            }
            stale_paths.push((slug, path));
        }
        for (slug, path) in stale_paths {
            std::fs::remove_file(&path)?;
            let wiki_uri = format!("wiki://{slug}");
            // Library remove_source pair — never delete_source alone (orphan vec/FTS).
            store.delete_chunks_for_source(&wiki_uri)?;
            store.delete_source(&wiki_uri)?;
            cleaned += 1;
        }
    }
    Ok(cleaned)
}

/// Rebuild root `index.md` by scanning sources/entities/concepts only (D-01…D-05).
pub fn rebuild_index_md_from_disk(wiki_root: &Path) -> Result<()> {
    let mut sources: Vec<(String, String)> = Vec::new();
    let mut entities: Vec<(String, String)> = Vec::new();
    let mut concepts: Vec<(String, String)> = Vec::new();

    scan_wiki_section(wiki_root, "sources", &mut sources)?;
    scan_wiki_section(wiki_root, "entities", &mut entities)?;
    scan_wiki_section(wiki_root, "concepts", &mut concepts)?;

    sources.sort_by(|a, b| a.0.cmp(&b.0));
    entities.sort_by(|a, b| a.0.cmp(&b.0));
    concepts.sort_by(|a, b| a.0.cmp(&b.0));

    let mut out = String::from("# Wiki\n");
    if !sources.is_empty() {
        out.push_str("\n## Sources\n");
        for (slug, title) in &sources {
            out.push_str(&format!("- {}\n", wikilink(slug, title)));
        }
    }
    if !entities.is_empty() {
        out.push_str("\n## Entities\n");
        for (slug, title) in &entities {
            out.push_str(&format!("- {}\n", wikilink(slug, title)));
        }
    }
    if !concepts.is_empty() {
        out.push_str("\n## Concepts\n");
        for (slug, title) in &concepts {
            out.push_str(&format!("- {}\n", wikilink(slug, title)));
        }
    }

    std::fs::create_dir_all(wiki_root)?;
    std::fs::write(wiki_root.join("index.md"), out)?;
    Ok(())
}

fn scan_wiki_section(
    wiki_root: &Path,
    dir_name: &str,
    out: &mut Vec<(String, String)>,
) -> Result<()> {
    let dir = wiki_root.join(dir_name);
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("page");
        if stem.contains("..") || stem.contains('/') || stem.contains('\\') {
            continue;
        }
        let slug = format!("{dir_name}/{stem}");
        let text = std::fs::read_to_string(&path)?;
        let title = title_from_frontmatter_or_slug(&text, &slug);
        out.push((slug, title));
    }
    Ok(())
}

fn resolve_wiki_page_path(wiki_root: &Path, slug: &str) -> Result<PathBuf> {
    use std::path::Component;
    let rel = Path::new(slug);
    if rel.is_absolute()
        || rel.components().any(|c| {
            !matches!(c, Component::Normal(_))
        })
    {
        return Err(InsightsError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("unsafe wiki slug: {slug}"),
        )));
    }
    Ok(wiki_root.join(format!("{slug}.md")))
}

fn peek_frontmatter(raw: &str) -> Option<&str> {
    if !raw.starts_with("---") {
        return None;
    }
    raw[3..]
        .find("\n---")
        .map(|end| raw[3..3 + end].trim_matches('\n'))
}

fn frontmatter_is_generated(raw: &str) -> bool {
    let Some(front) = peek_frontmatter(raw) else {
        return false;
    };
    front.lines().any(|line| {
        let t = line.trim();
        t == "generated: true" || t == "generated:true"
    })
}

fn frontmatter_sources_contains(raw: &str, source_uri: &str) -> bool {
    let Some(front) = peek_frontmatter(raw) else {
        return false;
    };
    let needle = format!("\"{source_uri}\"");
    front.lines().any(|line| {
        let t = line.trim();
        t.starts_with("sources:") && t.contains(&needle)
    })
}

fn title_from_frontmatter_or_slug(raw: &str, slug: &str) -> String {
    if let Some(front) = peek_frontmatter(raw) {
        for line in front.lines() {
            if let Some((k, v)) = line.split_once(':') {
                if k.trim() == "title" {
                    let mut title = v.trim().to_string();
                    if title.starts_with('"') && title.ends_with('"') && title.len() >= 2 {
                        title = title[1..title.len() - 1]
                            .replace("\\\"", "\"")
                            .replace("\\\\", "\\");
                    }
                    if !title.is_empty() {
                        return title;
                    }
                }
            }
        }
    }
    slug.rsplit('/')
        .next()
        .unwrap_or(slug)
        .to_string()
}

fn parse_wiki_analysis(raw: &str) -> Result<WikiAnalysis> {
    let trimmed = raw.trim();
    let after_fence = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(trimmed);

    let json = extract_first_json_object(after_fence).ok_or_else(|| {
        InsightsError::InvalidWikiJson("no JSON object found in model reply".into())
    })?;

    serde_json::from_str(json).map_err(|e| InsightsError::InvalidWikiJson(e.to_string()))
}

fn extract_first_json_object(s: &str) -> Option<&str> {
    let start = s.find('{')?;
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    for (i, ch) in s[start..].char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[start..start + i + 1]);
                }
            }
            _ => {}
        }
    }
    None
}

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
    let mut sources_used = std::collections::HashSet::new();
    let mut entities_used = std::collections::HashSet::new();
    let mut concepts_used = std::collections::HashSet::new();

    let source_base = slugify(source_title);
    let source_slug_seg = uniquify_slug(&source_base, &mut sources_used);
    let source_slug = format!("sources/{source_slug_seg}");

    // Precompute entity/concept paths so the summary can forward-link.
    let mut entity_pages = Vec::with_capacity(analysis.entities.len());
    for entity in &analysis.entities {
        let seg = uniquify_slug(&slugify(&entity.name), &mut entities_used);
        let slug = format!("entities/{seg}");
        entity_pages.push((slug, entity));
    }
    let mut concept_pages = Vec::with_capacity(analysis.concepts.len());
    for concept in &analysis.concepts {
        let seg = uniquify_slug(&slugify(&concept.name), &mut concepts_used);
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

fn uniquify_slug(base: &str, used: &mut std::collections::HashSet<String>) -> String {
    if used.insert(base.to_string()) {
        return base.to_string();
    }
    let mut n = 2usize;
    loop {
        let candidate = format!("{base}-{n}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        n += 1;
    }
}

fn yaml_escape_double_quoted(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn build_frontmatter(title: &str, page_type_str: &str, source_uri: &str) -> String {
    let escaped_title = yaml_escape_double_quoted(title);
    let escaped_uri = yaml_escape_double_quoted(source_uri);
    format!(
        "---\ntitle: \"{escaped_title}\"\ntype: {page_type_str}\nsources: [\"{escaped_uri}\"]\ngenerated: true\n---\n"
    )
}

fn sanitize_display(display: &str) -> String {
    display
        .replace('|', "")
        .replace(']', "")
        .replace('\n', " ")
        .replace('\r', " ")
}

fn wikilink(path: &str, display: &str) -> String {
    format!("[[{path}|{}]]", sanitize_display(display))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunker::ChunkerConfig;
    use embedder::MockEmbedder;
    use llm::MockChatModel;
    use store::{NewChunk, SourceKind};

    fn sample_indexed(store: &Store, id: &str) {
        store
            .upsert_source(&store::Source {
                id: id.to_string(),
                kind: SourceKind::LocalFile,
                uri: format!("/tmp/{id}.md"),
                title: format!("Title {id}"),
                content_hash: "h".into(),
                indexed_at: Some(1),
                status: IndexStatus::Indexed,
                error: None,
                summary: None,
            })
            .unwrap();
        store
            .insert_chunks(
                id,
                &[NewChunk {
                    ord: 0,
                    text: "Jarvis wiki compile notes about local knowledge".into(),
                    loc: "L0".into(),
                    token_count: 5,
                    embedding: vec![0.1; 4],
                }],
            )
            .unwrap();
    }

    fn count_wiki_pages(store: &Store) -> usize {
        store
            .list_sources()
            .unwrap()
            .into_iter()
            .filter(|s| s.kind == SourceKind::WikiPage)
            .count()
    }

    fn any_content_md(wiki_root: &Path) -> bool {
        crate::wiki_has_exportable_notes(wiki_root)
    }

    #[tokio::test]
    async fn compile_rejects_when_disabled() {
        let store = Store::open_in_memory(4).unwrap();
        sample_indexed(&store, "dis-a");
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        let before = count_wiki_pages(&store);

        let err = compile_wiki_for_source(
            &store,
            &MockChatModel,
            &MockEmbedder::new(4),
            &ChunkerConfig::default(),
            "dis-a",
            &wiki_root,
            false,
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, InsightsError::WikiDisabled),
            "expected WikiDisabled, got {err:?}"
        );
        assert_eq!(
            count_files_recursive(&wiki_root),
            0,
            "disabled compile must create zero files"
        );
        assert_eq!(
            count_wiki_pages(&store),
            before,
            "disabled compile must not add WikiPage sources"
        );
    }

    #[tokio::test]
    async fn compile_rejects_wiki_page_input() {
        let store = Store::open_in_memory(4).unwrap();
        store
            .upsert_source(&store::Source {
                id: "wiki://sources/seed".into(),
                kind: SourceKind::WikiPage,
                uri: "wiki://sources/seed".into(),
                title: "Seed Wiki".into(),
                content_hash: "h".into(),
                indexed_at: Some(1),
                status: IndexStatus::Indexed,
                error: None,
                summary: None,
            })
            .unwrap();
        store
            .insert_chunks(
                "wiki://sources/seed",
                &[NewChunk {
                    ord: 0,
                    text: "seed wiki page body".into(),
                    loc: "L0".into(),
                    token_count: 3,
                    embedding: vec![0.1; 4],
                }],
            )
            .unwrap();

        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        let before_files = count_files_recursive(&wiki_root);

        let err = compile_wiki_for_source(
            &store,
            &MockChatModel,
            &MockEmbedder::new(4),
            &ChunkerConfig::default(),
            "wiki://sources/seed",
            &wiki_root,
            true,
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, InsightsError::WikiPageInput(_)),
            "expected WikiPageInput, got {err:?}"
        );
        assert_eq!(
            count_files_recursive(&wiki_root),
            before_files,
            "WikiPage input must create zero wiki files"
        );
    }

    #[tokio::test]
    async fn compile_writes_files_and_indexes() {
        let store = Store::open_in_memory(4).unwrap();
        sample_indexed(&store, "ok-a");
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");

        let summary = compile_wiki_for_source(
            &store,
            &MockChatModel,
            &MockEmbedder::new(4),
            &ChunkerConfig::default(),
            "ok-a",
            &wiki_root,
            true,
        )
        .await
        .expect("enabled compile should succeed");

        assert!(
            any_content_md(&wiki_root),
            "expected at least one sources|entities|concepts *.md"
        );
        assert!(
            wiki_root.join("index.md").is_file(),
            "index.md required after compile"
        );
        let wiki_pages: Vec<_> = store
            .list_sources()
            .unwrap()
            .into_iter()
            .filter(|s| s.kind == SourceKind::WikiPage)
            .collect();
        assert!(
            !wiki_pages.is_empty(),
            "expected at least one WikiPage source"
        );
        assert!(
            wiki_pages.iter().all(|s| s.uri.starts_with("wiki://")),
            "WikiPage uris must start with wiki://"
        );
        assert_eq!(summary.wiki_root, wiki_root);
    }

    #[tokio::test]
    async fn compile_idempotent_hash_skip() {
        let store = Store::open_in_memory(4).unwrap();
        sample_indexed(&store, "idem-a");
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        let embedder = MockEmbedder::new(4);
        let chunker = ChunkerConfig::default();

        compile_wiki_for_source(
            &store,
            &MockChatModel,
            &embedder,
            &chunker,
            "idem-a",
            &wiki_root,
            true,
        )
        .await
        .expect("first compile");
        let after_first = count_wiki_pages(&store);
        assert!(after_first > 0, "first compile must index WikiPages");

        compile_wiki_for_source(
            &store,
            &MockChatModel,
            &embedder,
            &chunker,
            "idem-a",
            &wiki_root,
            true,
        )
        .await
        .expect("second compile");
        assert_eq!(
            count_wiki_pages(&store),
            after_first,
            "identical re-compile must not increase WikiPage count"
        );
    }

    fn seed_generated_page(wiki_root: &Path, slug: &str, title: &str, page_type: &str) {
        let path = wiki_root.join(format!("{slug}.md"));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let body = format!(
            "---\ntitle: \"{title}\"\ntype: {page_type}\nsources: [\"file:///seed.md\"]\ngenerated: true\n---\n# {title}\n\nbody\n"
        );
        std::fs::write(path, body).unwrap();
    }

    #[test]
    fn rebuild_index_md_includes_multi_dir_pages() {
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        seed_generated_page(&wiki_root, "sources/a", "Source A", "source_summary");
        seed_generated_page(&wiki_root, "entities/b", "Entity B", "entity");

        rebuild_index_md_from_disk(&wiki_root).expect("rebuild");
        let index = std::fs::read_to_string(wiki_root.join("index.md")).unwrap();

        assert!(index.starts_with("# Wiki\n"), "expected # Wiki header: {index}");
        assert!(
            index.contains("## Sources"),
            "multi-dir vault must list Sources: {index}"
        );
        assert!(
            index.contains("## Entities"),
            "multi-dir vault must list Entities: {index}"
        );
        assert!(
            index.contains("[[sources/a|Source A]]"),
            "sources page missing: {index}"
        );
        assert!(
            index.contains("[[entities/b|Entity B]]"),
            "entities page missing: {index}"
        );
    }

    #[test]
    fn rebuild_index_md_lex_sorts_and_uses_title() {
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        // Insert out of order — catalog must sort by slug path.
        seed_generated_page(&wiki_root, "entities/zebra", "Zebra Co", "entity");
        seed_generated_page(&wiki_root, "entities/alpha", "Alpha Corp", "entity");

        rebuild_index_md_from_disk(&wiki_root).expect("rebuild");
        let index = std::fs::read_to_string(wiki_root.join("index.md")).unwrap();

        let alpha = index
            .find("[[entities/alpha|Alpha Corp]]")
            .expect("alpha link with title");
        let zebra = index
            .find("[[entities/zebra|Zebra Co]]")
            .expect("zebra link with title");
        assert!(
            alpha < zebra,
            "entities must be lexicographic by slug: {index}"
        );
    }

    #[test]
    fn rebuild_index_md_omits_empty_sections() {
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        seed_generated_page(&wiki_root, "sources/only", "Only Source", "source_summary");

        rebuild_index_md_from_disk(&wiki_root).expect("rebuild");
        let index = std::fs::read_to_string(wiki_root.join("index.md")).unwrap();

        assert!(index.contains("## Sources"), "sources section required: {index}");
        assert!(
            !index.contains("## Entities"),
            "empty Entities section must be omitted: {index}"
        );
        assert!(
            !index.contains("## Concepts"),
            "empty Concepts section must be omitted: {index}"
        );
    }

    #[test]
    fn rebuild_index_md_ignores_root_clutter() {
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        std::fs::create_dir_all(&wiki_root).unwrap();
        std::fs::write(
            wiki_root.join("note.md"),
            "---\ntitle: \"Root Note\"\ntype: entity\nsources: [\"file:///x\"]\ngenerated: true\n---\n# Root\n",
        )
        .unwrap();
        std::fs::write(wiki_root.join("index.md"), "# Old\n").unwrap();
        seed_generated_page(&wiki_root, "sources/kept", "Kept", "source_summary");

        rebuild_index_md_from_disk(&wiki_root).expect("rebuild");
        let index = std::fs::read_to_string(wiki_root.join("index.md")).unwrap();

        assert!(index.contains("# Wiki"), "rebuilt catalog header: {index}");
        assert!(
            index.contains("[[sources/kept|Kept]]"),
            "content page must appear: {index}"
        );
        assert!(
            !index.contains("[[note|") && !index.contains("Root Note"),
            "root note.md must not be a catalog entry: {index}"
        );
        assert!(
            !index.contains("index.md") && !index.contains("[[index|"),
            "index.md must never be listed as an entry: {index}"
        );
    }

    #[tokio::test]
    async fn compile_removes_stale_generated_page() {
        let store = Store::open_in_memory(4).unwrap();
        sample_indexed(&store, "stale-a");
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        let embedder = MockEmbedder::new(4);
        let chunker = ChunkerConfig::default();

        compile_wiki_for_source(
            &store,
            &MockChatModel,
            &embedder,
            &chunker,
            "stale-a",
            &wiki_root,
            true,
        )
        .await
        .expect("first compile");

        let source_uri = "/tmp/stale-a.md";
        let stale_slug = "entities/stale-gone";
        let stale_path = wiki_root.join(format!("{stale_slug}.md"));
        std::fs::create_dir_all(stale_path.parent().unwrap()).unwrap();
        let stale_body = format!(
            "---\ntitle: \"Stale Gone\"\ntype: entity\nsources: [\"{source_uri}\"]\ngenerated: true\n---\n# Stale Gone\n\nold\n"
        );
        std::fs::write(&stale_path, &stale_body).unwrap();
        let stale_uri = format!("wiki://{stale_slug}");
        let hash = ingest::hash_text(&stale_body);
        store
            .upsert_source(&store::Source {
                id: stale_uri.clone(),
                kind: SourceKind::WikiPage,
                uri: stale_uri.clone(),
                title: "Stale Gone".into(),
                content_hash: hash,
                indexed_at: Some(1),
                status: IndexStatus::Indexed,
                error: None,
                summary: None,
            })
            .unwrap();
        store
            .insert_chunks(
                &stale_uri,
                &[NewChunk {
                    ord: 0,
                    text: "stale entity body".into(),
                    loc: "L0".into(),
                    token_count: 3,
                    embedding: vec![0.1; 4],
                }],
            )
            .unwrap();
        assert!(stale_path.is_file());
        assert!(store.get_source(&stale_uri).is_ok());

        let summary = compile_wiki_for_source(
            &store,
            &MockChatModel,
            &embedder,
            &chunker,
            "stale-a",
            &wiki_root,
            true,
        )
        .await
        .expect("second compile cleans stale");

        assert!(
            summary.cleaned >= 1,
            "expected cleaned >= 1, got {}",
            summary.cleaned
        );
        assert!(
            !stale_path.exists(),
            "stale generated page must be removed from disk"
        );
        assert!(
            store.get_source(&stale_uri).is_err(),
            "stale wiki:// source must be deleted from Store"
        );
    }

    #[tokio::test]
    async fn compile_cleanup_and_user_edit_preserves_note() {
        let store = Store::open_in_memory(4).unwrap();
        sample_indexed(&store, "user-a");
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        let embedder = MockEmbedder::new(4);
        let chunker = ChunkerConfig::default();
        let source_uri = "/tmp/user-a.md";

        // User-curated note at the slug Mock will try to write (entities/jarvis).
        let user_slug = "entities/jarvis";
        let user_path = wiki_root.join(format!("{user_slug}.md"));
        std::fs::create_dir_all(user_path.parent().unwrap()).unwrap();
        let user_body = format!(
            "---\ntitle: \"My Jarvis Note\"\ntype: entity\nsources: [\"{source_uri}\"]\n---\n# My Jarvis Note\n\nUSER_CURATED_MARKER\n"
        );
        std::fs::write(&user_path, &user_body).unwrap();

        // Stale generated page owned by this source — must be cleaned (D-06…D-08).
        let stale_slug = "entities/stale-gone";
        let stale_path = wiki_root.join(format!("{stale_slug}.md"));
        let stale_body = format!(
            "---\ntitle: \"Stale Gone\"\ntype: entity\nsources: [\"{source_uri}\"]\ngenerated: true\n---\n# Stale Gone\n\nold\n"
        );
        std::fs::write(&stale_path, &stale_body).unwrap();
        let stale_uri = format!("wiki://{stale_slug}");
        store
            .upsert_source(&store::Source {
                id: stale_uri.clone(),
                kind: SourceKind::WikiPage,
                uri: stale_uri.clone(),
                title: "Stale Gone".into(),
                content_hash: ingest::hash_text(&stale_body),
                indexed_at: Some(1),
                status: IndexStatus::Indexed,
                error: None,
                summary: None,
            })
            .unwrap();
        store
            .insert_chunks(
                &stale_uri,
                &[NewChunk {
                    ord: 0,
                    text: "stale".into(),
                    loc: "L0".into(),
                    token_count: 1,
                    embedding: vec![0.1; 4],
                }],
            )
            .unwrap();

        let summary = compile_wiki_for_source(
            &store,
            &MockChatModel,
            &embedder,
            &chunker,
            "user-a",
            &wiki_root,
            true,
        )
        .await
        .expect("compile with user edit + stale");

        let after = std::fs::read_to_string(&user_path).unwrap();
        assert_eq!(after, user_body, "D-09: user-curated body must be unchanged");
        assert!(
            summary.skipped_user_edit >= 1,
            "expected skipped_user_edit >= 1, got {}",
            summary.skipped_user_edit
        );
        assert!(
            summary.cleaned >= 1,
            "expected cleaned >= 1, got {}",
            summary.cleaned
        );
        assert!(!stale_path.exists(), "stale generated page must be removed");
        assert!(store.get_source(&stale_uri).is_err());

        let user_uri = format!("wiki://{user_slug}");
        let indexed = store
            .get_source(&user_uri)
            .expect("D-12: user note must still be indexed");
        assert_eq!(indexed.kind, SourceKind::WikiPage);
        assert_eq!(indexed.status, IndexStatus::Indexed);
        assert_eq!(
            indexed.content_hash,
            ingest::hash_text(&user_body),
            "hash must be from on-disk user body, not draft"
        );
        let chunk_text = store.source_chunk_text(&user_uri).unwrap();
        assert!(
            chunk_text.contains("USER_CURATED_MARKER"),
            "indexed chunks must reflect on-disk user note"
        );
    }

    #[tokio::test]
    async fn compile_output_frontmatter_has_no_digest_key() {
        let store = Store::open_in_memory(4).unwrap();
        sample_indexed(&store, "fm-a");
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");

        compile_wiki_for_source(
            &store,
            &MockChatModel,
            &MockEmbedder::new(4),
            &ChunkerConfig::default(),
            "fm-a",
            &wiki_root,
            true,
        )
        .await
        .expect("compile");

        let page_path = wiki_root.join("sources").join("title-fm-a.md");
        assert!(page_path.is_file(), "expected source summary at {page_path:?}");
        let text = std::fs::read_to_string(&page_path).unwrap();
        let front = peek_frontmatter(&text).expect("YAML frontmatter required");
        for forbidden in [
            "content_hash",
            "contentHash",
            "digest",
            "hash:",
        ] {
            assert!(
                !front.to_lowercase().contains(&forbidden.to_lowercase().replace(':', "")),
                "D-10: frontmatter must not contain digest key '{forbidden}': {front}"
            );
        }
        // Phase 08 keys only.
        assert!(front.contains("title:"), "missing title: {front}");
        assert!(front.contains("type:"), "missing type: {front}");
        assert!(front.contains("sources:"), "missing sources: {front}");
        assert!(front.contains("generated:"), "missing generated: {front}");
    }

    #[tokio::test]
    async fn analyze_parses_mock_json() {
        let store = Store::open_in_memory(4).unwrap();
        sample_indexed(&store, "wiki-a");

        let analysis = analyze_source_for_wiki(&store, &MockChatModel, "wiki-a")
            .await
            .expect("mock wiki analyze");
        assert!(!analysis.summary.is_empty(), "summary must be non-empty");
        assert!(
            analysis.entities.len() >= 1,
            "expected at least one entity"
        );
    }

    #[tokio::test]
    async fn analyze_does_not_touch_source_summary() {
        let store = Store::open_in_memory(4).unwrap();
        sample_indexed(&store, "wiki-b");

        let before = store.get_source("wiki-b").unwrap().summary.clone();
        assert!(before.is_none());

        analyze_source_for_wiki(&store, &MockChatModel, "wiki-b")
            .await
            .expect("analyze");

        let after = store.get_source("wiki-b").unwrap().summary.clone();
        assert_eq!(after, before, "sources.summary must remain untouched");
        assert!(after.is_none());
    }

    #[test]
    fn parse_wiki_allows_empty_struct() {
        let raw = r#"{"summary":"","entities":[],"concepts":[]}"#;
        let analysis = parse_wiki_analysis(raw).expect("empty structural Ok");
        assert_eq!(analysis.summary, "");
        assert!(analysis.entities.is_empty());
        assert!(analysis.concepts.is_empty());
    }

    #[test]
    fn parse_wiki_accepts_fenced_or_chatter() {
        let fenced = "```json\n{\"summary\":\"s\",\"entities\":[],\"concepts\":[]}\n```";
        let a = parse_wiki_analysis(fenced).expect("fenced");
        assert_eq!(a.summary, "s");

        let chatter = "Here you go:\n{\"summary\":\"t\",\"entities\":[],\"concepts\":[]}\nThanks!";
        let b = parse_wiki_analysis(chatter).expect("chatter-prefixed");
        assert_eq!(b.summary, "t");
    }

    #[test]
    fn parse_wiki_rejects_truncated_object() {
        let err = parse_wiki_analysis(r#"{"summary":"partial""#).unwrap_err();
        assert!(
            matches!(err, InsightsError::InvalidWikiJson(_)),
            "expected InvalidWikiJson, got {err:?}"
        );
    }

    #[test]
    fn parse_wiki_rejects_schema() {
        let missing = parse_wiki_analysis(r#"{"summary":"x","entities":[]}"#).unwrap_err();
        assert!(matches!(missing, InsightsError::InvalidWikiJson(_)));

        let wrong_type =
            parse_wiki_analysis(r#"{"summary":1,"entities":[],"concepts":[]}"#).unwrap_err();
        assert!(matches!(wrong_type, InsightsError::InvalidWikiJson(_)));
    }

    #[test]
    fn write_wiki_pages_to_dir_writes_tree() {
        let analysis = WikiAnalysis {
            summary: "要点".into(),
            entities: vec![WikiEntity {
                name: "Acme".into(),
                blurb: "公司".into(),
            }],
            concepts: vec![],
        };
        let compiled = render_wiki_pages(&analysis, "file:///a.md", "Doc A");
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        write_wiki_pages_to_dir(&compiled, &wiki_root).expect("write tree");

        assert!(wiki_root.join("index.md").is_file(), "index.md required");
        let sources = wiki_root.join("sources");
        assert!(sources.is_dir(), "sources/ dir required");
        let source_pages: Vec<_> = std::fs::read_dir(&sources)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
            .collect();
        assert!(!source_pages.is_empty(), "at least one sources/*.md");
    }

    /// Test-only ChatModel that always returns a fixed reply (D-04 garbage fixtures).
    struct FixedReplyChat {
        reply: &'static str,
    }

    #[async_trait::async_trait]
    impl ChatModel for FixedReplyChat {
        fn id(&self) -> &str {
            "fixed-reply-test"
        }

        async fn complete(
            &self,
            _messages: &[llm::Message],
        ) -> std::result::Result<String, llm::LlmError> {
            Ok(self.reply.into())
        }

        async fn complete_stream(
            &self,
            messages: &[llm::Message],
            on_token: &mut (dyn FnMut(String) + Send),
        ) -> std::result::Result<String, llm::LlmError> {
            let answer = self.complete(messages).await?;
            on_token(answer.clone());
            Ok(answer)
        }
    }

    fn count_files_recursive(root: &Path) -> usize {
        if !root.exists() {
            return 0;
        }
        let mut n = 0usize;
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    n += 1;
                }
            }
        }
        n
    }

    async fn assert_fail_closed(reply: &'static str) {
        let store = Store::open_in_memory(4).unwrap();
        sample_indexed(&store, "fail-a");
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");

        let err = analyze_source_for_wiki(&store, &FixedReplyChat { reply }, "fail-a")
            .await
            .unwrap_err();
        assert!(
            matches!(err, InsightsError::InvalidWikiJson(_)),
            "expected InvalidWikiJson, got {err:?}"
        );
        // Fail-closed gate: never call write on Err.
        assert_eq!(
            count_files_recursive(&wiki_root),
            0,
            "parse failure must leave zero files under wiki root"
        );
    }

    #[tokio::test]
    async fn wiki_parse_fail_prose_writes_zero_files() {
        assert_fail_closed("这不是 JSON，只是散文。").await;
    }

    #[tokio::test]
    async fn wiki_parse_fail_truncated_writes_zero_files() {
        assert_fail_closed(r#"{"summary":"partial","entities":[{"name":"A""#).await;
    }

    #[tokio::test]
    async fn wiki_parse_fail_schema_writes_zero_files() {
        assert_fail_closed(r#"{"summary":"x","entities":"not-array","concepts":[]}"#).await;
    }

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
    fn render_cjk_entity_uses_hash_slug_path() {
        let analysis = WikiAnalysis {
            summary: "人物".into(),
            entities: vec![WikiEntity {
                name: "张三".into(),
                blurb: "某人".into(),
            }],
            concepts: vec![],
        };
        let out = render_wiki_pages(&analysis, "file:///a.md", "Doc A");
        let entity = out
            .pages
            .iter()
            .find(|p| p.page_type == WikiPageType::Entity)
            .expect("entity");
        assert!(
            entity.slug.starts_with("entities/e-"),
            "CJK entity slug: {}",
            entity.slug
        );
        let hex = entity.slug.trim_start_matches("entities/e-");
        assert_eq!(hex.len(), 6);
        assert!(
            out.pages[0]
                .body_markdown
                .contains(&format!("[[{}|张三]]", entity.slug)),
            "summary must wikilink CJK entity by path|display: {}",
            out.pages[0].body_markdown
        );
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
        let mut entities = std::collections::HashSet::new();
        let a = uniquify_slug(&slugify("Acme"), &mut entities);
        let b = uniquify_slug(&slugify("Acme"), &mut entities);
        assert_eq!(a, "acme");
        assert_eq!(b, "acme-2");

        let mut concepts = std::collections::HashSet::new();
        let c = uniquify_slug(&slugify("Acme"), &mut concepts);
        assert_eq!(c, "acme", "cross-directory names must not collide");
    }

    #[test]
    fn uniquify_reserves_emitted_suffix_slugs() {
        // Natural slug "foo-2" must not collide with the -2 suffix of a prior "foo".
        let mut used = std::collections::HashSet::new();
        assert_eq!(uniquify_slug("foo", &mut used), "foo");
        assert_eq!(uniquify_slug("foo", &mut used), "foo-2");
        assert_eq!(uniquify_slug("foo-2", &mut used), "foo-2-2");
        assert_eq!(used.len(), 3);
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

    #[test]
    fn index_omits_empty_sections() {
        let analysis = WikiAnalysis {
            summary: "only source".into(),
            entities: vec![],
            concepts: vec![],
        };
        let out = render_wiki_pages(&analysis, "file:///a.md", "Doc A");
        assert!(out.index_markdown.contains("# Wiki"));
        assert!(out.index_markdown.contains("## Sources"));
        assert!(!out.index_markdown.contains("## Entities"));
        assert!(!out.index_markdown.contains("## Concepts"));
    }

    #[test]
    fn index_with_all_sections() {
        let analysis = WikiAnalysis {
            summary: "full".into(),
            entities: vec![WikiEntity {
                name: "Acme".into(),
                blurb: "co".into(),
            }],
            concepts: vec![WikiConcept {
                name: "Foo".into(),
                blurb: "idea".into(),
            }],
        };
        let out = render_wiki_pages(&analysis, "file:///a.md", "Doc A");
        assert!(out.index_markdown.contains("## Sources"));
        assert!(out.index_markdown.contains("## Entities"));
        assert!(out.index_markdown.contains("## Concepts"));
        assert!(
            out.index_markdown.contains("[[sources/doc-a|Doc A]]"),
            "sources bullet: {}",
            out.index_markdown
        );
        assert!(
            out.index_markdown.contains("[[entities/acme|Acme]]"),
            "entities bullet: {}",
            out.index_markdown
        );
        assert!(
            out.index_markdown.contains("[[concepts/foo|Foo]]"),
            "concepts bullet: {}",
            out.index_markdown
        );
    }

    #[test]
    fn index_has_no_yaml_frontmatter() {
        let analysis = WikiAnalysis {
            summary: "x".into(),
            entities: vec![],
            concepts: vec![],
        };
        let out = render_wiki_pages(&analysis, "file:///a.md", "Doc A");
        assert!(
            !out.index_markdown.starts_with("---"),
            "index must not start with YAML frontmatter: {}",
            out.index_markdown
        );
    }
}
