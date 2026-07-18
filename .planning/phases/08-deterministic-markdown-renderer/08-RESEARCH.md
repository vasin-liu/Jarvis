# Phase 08: Deterministic Markdown renderer - Research

**Researched:** 2026-07-18  
**Domain:** Pure Rust Markdown compile (`WikiAnalysis` → page drafts + index string)  
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Wikilink style
- **D-01:** Forward links use Obsidian alias form **`[[path|display]]`**, e.g. `[[entities/acme|Acme]]`, `[[concepts/foo|Foo]]`.
- **D-02:** Linking is **bidirectional**: source-summary body links to each entity/concept page; each entity/concept body links back to the source summary.
- **D-03:** Back-link form is **`[[sources/{slug}|{source_title}]]`** (same path|display convention).

#### Page tree / slugs
- **D-04:** Output tree layout: **`sources/`**, **`entities/`**, **`concepts/`**, plus root-level **`index.md`** content (as `index_markdown` string in Phase 08).
- **D-05:** Slugify rule: keep `[a-z0-9-]` from unicode lowercase of the name; if empty after filtering, use **`e-{sha256_6(name)}`** (6 hex chars). Title/display name stays in frontmatter and link display text — never require pinyin crates.
- **D-06:** Source-summary slug uses **the same slugify rules** as entity/concept names (applied to `source_title`).
- **D-07:** Same-directory slug collisions: deterministic suffix **`-2`, `-3`, …** by input order within that directory. Cross-directory same name (entity vs concept) is **not** a collision.

#### Frontmatter (Phase 08 drafts)
- **D-08:** Every page draft body includes YAML frontmatter with **`title`**, **`type`**, **`sources`**, **`generated: true`**. **Omit `content_hash`** in Phase 08 (filled at write time in Phase 10).
- **D-09:** `type` literals are snake_case: **`source_summary`**, **`entity`**, **`concept`** (aligned with `WikiPageType` serde).
- **D-10:** `sources` is a **one-line JSON-style array**, e.g. `sources: ["file:///a.md"]`.

#### index.md shape (`index_markdown`)
- **D-11:** Structure: `# Wiki` heading, then sections **`## Sources`**, **`## Entities`**, **`## Concepts`** with bullet `[[path|display]]` entries.
- **D-12:** **Omit empty sections** (no entities → no `## Entities` block).
- **D-13:** No YAML frontmatter on the index string (catalog only, not a `WikiPageDraft`).

### Claude's Discretion
- Exact body prose layout under frontmatter (headings for blurb/summary text) as long as D-01…D-03 links and D-08…D-10 frontmatter hold.
- Whether slugify lives in `wiki.rs` vs a tiny helper module inside `insights`.
- Test placement: `#[cfg(test)]` in `wiki.rs` vs `crates/insights/tests/` — follow crate conventions.
- SHA-256 truncation helper reuse of workspace `sha2`/`hex`.

### Deferred Ideas (OUT OF SCOPE)
- Filling / overwriting `content_hash` and disk merge of `index.md` — Phase 10
- LLM JSON → `WikiAnalysis` — Phase 09
- Settings / Library compile UX — Phase 11
- Obsidian zip — Phase 12
- Title characters that break wikilink alias syntax (`|`, `]`) escaping policy — not deeply specified; implementer should sanitize display text conservatively if needed (note for planner, not a new phase)

None — discussion stayed within phase scope (no folded todos)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WIKI-03 (render half) | Compile indexed source into Markdown wiki pages (source summary + entities + concepts) with YAML frontmatter, `[[wikilinks]]`, and `index.md` | Pure `render_wiki_pages` in `crates/insights`; types from plan Task 2; locked D-01…D-13 override plan’s bare `[[Acme]]` / optional `content_hash` examples |
</phase_requirements>

## Summary

Phase 08 delivers a **pure, deterministic** function `render_wiki_pages(analysis, source_uri, source_title) -> WikiCompileResult` inside `crates/insights` — no LLM, no filesystem, no Tauri. [CITED: `.planning/ROADMAP.md` Phase 08; `08-CONTEXT.md`]

Interfaces are already sketched in the wiki compile plan (`WikiAnalysis`, `WikiPageDraft`, `WikiPageType`, `WikiCompileResult`). Discussion **overrides** that sketch for link style (`[[path|display]]`), directory tree (`sources/` / `entities/` / `concepts/`), frontmatter (`generated: true`, **omit** `content_hash`), and index catalog shape. [CITED: `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` Task 2; `08-CONTEXT.md` D-01…D-13]

Stack additions are **workspace `sha2` + `hex` only** (already used by `ingest` / `embedder`). Hand-roll slugify and YAML frontmatter strings — do **not** add `serde_yaml` (deprecated on crates.io) or a `slug` crate. [VERIFIED: workspace `Cargo.toml`; `cargo search`; crates.io `serde_yaml` 0.9.34+deprecated; `.planning/research/STACK.md`]

**Primary recommendation:** Create `crates/insights/src/wiki.rs` with TDD unit tests matching ROADMAP success criteria; re-export from `lib.rs`; add `sha2`/`hex` as `{ workspace = true }`; implement slugify + collision suffixes + bidirectional `[[path|display]]` + frontmatter helper as pure string builders.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `WikiAnalysis` → Markdown drafts | Library / domain (`insights`) | — | Pure transform; no UI, no IPC |
| Slugify + collision resolution | Library / domain (`insights`) | — | Path segments must be ASCII-safe before Phase 10 FS write |
| YAML frontmatter string build | Library / domain (`insights`) | — | Tiny fixed schema; no YAML parser needed |
| Wikilink generation | Library / domain (`insights`) | — | Obsidian-compatible text only |
| `index_markdown` catalog | Library / domain (`insights`) | — | In-memory string; disk merge is Phase 10 |
| Persist / index / UI / LLM | Deferred (09–11) | — | Explicitly out of Phase 08 |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust `stable` | MSRV 1.85 | Implementation language | Workspace pin [VERIFIED: `Cargo.toml`, `rust-toolchain.toml`] |
| `serde` / `serde_json` | workspace | Derive on wiki types | Already on `insights` [VERIFIED: `crates/insights/Cargo.toml`] |
| `sha2` | workspace `"0.10"` (crates.io latest seen `0.11.0`; **stay on workspace 0.10**) | SHA-256 for empty-slug fallback | Existing Jarvis pattern [VERIFIED: root `Cargo.toml`; `crates/ingest/src/hash.rs`; docs.rs `sha2` 0.10.9] |
| `hex` | workspace `"0.4"` (crates.io `0.4.3`) | Encode digest → hex | Paired with `sha2` in ingest [VERIFIED] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `thiserror` | workspace | Only if renderer gains typed errors | **Not required** for pure infallible render [ASSUMED: function returns `WikiCompileResult` not `Result`] |
| `tempfile` | — | FS tests | **Do not use in Phase 08** — no disk I/O [CITED: CONTEXT phase boundary] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand YAML `format!` | `serde_yaml` 0.9 | Deprecated / unmaintained — **reject** [VERIFIED: crates.io `0.9.34+deprecated`] |
| Hand YAML | `serde_yml` / `noyalib` | Overkill for 4 fixed keys; adds deps [ASSUMED] |
| Hand slugify | `slug` crate | Still needs empty/CJK → hash fallback; extra dep for little gain [CITED: `.planning/research/STACK.md`] |
| Bare `[[Name]]` links | `[[path\|display]]` | Locked D-01; plan Task 2 test is outdated — update tests to path\|display |

**Installation (insights only — no new registry packages):**

```toml
# crates/insights/Cargo.toml
sha2 = { workspace = true }
hex = { workspace = true }
```

**Version verification:** `cargo search sha2` → `0.11.0` latest; workspace pins `0.10`. `hex` → `0.4.3`; workspace `0.4`. Do **not** bump workspace pins in this phase. [VERIFIED: 2026-07-18 `cargo search`]

## Package Legitimacy Audit

> Adding existing workspace deps to `insights` — not introducing new package names.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| `sha2` | crates | since 2016 | ~15.5M/wk | github.com/RustCrypto/hashes | OK | Approved — workspace pin 0.10 |
| `hex` | crates | since 2015 | ~9.3M/wk | github.com/KokaKiwi/rust-hex | OK | Approved — workspace pin 0.4 |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none  
**Do not install:** `serde_yaml`, `slug`, pinyin crates [CITED: CONTEXT D-05; STACK.md]

## Architecture Patterns

### System Architecture Diagram

```
WikiAnalysis { summary, entities[], concepts[] }
        + source_uri + source_title
                    │
                    ▼
         ┌──────────────────────┐
         │  slugify + uniquify  │  per directory: sources|entities|concepts
         │  (D-05…D-07)         │
         └──────────┬───────────┘
                    ▼
         ┌──────────────────────┐
         │  build page drafts   │
         │  frontmatter D-08–10 │
         │  body + [[path|disp]]│  bidirectional D-01…D-03
         └──────────┬───────────┘
                    ▼
         ┌──────────────────────┐
         │  build index_markdown│  # Wiki + optional sections D-11…D-13
         └──────────┬───────────┘
                    ▼
         WikiCompileResult { pages: Vec<WikiPageDraft>, index_markdown }
                    │
        Phase 09 fills analysis │ Phase 10 writes disk + content_hash
```

### Recommended Project Structure

```
crates/insights/
├── Cargo.toml          # + sha2, hex workspace deps
├── src/
│   ├── lib.rs          # mod wiki; pub use render types/fn
│   ├── wiki.rs         # NEW: types + slugify + render_wiki_pages + #[cfg(test)]
│   ├── summarize.rs    # existing (pattern only — has Store/LLM)
│   ├── tasks.rs        # existing
│   └── error.rs
└── (no tests/ dir today — prefer inline tests)
```

[VERIFIED: `crates/insights/src/` has only `error`, `summarize`, `tasks`; no `tests/` directory]

### Pattern 1: Pure render then impure persist

**What:** `render_wiki_pages` is pure and unit-tested; Phase 10 owns FS + `content_hash` + index.  
**When to use:** Always for this phase.  
**Source:** [CITED: `.planning/research/ARCHITECTURE.md` Pattern 3]

### Pattern 2: Match ingest hash helper for `e-{hash6}`

**What:** Reuse the same `Sha256::digest` + `hex::encode` approach as `ingest::hash_text`, then take first 6 hex chars.  
**Example (adapt):**

```rust
// Source: crates/ingest/src/hash.rs [VERIFIED]
use sha2::{Digest, Sha256};

fn hash6(name: &str) -> String {
    let digest = Sha256::digest(name.as_bytes());
    let full = hex::encode(digest);
    full[..6].to_string()
}
```

Hash **UTF-8 bytes of the original name** (not the filtered slug), so CJK names are stable. [ASSUMED: plan says `sha256_6(name)` — name string, not empty slug]

### Pattern 3: Serde snake_case page types

```rust
// Source: docs/superpowers/plans/2026-07-16-wiki-compile-layer.md Task 2 [CITED]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WikiPageType {
    SourceSummary, // → "source_summary"
    Entity,        // → "entity"
    Concept,       // → "concept"
}
```

Frontmatter `type:` must emit these literals via a small match/`as_str`, not Debug print. [CITED: D-09]

### Pattern 4: Insights crate test style

Prefer `#[cfg(test)] mod tests` colocated in `wiki.rs` (same as `summarize.rs` / `tasks.rs`). Integration `tests/` directory does not exist yet — do not invent one unless tests need private-API isolation. [VERIFIED: crate layout]

### Anti-Patterns to Avoid

- **Copying plan Task 2 tests verbatim:** They accept `[[Acme]]` OR path links and show `content_hash` in frontmatter — both contradict D-01 / D-08. Rewrite tests to locked contract. [CITED]
- **Calling Store / ChatModel from renderer:** Summarize/tasks do I/O+LLM; wiki render must not. [CITED: phase boundary]
- **Using entity `name` as filesystem path:** CJK / `/` / `\` break Windows and Obsidian. [CITED: PITFALLS.md Pitfall 4]
- **Including empty `## Entities` / `## Concepts`:** Violates D-12.
- **Putting `WikiPageType` in `store`:** Draft enum lives in insights for compile; `SourceKind::WikiPage` already shipped in Phase 07 — different concept. [VERIFIED: `store::SourceKind::WikiPage`]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SHA-256 | Custom hash | Workspace `sha2` + `hex` | Crypto correctness; already in tree [VERIFIED] |
| Full YAML engine | Custom YAML parser | Tiny `format!` frontmatter block | Only 4 keys; serde_yaml deprecated [VERIFIED] |
| Obsidian plugin / vault API | Embed Obsidian | Plain Markdown + wikilink text | Export is Phase 12; render is strings only [CITED] |
| Unicode → pinyin slug | Pinyin crate | `e-{hash6}` fallback (D-05) | Locked; no new deps [CITED] |

**Key insight:** The “hard” parts (CJK paths, idempotent hash, Obsidian compatibility) are solved by **ASCII slugs + display aliases + deferring FS hash to Phase 10** — not by new libraries.

## Common Pitfalls

### Pitfall 1: CJK / unsafe names → empty or illegal paths

**What goes wrong:** `张三` or `Acme/Corp` → empty slug, `entities/.md`, or path separators in slug.  
**Why:** Filtering non-ASCII without hash fallback; keeping `/` `\`.  
**How to avoid:** Unit tests for CJK, slash, empty-after-filter; assert `e-` + 6 hex; assert no `/` in slug segment. [CITED: PITFALLS.md; ROADMAP criterion 3]  
**Warning signs:** Wikilink path ≠ draft `slug` field.

### Pitfall 2: Plan Task 2 / frontmatter drift

**What goes wrong:** Tests assert `content_hash` present or bare `[[Acme]]`; Phase 10 later assumes omit-then-fill.  
**Why:** Older plan examples predate discuss-phase.  
**How to avoid:** Treat `08-CONTEXT.md` as source of truth over Task 2 prose; update example assertions. [CITED]

### Pitfall 3: Same-directory collisions / cross-directory false collisions

**What goes wrong:** Two entities named “Acme” overwrite; or entity+concept both “Acme” incorrectly get `-2`.  
**Why:** Global uniquify vs per-directory (D-07).  
**How to avoid:** Separate collision maps for `sources/`, `entities/`, `concepts/`; suffix by **input order**. [CITED: D-07]

### Pitfall 4: Display text breaks `[[…|…]]`

**What goes wrong:** Title contains `|` or `]` → malformed wikilink.  
**Why:** Unspecified escape policy (deferred note).  
**How to avoid:** Conservative sanitize in discretion (e.g. strip/replace `|` and `]` in **display** only; path stays slug). Document chosen rule in SUMMARY. [CITED: deferred; ASSUMED: strip is enough for v1]

### Pitfall 5: Accidental I/O or LLM in Phase 08

**What goes wrong:** “Helpful” wiring to Store or analyze.  
**Why:** Copying summarize/tasks patterns.  
**How to avoid:** `render_wiki_pages` signature takes only `&WikiAnalysis` + two `&str`; no `Store`/`ChatModel` params. [CITED]

### Pitfall 6: Index always lists empty sections

**What goes wrong:** Empty `## Entities` clutter.  
**How to avoid:** Build sections only when `pages` of that type exist (D-12). Always include Sources when source summary exists (always). [CITED]

## Code Examples

### Interface (locked shape from plan + CONTEXT overrides)

```rust
// Adapted from docs/superpowers/plans/2026-07-16-wiki-compile-layer.md Task 2 [CITED]
// Overrides: slug paths under sources|entities|concepts; no content_hash in body yet.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiCompileResult {
    pub pages: Vec<WikiPageDraft>,
    pub index_markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiPageDraft {
    pub slug: String, // e.g. "entities/acme", "sources/doc-a"
    pub title: String,
    pub page_type: WikiPageType,
    pub body_markdown: String, // YAML frontmatter + body
    pub source_uris: Vec<String>,
}

pub fn render_wiki_pages(
    analysis: &WikiAnalysis,
    source_uri: &str,
    source_title: &str,
) -> WikiCompileResult;
```

### Frontmatter helper (hand-rolled)

```rust
// Recommended shape per D-08…D-10 [CITED]
fn frontmatter(title: &str, page_type: &str, source_uri: &str) -> String {
    format!(
        "---\ntitle: \"{title}\"\ntype: {page_type}\nsources: [\"{source_uri}\"]\ngenerated: true\n---\n"
    )
}
// Escape " in title if needed [ASSUMED: replace " with ' or backslash]
```

### Slugify (locked algorithm)

```rust
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
            _ => {} // drop CJK / punctuation / path seps
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        format!("e-{}", hash6(name))
    } else {
        trimmed
    }
}
```

Whitespace→`-` handling is **Claude's discretion** within D-05’s keep `[a-z0-9-]`; recommend collapsing runs and trimming edges. [ASSUMED]

### Failing tests to plan first (TDD)

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
    let p = &out.pages[0];
    assert!(p.slug.starts_with("sources/"));
    assert!(p.body_markdown.contains("sources:"));
    assert!(p.body_markdown.contains("file:///a.md"));
    assert!(p.body_markdown.contains("generated: true"));
    assert!(!p.body_markdown.contains("content_hash"));
    assert!(out.index_markdown.contains("# Wiki"));
    assert!(out.index_markdown.contains("## Sources"));
    assert!(!out.index_markdown.contains("## Entities"));
}

#[test]
fn render_links_entities_bidirectional() {
    let analysis = WikiAnalysis {
        summary: "关于 Acme".into(),
        entities: vec![WikiEntity {
            name: "Acme".into(),
            blurb: "公司".into(),
        }],
        concepts: vec![],
    };
    let out = render_wiki_pages(&analysis, "file:///a.md", "Doc A");
    let summary = out
        .pages
        .iter()
        .find(|p| p.page_type == WikiPageType::SourceSummary)
        .unwrap();
    assert!(summary.body_markdown.contains("[[entities/acme|Acme]]"));
    let entity = out
        .pages
        .iter()
        .find(|p| p.slug.starts_with("entities/"))
        .unwrap();
    assert!(entity.body_markdown.contains("[[sources/"));
    assert!(entity.body_markdown.contains("|Doc A]]"));
}

#[test]
fn cjk_name_uses_hash_slug() {
    let analysis = WikiAnalysis {
        summary: "s".into(),
        entities: vec![WikiEntity {
            name: "张三".into(),
            blurb: "人".into(),
        }],
        concepts: vec![],
    };
    let out = render_wiki_pages(&analysis, "file:///a.md", "Doc");
    let ent = out.pages.iter().find(|p| p.page_type == WikiPageType::Entity).unwrap();
    assert!(ent.slug.starts_with("entities/e-"));
    assert_eq!(ent.slug.len(), "entities/e-".len() + 6);
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Bare `[[Title]]` in early plan draft | `[[path\|display]]` (D-01) | 2026-07-18 discuss | Stable links when title ≠ path |
| Frontmatter includes `content_hash` at render | Omit until write (D-08) | 2026-07-18 discuss | Pure render; hash of final body at Phase 10 |
| Flat slugs | `sources/` `entities/` `concepts/` tree (D-04) | 2026-07-18 discuss | Matches Obsidian vault layout / zip later |

**Deprecated/outdated:**
- `serde_yaml` for frontmatter — crates.io deprecated 2024-03-25 [VERIFIED: crates.io / dtolnay release notes]
- Plan Task 2 assertion allowing only `[[Acme]]` — superseded by D-01

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `render_wiki_pages` is infallible (`WikiCompileResult`, not `Result`) | Standard Stack | Planner may omit error type — low risk |
| A2 | Hash input is original `name` UTF-8 bytes | Pattern 2 | Slug instability if someone hashes slugified form |
| A3 | Whitespace in ASCII names collapses to `-` | Slugify example | Mild path churn; lock in tests |
| A4 | Display sanitize = strip `\|` and `]` | Pitfall 4 | Broken wikilinks for pathological titles |
| A5 | YAML title quoting uses escaped/replaced `"` | Frontmatter helper | Invalid YAML if titles contain `"` |

**If wrong:** Prefer discuss-phase follow-up only for A4/A5 if implementer wants a different escape policy; A1–A3 are implementer discretion under CONTEXT.

## Open Questions

1. **Exact body headings under frontmatter**
   - What we know: Discretion allows any layout if links + frontmatter hold.
   - Recommendation: `# {title}` then summary/blurb paragraph, then `## Links` (or inline list) for wikilinks — keep minimal.

2. **Source title collision with entity slug**
   - What we know: Cross-directory not a collision (D-07).
   - Recommendation: No special case.

3. **Multiple `source_uris` on draft**
   - What we know: Plan type has `Vec<String>`; Phase 08 single-source compile → one URI.
   - Recommendation: `source_uris: vec![source_uri.to_string()]`; frontmatter array length 1.

## Environment Availability

Step 2.6: **SKIPPED for external services** — Phase 08 is code-only (no LLM, Feishu, zip tools). Local toolchain verified:

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` / `rustc` | unit tests | ✓ | rustc 1.97.0 / cargo 1.97.0 | — |
| workspace `sha2`/`hex` | slug hash | ✓ | 0.10 / 0.4 | — |

**Missing dependencies with no fallback:** none

## Validation Architecture

> `workflow.nyquist_validation: true` in `.planning/config.json` [VERIFIED]

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (lib unit tests via `#[cfg(test)]`) |
| Config file | none — Cargo workspace default |
| Quick run command | `cargo test -p insights wiki -- --test-threads=1` |
| Full suite command | `cargo test -p insights -- --test-threads=1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WIKI-03 | Empty analysis → 1 source-summary with `sources:` + `generated: true`, no `content_hash` | unit | `cargo test -p insights render_always_emits_source_summary -- --exact` | ❌ Wave 0 |
| WIKI-03 | Entities → `entities/` draft + `[[entities/…\|…]]` from summary + backlink | unit | `cargo test -p insights render_links_entities_bidirectional -- --exact` | ❌ Wave 0 |
| WIKI-03 | Concepts mirror entity link rules | unit | `cargo test -p insights render_links_concepts_…` | ❌ Wave 0 |
| WIKI-03 | CJK → `e-{6 hex}` slug | unit | `cargo test -p insights cjk_name_uses_hash_slug -- --exact` | ❌ Wave 0 |
| WIKI-03 | Same-dir collision → `-2` suffix | unit | `cargo test -p insights slug_collision_suffix -- --exact` | ❌ Wave 0 |
| WIKI-03 | Index omits empty sections | unit | `cargo test -p insights index_omits_empty_sections -- --exact` | ❌ Wave 0 |
| — | E2E / UI | — | N/A this phase (no user-facing UI) | skip |

### Sampling Rate

- **Per task commit:** `cargo test -p insights wiki -- --test-threads=1`
- **Per wave merge:** `cargo test -p insights -- --test-threads=1`
- **Phase gate:** Same + manual review that no `std::fs` / `Store` / `ChatModel` in `wiki.rs`

### Wave 0 Gaps

- [ ] `crates/insights/src/wiki.rs` with `#[cfg(test)]` covering ROADMAP criteria 1–3 + collisions + index omit-empty
- [ ] `crates/insights/Cargo.toml` — add `sha2` / `hex` workspace deps
- [ ] `crates/insights/src/lib.rs` — `mod wiki` + pub re-exports
- [ ] Framework install: none (Cargo already available)

*(No E2E Wave 0 — Phase 08 is library-only; E2E is Phase 13 / WIKI-09.)*

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Slugify strips path seps / non-`[a-z0-9-]`; never use raw name as path segment |
| V6 Cryptography | yes (hash only) | Workspace `sha2` — never hand-roll SHA-256 |

### Known Threat Patterns for Markdown wiki render

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via entity name (`../`, `\`) | Tampering | Slug filter drops non-allowed chars; `/` `\` never enter slug [CITED: PITFALLS] |
| Wikilink injection / broken parse via `\|` `]` in titles | Tampering | Sanitize display text (discretion) |
| YAML injection via quotes in title | Tampering | Escape/replace `"` in frontmatter title field |
| Partial tree write on bad LLM JSON | — | Out of scope (Phase 09 WIKI-05); render assumes valid `WikiAnalysis` |

## Project Constraints (from .cursor/rules/)

| Rule | Directive for Phase 08 |
|------|------------------------|
| `tdd-goal-driven.mdc` | State goal + acceptance; **failing tests first**; unit layer for pure chunker-like logic (renderer fits); run `cargo test -p insights` before done |
| `karpathy-guidelines.mdc` | No speculative features; surgical diff; honor locked CONTEXT; simplicity (hand frontmatter, not YAML crate) |
| `jarvis-stack.mdc` | Rust stable / MSRV 1.85; core logic in library crates; no new pre-release crates |
| `e2e-required.mdc` | User-facing features need E2E — **Phase 08 is not user-facing UI**; no E2E required this phase; later phases (11/13) cover journeys |
| `frontend-taste.mdc` | N/A — no `src/**` UI work |

## Handoff Notes (not in scope — for planner awareness only)

| Later phase | Expects from Phase 08 |
|-------------|----------------------|
| 09 | Same `WikiAnalysis` / entity / concept types for LLM JSON parse |
| 10 | `WikiPageDraft.slug` + `body_markdown` writable under `{app_data}/wiki/`; inject `content_hash`; merge `index_markdown` |
| 12 | Same tree shape for zip |

## Sources

### Primary (HIGH confidence)

- [VERIFIED] Workspace: `Cargo.toml` (`sha2 = "0.10"`, `hex = "0.4"`), `crates/ingest/src/hash.rs`, `crates/insights/{Cargo.toml,src/lib.rs}`
- [VERIFIED] `cargo search` / package-legitimacy: `sha2` OK, `hex` OK (2026-07-18)
- [VERIFIED] docs.rs `sha2` 0.10.9 — `Sha256::digest` / `Digest` trait
- [CITED] `.planning/phases/08-deterministic-markdown-renderer/08-CONTEXT.md` — locked D-01…D-13
- [CITED] `.planning/ROADMAP.md` Phase 08 success criteria
- [CITED] `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` Task 2 interfaces
- [CITED] `.planning/research/{SUMMARY,STACK,FEATURES,PITFALLS,ARCHITECTURE}.md`

### Secondary (MEDIUM confidence)

- [CITED] crates.io `serde_yaml` 0.9.34+deprecated + dtolnay archive note (2024-03-25)
- [CITED] Obsidian help / community docs — `[[Note\|Display]]` alias form (websearch 2026-07-18; help.obsidian.md fetch timed out — cross-checked via Mintlify mirrors + forum)

### Tertiary (LOW confidence)

- [ASSUMED] Exact whitespace→hyphen collapsing and display-character sanitize policy (discretion)

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — workspace deps verified; no new packages
- Architecture: **HIGH** — interfaces + tree locked in CONTEXT; crate home clear
- Pitfalls: **HIGH** — milestone PITFALLS + discuss overrides align
- Obsidian link dialect: **MEDIUM** — official help fetch timeout; alias form widely documented

**Research date:** 2026-07-18  
**Valid until:** 2026-08-17 (stable domain; re-check only if workspace bumps `sha2` major)
