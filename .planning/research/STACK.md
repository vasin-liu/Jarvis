# Stack Research

**Domain:** Local-first knowledge hub — Wiki Compile Layer (v1.10)
**Researched:** 2026-07-17
**Confidence:** HIGH

## Recommended Stack

### Core Technologies (preserved — do not replace)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust stable | MSRV 1.85 | Domain logic | Existing workspace |
| Tauri 2 | workspace pin | Desktop shell / IPC | Existing shell |
| React 19 + Vite 7 | package.json | Library/Settings UI | Existing FE |
| SQLite + FTS5 + sqlite-vec | rusqlite 0.32 | Index wiki pages as sources | Single DB owner via `store` |
| `ChatModel` / Mock | crates/llm | Wiki analysis LLM | Trait injection + E2E mocks |

### Supporting Libraries (NEW for v1.10)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `zip` | **2.4.2** (stable; avoid `9.0.0-pre2`) | Write Obsidian vault zip | Export only; prefer `deflate` feature subset if size matters |
| `walkdir` | **2.5.0** | Walk `wiki/` tree for zip | Export; or `std::fs` recurse if tree stays shallow |
| `tempfile` | **3.27.0** (or workspace pin) | Temp dirs in export/unit tests | Tests only — already common in workspace |
| Hand-rolled slugify | — | `entities/{slug}` paths | Prefer over `slug` crate (CJK → empty slug; plan uses hash fallback) |
| Hand-rolled YAML frontmatter | — | `title` / `type` / `sources` / `content_hash` | Avoid `serde_yaml` (deprecated on crates.io) |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `cargo test -p insights` | Renderer + analyze unit tests | MockChatModel |
| Vitest | `sourceDisplay` wiki_page label | Existing |
| WebdriverIO + `JARVIS_E2E=1` | `e2e/specs/wiki.spec.ts` | Deterministic mocks |

## Installation

```toml
# workspace Cargo.toml [workspace.dependencies]
zip = { version = "2.4.2", default-features = false, features = ["deflate"] }
walkdir = "2.5"

# crates/insights (or src-tauri if export lives in shell):
# zip = { workspace = true }
# walkdir = { workspace = true }
```

No new npm packages required for v1.10 wiki UI (existing React + Tabler).

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `zip` 2.4.2 | `zip` 9.x pre | Never in product until stable release |
| Hand YAML strings | `serde_yaml` 0.9 | Avoid — marked deprecated; frontmatter is tiny |
| Hand YAML strings | `yaml_serde` / `serde_yml` | Only if round-trip parse becomes required |
| Hand slugify | `slug` 0.1.6 | OK if tests cover CJK empty-slug → hash fallback |
| `walkdir` | Manual `read_dir` recurse | Fine for shallow `wiki/{entities,concepts,sources}` |
| Store wiki as files + `SourceKind::WikiPage` | Separate LanceDB / graph DB | Out of scope — breaks single-store rule |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| LanceDB / vector DB swap | Dual index, ops cost | Existing sqlite-vec pipeline |
| Graph UI / Louvain crates | Scope creep | Obsidian viewer after zip export |
| `serde_yaml` | Deprecated upstream | Format frontmatter with `format!` / small helper |
| Bidirectional Obsidian sync crates | Conflict / merge complexity | Export-only zip |
| Native Obsidian plugin host | Not a desktop embedding target | Zip vault for external Obsidian |
| Live LLM in CI | Flaky / cost | MockChatModel + `JARVIS_E2E=1` |

## Stack Patterns by Variant

**If export stays in `insights` crate:**
- Depend on `zip` + `walkdir` there; Tauri command stays thin.

**If export stays in `src-tauri`:**
- Keep pure Markdown compile in `insights`; shell owns zip I/O + dialog path — still no SQLite outside `store`.

**If CJK entity names dominate:**
- Slug = `e-{sha256_6(name)}` when ASCII slug empty; title stays in frontmatter/body.

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `zip` 2.4.2 | MSRV 1.73+ | Fine under workspace 1.85 |
| `walkdir` 2.5 | any recent Rust | Stable API |
| Existing `sha2` / `hex` | slug hash fallback | Already in workspace |

## Sources

- crates.io `cargo search` / `cargo info zip` — stable **2.4.2**, latest pre **9.0.0-pre2** (2026-07-17)
- Plan: `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md`
- Workspace: root `Cargo.toml`, `crates/insights`

---
*Stack research for: Wiki Compile Layer (Jarvis v1.10)*
*Researched: 2026-07-17*
