# Pitfalls Research

**Domain:** LLM wiki compile layer beside existing RAG (Jarvis v1.10)
**Researched:** 2026-07-17
**Confidence:** HIGH

**Plan:** `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md`  
**Constraints:** `wiki.enabled=false` by default; RAG authoritative; rebuildable via `content_hash`; Mock LLM in tests; only `crates/store` opens SQLite.

**Phase key** (v1.10 continues after v1.9 Phase 06):

| Phase | Focus | Plan task |
|-------|--------|-----------|
| **07** | `SourceKind::WikiPage` + `WikiConfig` (default off) | Task 1 |
| **08** | Deterministic Markdown renderer + safe slugs | Task 2 |
| **09** | LLM → `WikiAnalysis` (Mock-friendly JSON parse) | Task 3 |
| **10** | Persist wiki files + index as sources (`content_hash`) | Task 4 |
| **11** | Tauri IPC + Library/Settings UI (feature-flagged) | Task 5 |
| **12** | Obsidian zip export | Task 6 |
| **13** | E2E journey + RAG/citation regression gate | Task 7 |

---

## Critical Pitfalls

### Pitfall 1: Dual-write / re-index without `content_hash` skip

**What goes wrong:**
Every wiki compile creates new `WikiPage` rows or re-embeds identical Markdown. Library fills with duplicates; embed cost spikes; `wiki://` URIs drift; rebuild is no longer idempotent.

**Why it happens:**
Compile path builds a fresh `Document` each time and calls `index_document` without setting a stable URI + `content_hash`, or bypasses the indexer skip that already exists for unchanged hashes (`crates/indexer` skips when `existing.content_hash == doc.content_hash` and status is Indexed).

**How to avoid:**
- Stable URI: `wiki://{slug}` for every page.
- `content_hash: sha256(body)` (or equivalent) at write time; store in frontmatter and on the source row.
- Reuse `indexer::index_document` — never a parallel insert path.
- Integration test: compile twice with same Mock output → same source count, no duplicate URIs.

**Warning signs:**
- Second compile increases `list_sources()` WikiPage count.
- Embedder called on unchanged bodies in tests/logs.
- Multiple Library rows for the same title/slug.

**Phase to address:**
**10** (persist/index); verify again in **13**.

---

### Pitfall 2: Wiki pages replace or crowd out original citations

**What goes wrong:**
RAG answers cite `wiki_page` / `wiki://…` instead of the user’s original files. Trust in “answers come from my data” erodes; E2E QA journeys pass for the wrong reason (Mock answers from wiki text).

**Why it happens:**
Wiki pages are indexed into the same hybrid retriever. RRF promotes dense summary pages over sparse original chunks. Prompt/citation UI treats all `SourceKind` equally. Product copy implies wiki is the knowledge base.

**How to avoid:**
- Product rule: **RAG remains authoritative** — wiki is additional context, not a citation substitute.
- Prefer citing non-wiki sources when both rank; if a wiki page is cited, require/link through frontmatter `sources:`.
- Do not change `rag` / retriever defaults to “wiki-first.”
- Keep `qa.spec` / `full-ui` asserting answers still resolve to fixture original sources when wiki is off; with wiki on, assert originals still appear when relevant.
- Never make wiki the only indexed representation of a source.

**Warning signs:**
- Citation chips show only “Wiki” / `wiki://`.
- Disabling wiki changes answer quality dramatically (should be additive).
- Agent `search_knowledge` results dominated by WikiPage after compile.

**Phase to address:**
**10** (index semantics), **11** (UI labeling), **13** (E2E citation regression).

---

### Pitfall 3: Default-on or auto-compile breaks existing installs

**What goes wrong:**
Upgrade enables wiki or `auto_on_insights` without consent → surprise LLM calls, disk writes under `wiki/`, Library clutter, slower insights path, cost for cloud users.

**Why it happens:**
`WikiConfig` defaults flipped to `true`, or `#[serde(default)]` missing so missing fields deserialize oddly; `auto_on_insights` wired into `insights_ops` without checking `enabled`; Settings toggle defaults checked.

**How to avoid:**
- `WikiConfig { enabled: false, auto_on_insights: false }` via `Default` + `#[serde(default)]` on nested field.
- Golden config test: load pre-v1.10 `config.json` → `wiki.enabled == false`.
- Commands return clear error / no-op when `!enabled`.
- UI: compile/export controls hidden unless enabled (`data-testid` absent in default E2E).
- `full-ui.spec.ts`: with default config, wiki controls not shown.

**Warning signs:**
- Fresh clone / upgrade runs compile during summarize without user action.
- Cloud API usage jumps after upgrade with wiki never toggled.
- E2E full-ui fails only when wiki UI unexpectedly appears.

**Phase to address:**
**07** (config defaults), **11** (UI gating), **13** (default-off E2E).

---

### Pitfall 4: CJK / unsafe slug paths break Windows filesystem and Obsidian

**What goes wrong:**
Entity names like `张三` or `Acme/Corp` become path components → empty slug, invalid Windows paths, path separators in filenames, or Obsidian links that don’t resolve. Export zip contains broken entries.

**Why it happens:**
Naïve `slugify` that strips non-ASCII to empty string, or keeps `/`, `\`, `:`, control chars. Title used as filename without fallback hash.

**How to avoid:**
- Lock plan rule in tests: keep `[a-z0-9-]`; if empty after slugify → `e-{sha256_6(name)}` (and likewise for concepts).
- Never use raw user/LLM strings as path segments.
- Title lives in frontmatter / display; filesystem slug is ASCII-safe.
- Windows CI / tempfile tests with CJK and slash-containing names.

**Warning signs:**
- Compile fails only on Chinese sources.
- `entities/.md` or empty filename on disk.
- Wikilinks in body don’t match on-disk paths.

**Phase to address:**
**08** (renderer/slugify); re-check in **12** (zip paths).

---

### Pitfall 5: LLM JSON parse failure still writes a partial wiki

**What goes wrong:**
Model returns prose, truncated JSON, or fenced junk. Code writes half the pages / updates `index.md` / indexes incomplete `WikiPage`s. User sees corrupt notes; re-compile compounds mess.

**Why it happens:**
Happy-path-only parsing; “best effort” write after soft parse errors; streaming tokens flushed to disk; per-entity write loop continues after first failure.

**How to avoid:**
- Parse to complete `WikiAnalysis` first; on failure → `InsightsError::Parse` (or equivalent) and **abort before any disk/index write**.
- Strip fences / extract first `{…}` object; unit-test garbage, truncated, and non-JSON Mock replies.
- Atomicity preference: write drafts to temp then rename, or transactional “all pages or none” for a compile run.
- Never leave `generated: true` pages half-updated without rolling back index upserts for that run when possible.

**Warning signs:**
- `index.md` updated but entity files missing after an error toast.
- Store has WikiPage rows with empty/placeholder bodies after failed compile.
- Tests only cover valid Mock JSON.

**Phase to address:**
**09** (parse), **10** (write gating).

---

### Pitfall 6: Zip export path traversal / unsafe zip entries

**What goes wrong:**
Malicious or buggy relative paths under `wiki_root` (or crafted page slugs) produce zip entries like `../../config.json` or absolute paths. Unzip outside intended folder; user secrets or app data exposed/overwritten when opening in Obsidian tooling.

**Why it happens:**
Walking `wiki_root` with `strip_prefix` skipped; using slug strings directly as zip entry names without normalization; including files outside wiki via symlink follow.

**How to avoid:**
- Canonicalize each file path; refuse if not under `wiki_root`.
- Zip entry names = relative paths only; reject `..` components.
- Do not follow symlinks out of root (or disallow symlinks).
- Unit test: planted `../escape.md` style path must fail or be excluded; happy path includes `index.md` + `.obsidian/app.json` only from intended tree.
- Dest path from dialog is the **output zip**, not a write into wiki root with user-controlled join.

**Warning signs:**
- Zip listing shows absolute paths or `..`.
- Export “succeeds” with files from parent of `wiki/`.
- No tests around zip entry names.

**Phase to address:**
**12** primarily; slug safety in **08** reduces input surface.

---

### Pitfall 7: Indexing wiki pages triggers infinite compile loops

**What goes wrong:**
Post-index / insights / watcher hooks see new `WikiPage` sources and call `compile_wiki_for_source` again → more pages → more compiles. CPU, disk, and LLM burn; Library explodes.

**Why it happens:**
`auto_on_insights` or scheduler treats all Indexed sources equally; compile indexes WikiPages which re-enter the same hook; watch folder includes `{app_data}/wiki`.

**How to avoid:**
- **Never** auto-compile `SourceKind::WikiPage` (and ideally never treat wiki URIs as compile inputs).
- Gate auto path: `enabled && auto_on_insights && kind != WikiPage`.
- Keep wiki root **outside** user watch folders by default; document that watching `wiki/` is unsupported.
- Integration test: after compile, insights/auto hook does not recurse.

**Warning signs:**
- Compile progress never settles; WikiPage count grows without new user sources.
- Scheduler CPU high after enabling auto wiki.
- Nested `wiki://` sources as inputs to analyze.

**Phase to address:**
**10** (index wiring + hook guards), **11** if auto toggle lands in UI.

---

### Pitfall 8: Store ownership violations (SQLite outside `crates/store`)

**What goes wrong:**
Wiki feature “quickly” opens `rusqlite` in `insights` or `src-tauri` for page metadata → lock contention with indexer/scheduler, missed migrations, FTS/vec inconsistency.

**Why it happens:**
New metadata tables or ad-hoc queries seem easier than extending `Store`; copy-paste from tests that open connections.

**How to avoid:**
- Persist wiki **as sources** via existing Store + indexer APIs only.
- Any new query = new `Store` method.
- PR checklist: no `rusqlite` in `insights` / `src-tauri` production code.
- Architecture review at end of **10** / **13**.

**Warning signs:**
- `rusqlite` appears in `crates/insights/Cargo.toml` or `src-tauri` beyond tests.
- `SQLITE_BUSY` during compile + sync.
- Wiki metadata not visible through `list_sources`.

**Phase to address:**
**10** (and any phase touching persistence); **All 07–13** as review gate.

---

### Pitfall 9: E2E / unit tests without Mock LLM (live API in CI)

**What goes wrong:**
Wiki compile E2E hits real Ollama/cloud → flaky, slow, costly, or secret-dependent CI. Unit tests skip parse/compile coverage. Green locally with API key, red in Windows CI.

**Why it happens:**
Compile command uses ambient `ChatModel` without ensuring `JARVIS_E2E=1` Mock sequence returns wiki JSON; Mock still returns RAG prose; live provider left as default in test config.

**How to avoid:**
- Unit/integration: `MockChatModel` with fixed wiki JSON (plan Task 3/4).
- E2E: extend `src-tauri/src/e2e.rs` so compile path gets dedicated Mock responses (don’t collide with ask/agent sequences).
- Never call live LLM in `cargo test` or `npm run test:e2e`.
- Spec map row for `wiki.spec.ts` in e2e rules.

**Warning signs:**
- E2E requires network or API key.
- Compile step times out waiting for tokens.
- `analyze_source_for_wiki` untested except manually.

**Phase to address:**
**09** (Mock unit), **13** (E2E mocks + journey).

---

### Pitfall 10: Overwriting user-edited wiki pages

**What goes wrong:**
User edits a note in Obsidian or on disk; next compile clobbers edits. Trust in export/edit workflow dies.

**Why it happens:**
Compile always overwrites `wiki_root/{slug}.md` without checking `generated: true` frontmatter (plan v1.10 rule).

**How to avoid:**
- Overwrite only pages with `generated: true` in frontmatter.
- Skip or rename collision for user pages lacking the marker.
- Test: hand-written file without marker survives re-compile.

**Warning signs:**
- User reports lost edits after “生成笔记.”
- No frontmatter `generated` field in written pages.

**Phase to address:**
**10** (write policy).

---

### Pitfall 11: Exhaustive `SourceKind` / UI match arms missed

**What goes wrong:**
Compile fails or panics; Library shows blank kind; serde rejects `wiki_page` from DB; frontend label missing.

**Why it happens:**
New enum variant added in `store` without updating all Rust `match` arms and `sourceDisplay.ts`.

**How to avoid:**
- Roundtrip test for `wiki_page`; compile-fail or tests on exhaustive matches.
- Vitest for `sourceDisplay` label.
- Grep for `SourceKind::` matches when adding variant.

**Warning signs:**
- `non-exhaustive patterns` only caught late; runtime “unknown kind” in UI.

**Phase to address:**
**07**.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Index wiki into same FTS/vec without citation policy | Fast “wiki searchable” | Wiki crowds citations | Never without Phase 13 citation checks |
| Soft-fail LLM parse + write what we can | Something on disk | Corrupt wiki tree | Never |
| `auto_on_insights=true` by default | “Magic” UX | Surprise LLM cost / loops | Never in v1.10 |
| Put wiki under a watched folder | Convenient sync | Compile loops | Never by default |
| Open SQLite in insights for wiki meta | Quick metadata | Store ownership break | Never |
| Live LLM in E2E “just this once” | Faster local demo | Flaky CI | Never |
| Slug = raw Unicode title | Readable paths | Windows/Obsidian breakage | Never — use hash fallback |
| Skip `generated: true` check | Simpler writer | Destroys user edits | Never for overwrite |

## Integration Gotchas

Common mistakes when connecting wiki to Jarvis subsystems.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `indexer::index_document` | Insert WikiPage without hash skip | Stable `wiki://` URI + `content_hash`; rely on existing skip |
| `insights_ops` / post-index | Auto-compile every Indexed source | Exclude `WikiPage`; require `wiki.enabled` |
| `ChatModel` / Mock | Same Mock reply as RAG ask | Dedicated wiki JSON sequence for compile |
| `AppConfig` serde | Flat `wiki_enabled` without nested default | `wiki: WikiConfig` + `#[serde(default)]` |
| Library UI | Always show “生成笔记” | Gate on `config.wiki.enabled` |
| Watcher / scheduler | Watch `{app_data}/wiki` | Keep wiki root out of watch folders |
| Retriever / RAG | Prefer wiki chunks in prompt | Original sources authoritative; wiki additive |
| Zip / dialog | Join user path into wiki tree unsafely | Export writes zip at dest; entries relative + sanitized |
| Store | Ad-hoc connection for wiki tables | Only `SourceKind::WikiPage` via Store APIs |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Re-embed every compile | Slow compile, fan spin | `content_hash` skip | >50 pages or cloud embed |
| Unbounded entities/concepts from LLM | Huge wiki trees, long index | Cap entities/concepts in prompt + truncate input (12_000 chars) | Noisy long docs |
| Compile all sources on enable | UI freeze on large libraries | Per-source explicit compile only in v1.10 | Libraries with hundreds of sources |
| Full wiki re-index on every export | Export takes minutes | Export = zip files only, no re-embed | Large `wiki/` trees |
| Auto-compile on every insight | LLM queue backlog | Default `auto_on_insights=false`; debounce | Continuous sync + insights |

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Zip path traversal | Overwrite files outside export dir / leak paths | Canonicalize under `wiki_root`; reject `..` |
| LLM-injected path/slug | Write outside wiki root | Slug allowlist + hash fallback; never trust model for paths |
| Export includes secrets | API keys / config in zip | Zip only `wiki_root` + stub `.obsidian/`; no `config.json` |
| Wiki prompt includes secrets from source | Leak via cloud LLM | Existing truncate; don’t special-case paste of key files into compile |
| Symlink escape from wiki root | Read arbitrary files into zip | Don’t follow outbound symlinks |

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Wiki on by default | Surprise cost and clutter | Default off; explicit Settings toggle |
| No busy/error on compile | Stuck button / silent fail | Loading + `chat-error`-style error + done testid |
| Wiki kind unlabeled | Confusing Library list | `wiki_page` → clear “Wiki” / “笔记页” label |
| Citations look like “made up notes” | Distrust | Prefer original citations; show wiki as derived |
| Clobbering Obsidian edits | Lost work | `generated: true` overwrite rule |
| Export with empty wiki | Confusing empty vault | Disable export or clear empty-state message |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Default off:** Pre-v1.10 config loads with `wiki.enabled == false` — verify serde test + full-ui hidden controls
- [ ] **Idempotent compile:** Second compile same Mock output does not duplicate `wiki://` sources — verify integration test
- [ ] **Parse abort:** Invalid LLM JSON writes zero files — verify negative unit test
- [ ] **No compile loop:** WikiPage sources never auto-compile — verify hook guard test
- [ ] **CJK slugs:** Chinese entity names produce safe ASCII paths — verify unit test
- [ ] **Zip safety:** Entries have no `..` or absolute paths — verify export unit test
- [ ] **RAG authority:** QA E2E still cites fixture originals with wiki enabled — verify `qa` / wiki / full-ui
- [ ] **E2E mocks:** `JARVIS_E2E=1` compile uses Mock wiki JSON — verify `e2e.rs` sequence
- [ ] **Store ownership:** No new `rusqlite` outside `store` — verify crate deps / review
- [ ] **User edit preserve:** Pages without `generated: true` not overwritten — verify write policy test
- [ ] **UI testids:** `wiki-enabled-toggle`, compile, export, done — verify `wiki.spec.ts`
- [ ] **Exhaustive kind:** All `SourceKind` matches + `sourceDisplay` updated — verify compile + Vitest

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Duplicate WikiPage sources | MEDIUM | Delete WikiPage sources by kind/URI; wipe `{app_data}/wiki`; re-compile once |
| Corrupt partial wiki | LOW | Delete `wiki/` tree + WikiPage sources; fix parse; re-compile |
| Compile loop | HIGH | Set `wiki.enabled=false` / `auto_on_insights=false`; kill runaway; remove wiki from watch folders; delete excess WikiPages |
| Citation regression | MEDIUM | Disable wiki; confirm QA green; add citation preference; re-enable |
| Zip traversal incident | HIGH | Stop using export; rotate any exposed secrets; patch sanitizer; re-export |
| User edits clobbered | HIGH | Restore from Obsidian/backup/zip if any; enforce `generated` flag; document |
| Default-on upgrade | LOW | Ship config patch forcing `enabled=false`; communicate toggle location |
| Store ownership mess | HIGH | Remove rogue connections; migrate any ad-hoc tables into Store; run integrity checks |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Dual-write without `content_hash` | **10** | Double-compile integration test; source count stable |
| Wiki replaces citations | **10**, **13** | QA/full-ui still cite originals; optional citation policy test |
| Default-on / surprise auto | **07**, **11**, **13** | Config default test; controls hidden; full-ui green |
| CJK / unsafe slugs | **08**, **12** | Slug unit tests; zip lists safe relative paths |
| Partial wiki on parse fail | **09**, **10** | Invalid JSON → no files / no new sources |
| Zip path traversal | **12** | Escape-path unit test fails closed |
| Infinite compile loop | **10**, **11** | Auto-hook ignores WikiPage; no watch on wiki root |
| Store ownership violation | **10** (+ review all) | No rusqlite outside store; architecture checklist |
| E2E without mocks | **09**, **13** | Mock unit + `wiki.spec.ts` offline CI |
| Overwrite user edits | **10** | Non-`generated` file survives re-compile |
| Missed `SourceKind` arms | **07** | Roundtrip + UI label + compile |

## Sources

- Plan: `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` (global constraints, Tasks 1–7)
- Project constraints: `.planning/PROJECT.md` (v1.10 Wiki Compile Layer)
- Indexer hash skip: `crates/indexer` `index_document` unchanged-hash early return
- Architecture invariants: single DB owner; normalize-to-Document; trait providers; E2E mocks
- Prior brownfield pitfalls: `.planning/research/PITFALLS.md` (v1.9 archive context — superseded for v1.10 focus)
- Community / domain: LLM JSON fragility; zip-slip class bugs; secondary index poisoning RAG citations

---
*Pitfalls research for: Jarvis Wiki Compile Layer (v1.10)*
*Researched: 2026-07-17*
*)
