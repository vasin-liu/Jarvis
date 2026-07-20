# Wiki Compile Layer (v1.10) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an optional, rebuildable Markdown wiki layer (entity/concept/source pages + Obsidian zip export) that sits beside existing RAG chunks — without replacing hybrid retrieval or citations.

**Architecture:** Extend `crates/insights` with a compile step that reads an indexed source + LLM output into structured wiki Markdown under `{app_data}/wiki/`. Persist wiki pages as first-class `SourceKind::WikiPage` documents through the existing ingest→index pipeline so RAG can retrieve them. Keep the feature behind `AppConfig.wiki.enabled` (default `false`). Export is a deterministic zip of the wiki tree plus a minimal `.obsidian/` stub.

**Tech Stack:** Rust crates (`insights`, `store`, `indexer`, `config`), Tauri commands, React Library UI, Vitest + `cargo test` + E2E (`JARVIS_E2E=1`).

## Global Constraints

- Do not start until v1.9 structural refactor is complete and E2E green (or explicitly bypassed by user).
- RAG remains authoritative: answers must still cite original chunks when available; wiki pages are additional sources, not a replacement.
- Wiki is rebuildable from sources via `content_hash`; no dual-write without hash skip.
- Default off: `wiki.enabled = false` — zero behavior change for existing users.
- Only `crates/store` opens SQLite.
- No LanceDB, no graph UI, no Deep Research, no Chrome clipper in this plan (see Out of Scope).
- User-facing changes require E2E + `data-testid`s.
- Mock LLM in unit/E2E — no live API calls in CI.

## Prerequisites

- [ ] Confirm `npm run test:e2e:local` green on `dev` after Phase 06.
- [ ] Confirm `SourceKind` / insights / Library patterns still match this plan (paths below assume post-refactor layout).

## File Map

| Path | Role |
|------|------|
| `crates/store/src/types.rs` | Add `SourceKind::WikiPage` |
| `crates/store/src/schema.rs` (or migrate) | Accept new kind string in DB |
| `crates/insights/src/wiki.rs` | Compile LLM JSON → Markdown pages + index.md |
| `crates/insights/src/lib.rs` | Export `compile_wiki_for_source` |
| `crates/config/src/types.rs` | `WikiConfig { enabled, auto_on_insights }` |
| `src-tauri/src/commands/` (insights or new `wiki.rs`) | `compile_wiki`, `export_wiki_zip`, `list_wiki_pages` |
| `src/views/LibraryView.tsx` | “生成笔记” + “导出 Wiki” when enabled |
| `src/lib/sourceDisplay.ts` | Label for `wiki_page` |
| `e2e/specs/wiki.spec.ts` | Journey: enable → compile → see page → export |
| `e2e/fixtures/` | Optional tiny md source already covered by sample.md |

---

### Task 1: `SourceKind::WikiPage` + config flag

**Files:**
- Modify: `crates/store/src/types.rs`
- Modify: `crates/config/src/types.rs` (and nested serde flatten if Phase 02 nesting is live)
- Modify: `src/lib/sourceDisplay.ts` + `src/lib/sourceDisplay.test.ts`
- Test: `crates/store` kind roundtrip; `crates/config` default/serde

**Interfaces:**
- Produces: `SourceKind::WikiPage` ↔ `"wiki_page"`
- Produces: 
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct WikiConfig {
    pub enabled: bool,
    /// When true, `compile_wiki_for_source` runs after successful summarize/insights.
    pub auto_on_insights: bool,
}
impl Default for WikiConfig {
    fn default() -> Self {
        Self { enabled: false, auto_on_insights: false }
    }
}
```
- Wire into `AppConfig` as `pub wiki: WikiConfig` with `#[serde(default)]` so old configs load.

- [ ] **Step 1: Write failing store test**

```rust
#[test]
fn wiki_page_kind_roundtrips() {
    assert_eq!(SourceKind::WikiPage.as_str(), "wiki_page");
    assert_eq!(SourceKind::parse("wiki_page"), Some(SourceKind::WikiPage));
}
```

- [ ] **Step 2: Run test — expect fail**

Run: `cargo test -p store wiki_page_kind_roundtrips -- --nocapture`  
Expected: compile fail or assertion fail (variant missing).

- [ ] **Step 3: Add enum variant + parse/as_str arms; update any exhaustive matches**

- [ ] **Step 4: Add `WikiConfig` + config roundtrip test (default false; unknown fields ignored)**

Run: `cargo test -p config`  
Expected: PASS

- [ ] **Step 5: Frontend label**

In `sourceDisplay.ts`, map `wiki_page` → `Wiki` / `笔记页`. Extend existing Vitest cases.

- [ ] **Step 6: Commit**

```bash
git add crates/store crates/config src/lib/sourceDisplay.ts src/lib/sourceDisplay.test.ts
git commit -m "feat(wiki): add WikiPage source kind and WikiConfig (default off)"
```

---

### Task 2: Wiki Markdown model + pure compiler (no LLM)

**Files:**
- Create: `crates/insights/src/wiki.rs`
- Modify: `crates/insights/src/lib.rs`
- Create: `crates/insights/tests/wiki_compile.rs` (or `#[cfg(test)]` in `wiki.rs`)

**Interfaces:**
- Produces:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiCompileResult {
    pub pages: Vec<WikiPageDraft>,
    pub index_markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiPageDraft {
    pub slug: String,           // e.g. "entities/acme"
    pub title: String,
    pub page_type: WikiPageType, // SourceSummary | Entity | Concept
    pub body_markdown: String,  // includes YAML frontmatter
    pub source_uris: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WikiPageType { SourceSummary, Entity, Concept }

/// Deterministic: structured analysis → markdown files. No I/O.
pub fn render_wiki_pages(analysis: &WikiAnalysis, source_uri: &str, source_title: &str) -> WikiCompileResult;

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
```

Frontmatter contract (every page):

```yaml
---
title: "Acme"
type: entity
sources: ["file:///path/to/doc.md"]
content_hash: "abc..."   # filled at write time
---
```

Body must include at least one `[[wikilink]]` from summary → entities when entities non-empty.

- [ ] **Step 1: Failing unit test — empty analysis still yields source summary page**

```rust
#[test]
fn render_always_emits_source_summary() {
    let analysis = WikiAnalysis {
        summary: "要点".into(),
        entities: vec![],
        concepts: vec![],
    };
    let out = render_wiki_pages(&analysis, "file:///a.md", "Doc A");
    assert_eq!(out.pages.len(), 1);
    assert!(out.pages[0].body_markdown.contains("sources:"));
    assert!(out.pages[0].body_markdown.contains("file:///a.md"));
    assert!(out.index_markdown.contains("Doc A"));
}
```

- [ ] **Step 2: Run — fail**

Run: `cargo test -p insights render_always_emits_source_summary`

- [ ] **Step 3: Implement `render_wiki_pages` + slugify (ASCII/safe path; CJK → hex or pinyin-free: use percent-ish slug `entity-{hash6}` plus title in frontmatter)**

Slug rule (lock in tests):  
`entities/{slugify(name)}` where `slugify` keeps `[a-z0-9-]` from unicode lowercase; if empty, use `e-{sha256_6(name)}`.

- [ ] **Step 4: Test entities emit wikilinks**

```rust
#[test]
fn render_links_entities_from_summary_page() {
    let analysis = WikiAnalysis {
        summary: "关于 Acme".into(),
        entities: vec![WikiEntity { name: "Acme".into(), blurb: "公司".into() }],
        concepts: vec![],
    };
    let out = render_wiki_pages(&analysis, "file:///a.md", "Doc A");
    assert!(out.pages.iter().any(|p| p.slug.starts_with("entities/")));
    let summary = out.pages.iter().find(|p| p.page_type == WikiPageType::SourceSummary).unwrap();
    assert!(summary.body_markdown.contains("[[Acme]]") || summary.body_markdown.contains("[[entities/"));
}
```

- [ ] **Step 5: Commit**

```bash
git add crates/insights
git commit -m "feat(insights): deterministic wiki markdown renderer"
```

---

### Task 3: LLM analysis → `WikiAnalysis` (Mock-friendly)

**Files:**
- Modify: `crates/insights/src/wiki.rs`
- Modify: `crates/llm` only if a shared JSON helper is needed (prefer keep parsing in insights)

**Interfaces:**
```rust
pub async fn analyze_source_for_wiki(
    store: &Store,
    chat: &dyn ChatModel,
    source_id: &str,
) -> Result<WikiAnalysis>;
```

Prompt contract: system asks for **bare JSON only**:

```json
{
  "summary": "...",
  "entities": [{"name":"...","blurb":"..."}],
  "concepts": [{"name":"...","blurb":"..."}]
}
```

Reuse `truncate_chars` (12_000). On parse failure → `InsightsError::Parse` (add variant if missing) — do not write partial wiki.

- [ ] **Step 1: Failing test with `MockChatModel` returning fixed JSON**

```rust
#[tokio::test]
async fn analyze_parses_mock_json() {
    // upsert indexed source + chunks as in summarize tests
    let chat = MockChatModel::new(vec![
        r#"{"summary":"S","entities":[{"name":"E","blurb":"b"}],"concepts":[]}"#.into(),
    ]);
    let analysis = analyze_source_for_wiki(&store, &chat, "s1").await.unwrap();
    assert_eq!(analysis.entities[0].name, "E");
}
```

Adjust `MockChatModel` constructor to whatever the crate already exposes (see `crates/llm/src/mock.rs`).

- [ ] **Step 2: Implement analyze + JSON extract (strip fences if present; prefer first `{...}` object)**

- [ ] **Step 3: `cargo test -p insights` green**

- [ ] **Step 4: Commit**

```bash
git add crates/insights
git commit -m "feat(insights): LLM wiki analysis with Mock coverage"
```

---

### Task 4: Persist wiki to disk + index as `WikiPage` sources

**Files:**
- Modify: `crates/insights/src/wiki.rs`
- Modify: `src-tauri` insights glue (e.g. `insights_ops.rs` or `commands/insights.rs`)
- Possibly: `crates/indexer` only via existing `index_document`

**Interfaces:**
```rust
pub struct WikiPaths {
    pub root: PathBuf, // {app_data}/wiki
}

pub async fn compile_wiki_for_source(
    store: &Store,
    chat: &dyn ChatModel,
    embedder: &dyn Embedder,
    source_id: &str,
    wiki_root: &Path,
) -> Result<WikiCompileResult>;
```

Behavior:
1. Load source; skip if not `Indexed`.
2. `analyze_source_for_wiki` → `render_wiki_pages`.
3. Write files under `wiki_root/{slug}.md`; merge/update `wiki_root/index.md`.
4. For each page, build `ingest::Document` (or store `NewSource` + chunks via existing indexer helper) with:
   - `kind: WikiPage`
   - `uri: wiki://{slug}` (stable)
   - `content_hash: sha256(body)`
   - skip re-embed when hash unchanged (indexer already skips)
5. Do not delete user-edited pages in this milestone unless hash matches previous auto-generated marker — **v1.10 rule:** overwrite only pages whose frontmatter contains `generated: true`.

Frontmatter add: `generated: true`.

- [ ] **Step 1: Integration test with tempfile + MockChatModel + MockEmbedder**

```rust
#[tokio::test]
async fn compile_writes_files_and_indexes() {
    let dir = tempfile::tempdir().unwrap();
    let wiki = dir.path().join("wiki");
    // ... open Store, index a local source, mock chat JSON ...
    compile_wiki_for_source(&store, &chat, &embedder, "s1", &wiki).await.unwrap();
    assert!(wiki.join("index.md").is_file());
    let sources = store.list_sources().unwrap();
    assert!(sources.iter().any(|s| s.kind == SourceKind::WikiPage));
}
```

- [ ] **Step 2: Implement write + index wiring**

- [ ] **Step 3: Hash skip test — second compile with same mock output does not duplicate sources (same uri upsert)**

- [ ] **Step 4: Commit**

```bash
git add crates/insights crates/store src-tauri
git commit -m "feat(insights): compile wiki to disk and index WikiPage sources"
```

---

### Task 5: Tauri IPC + Library UI (feature-flagged)

**Files:**
- Create or modify: `src-tauri/src/commands/wiki.rs` (or extend insights commands)
- Modify: `src-tauri/src/lib.rs` / `commands/mod.rs` registration
- Modify: `src/views/LibraryView.tsx`, `src/hooks/useLibrary.ts` if needed
- Modify: Settings accordion — toggle `wiki.enabled` (reuse SettingsView pattern)
- Types: `src/types/ipc.ts` or config types

**IPC (camelCase serde):**
- `compile_wiki({ sourceId: string }) -> { pageCount: number }`
- `export_wiki_zip({ destPath: string }) -> { path: string }` (Task 6 can stub export)
- Commands no-op / return clear error if `!config.wiki.enabled`

**UI:**
- Settings: checkbox `启用 Wiki 笔记层` → `data-testid="wiki-enabled-toggle"`
- Library per-source button when enabled + indexed: `生成笔记` → `data-testid="wiki-compile-{id}"` or shared `wiki-compile` with data-source-id
- Show wiki pages in library list with kind label (existing list path)

- [ ] **Step 1: Vitest — Library shows compile button only when `config.wiki.enabled`**

- [ ] **Step 2: Wire invoke + busy/error via existing patterns**

- [ ] **Step 3: Manual smoke / E2E in Task 7**

- [ ] **Step 4: Commit**

```bash
git add src-tauri src
git commit -m "feat(ui): wiki compile controls behind WikiConfig.enabled"
```

---

### Task 6: Obsidian zip export

**Files:**
- Create: `crates/insights/src/wiki_export.rs` (or functions in `wiki.rs`)
- IPC: implement `export_wiki_zip`
- UI: Settings or Library button `导出 Wiki` → `data-testid="wiki-export"`

**Interfaces:**
```rust
pub fn export_wiki_zip(wiki_root: &Path, dest_zip: &Path) -> Result<()>;
```

Zip contents:
- all files under `wiki_root/`
- `.obsidian/app.json` minimal `{"legacyEditor":false}`
- no `target/`, no secrets

Use a small zip crate already in workspace if present; otherwise add `zip = "2"` workspace dep deliberately.

- [ ] **Step 1: Unit test — tempfile wiki with one md → zip → unzip list contains `index.md` and `.obsidian/app.json`**

- [ ] **Step 2: Implement + IPC + dialog save path via existing `@tauri-apps/plugin-dialog`**

- [ ] **Step 3: Commit**

```bash
git add crates/insights src-tauri src Cargo.toml Cargo.lock
git commit -m "feat(wiki): export Obsidian-compatible zip"
```

---

### Task 7: E2E journey + validation docs

**Files:**
- Create: `e2e/specs/wiki.spec.ts`
- Modify: `e2e/specs/full-ui.spec.ts` — short section or skip if flag off by default (assert controls hidden)
- Modify: `.planning/REQUIREMENTS.md` only if running GSD milestone (optional)
- Update: `.cursor/rules/e2e-required.mdc` spec map row for Wiki

**E2E flow (`JARVIS_E2E=1`):**
1. Open Settings → enable wiki toggle → save if required
2. Open Library → compile on fixture source
3. Assert a wiki_page appears OR file list / toast success (`data-testid="wiki-compile-done"`)
4. Export zip to temp path → assert invoke success
5. Ask RAG a question that Mock can answer — no regression on `qa.spec` / agent

Mock chat must return valid wiki JSON when compile runs — extend `src-tauri/src/e2e.rs` Mock responses if agent/RAG mocks collide (prefer dedicated mock sequence for wiki compile command).

- [ ] **Step 1: Write failing E2E spec**

- [ ] **Step 2: Implement `data-testid`s + e2e mock hooks**

- [ ] **Step 3: Run `npm run test:e2e:local`**

Expected: wiki.spec pass; full-ui / qa / agent still pass

- [ ] **Step 4: Commit**

```bash
git add e2e src-tauri src
git commit -m "test(e2e): wiki compile and export journey"
```

---

## Out of Scope (later milestones)

| Milestone | Item |
|-----------|------|
| v1.11 | Related-docs panel (source overlap); read-only MCP `search` / `list_sources` |
| v1.12 | PDF/MinerU enhancements; Deep Research as optional agent skill |
| Later | Chrome clipper; Louvain graph UI; bidirectional Obsidian sync; purpose.md as separate product surface |

## Success Criteria

- [ ] `wiki.enabled=false` → identical UX to pre-feature (E2E full-ui green)
- [ ] `wiki.enabled=true` → compile creates markdown + `WikiPage` sources; export zip opens as folder with `index.md`
- [ ] Re-compile same source is idempotent on `wiki://` URIs
- [ ] Citations still resolve to original non-wiki sources in existing qa journeys
- [ ] `cargo test -p insights -p store -p config` and `npm run test:e2e:local` green

## Self-Review

1. **Spec coverage:** Optional compile layer, Obsidian export, index as sources, default off, no graph/Lance/clipper — all tasked or explicitly out of scope.
2. **Placeholders:** None intentional; MockChatModel constructor must be matched to live API when executing.
3. **Types:** `WikiAnalysis` / `WikiPageDraft` / `WikiConfig` / `SourceKind::WikiPage` naming consistent across tasks.

---

## Execution Handoff

Plan saved to `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md`.

**Recommended next steps:**
1. Finish / gate v1.9 refactor + E2E.
2. Run `/gsd-new-milestone` with name `v1.10 Wiki Compile Layer` and paste this plan’s success criteria into REQUIREMENTS.
3. Execute with `/gsd-execute-phase` or subagent-driven development starting at Task 1.

Or say **执行 Task 1** to begin implementation in-repo now (only if you accept parallel work during refactor).
