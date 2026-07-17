# Feature Research

**Domain:** LLM-compiled personal wiki layer beside local-first RAG (Jarvis v1.10)
**Researched:** 2026-07-17
**Confidence:** HIGH

## Feature Landscape

### How LLM-compiled wikis typically work

Pattern (Karpathy LLM Wiki → `nashsu/llm_wiki`, Obsidian vaults, Memo/Reflect-style entity pages):

1. **Three layers** — immutable raw sources → LLM-maintained Markdown wiki → schema/rules (how pages look).
2. **Compile, don't re-derive** — ingest once into interlinked pages; later queries read the wiki (or chat over it). Competitors often *replace* ad-hoc RAG with this compiled layer.
3. **Page types** — source summaries, entities, concepts; YAML frontmatter; `[[wikilinks]]`; catalog `index.md`.
4. **Obsidian as viewer** — wiki directory (or zip) opens as a vault; graph/Dataview live *outside* the compiler.
5. **Human curates, LLM maintains** — overwrite policy, review queues, lint — productized in full apps; MVP keeps a simple `generated: true` guard.

**Jarvis stance:** wiki is an **optional compile layer beside RAG**, not a replacement. Hybrid retrieval + citations to original chunks stay authoritative; wiki pages are additional `SourceKind::WikiPage` documents in the same index.

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete *for a “wiki compile” feature*.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Opt-in wiki (`WikiConfig.enabled`, default **off**) | Existing users must see zero behavior change | LOW | serde `#[serde(default)]`; commands error/no-op when disabled |
| LLM → structured analysis (summary + entities + concepts) | “Compile” means synthesis, not copy-paste chunks | MEDIUM | JSON-only prompt; MockChatModel in unit/E2E; reuse insights truncate (~12k) |
| Markdown pages with YAML frontmatter + `sources: []` | Traceability is table stakes in llm_wiki / Obsidian PKM | LOW–MEDIUM | Frontmatter: `title`, `type`, `sources`, `content_hash`, `generated: true` |
| `[[wikilink]]` between summary and entity/concept pages | Interlinked wiki is the product metaphor | LOW | Deterministic renderer after LLM JSON |
| `index.md` catalog | Users expect a vault entry point | LOW | Merge/update on each compile |
| Persist under `{app_data}/wiki/` | Local-first; readable on disk without Jarvis | LOW | Same app-data pattern as skills/hooks |
| Index wiki pages via existing ingest→index pipeline | Wiki must participate in Library + RAG like Memory | MEDIUM | `SourceKind::WikiPage`, `uri: wiki://{slug}`, hash skip |
| Idempotent re-compile (`content_hash` / stable URIs) | Rebuildable layer; no duplicate sources | MEDIUM | Indexer already skips unchanged hashes |
| Library “生成笔记” + Settings toggle | Discoverable UX for opt-in feature | MEDIUM | `data-testid`s; hide controls when disabled |
| Obsidian-compatible zip export | Escape hatch / competitor parity without shipping a graph UI | MEDIUM | Zip tree + minimal `.obsidian/app.json`; export-only |
| Preserve RAG + citations | Core value; wiki must not steal answer authority | LOW (policy) | E2E: qa/agent journeys unchanged; wiki is additive sources |
| Deterministic E2E journey | User-facing = required E2E | MEDIUM | `wiki.spec.ts` + Mock wiki JSON sequence in `e2e.rs` |

### Differentiators (Competitive Advantage)

Features that set Jarvis apart. Not required for a standalone wiki app, but valuable *here*.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Wiki beside RAG, not instead of it** | Citations stay grounded in original chunks; wiki is a readable synthesis layer users can also retrieve | LOW–MEDIUM | Policy + retrieval ranking later; v1.10: don’t prefer wiki over originals in prompts |
| Reuse **insights + indexer + `SourceKind`** (Memory precedent) | One pipeline; no second DB or LanceDB | LOW | Extend `crates/insights` (`wiki.rs`); thin Tauri commands |
| Multi-source corpus already indexed (files, Lark, Cursor, memory) | Compile notes from Feishu/transcripts without a new ingest product | LOW (leverage) | Compile gated on `Indexed` sources only |
| Local-first + swappable LLM/embedder + CI mocks | Trust + shippable quality vs cloud-only wiki apps | MEDIUM | Aligns with Core Value |
| `generated: true` overwrite only | Safe coexistence with light human edits without full review queue | LOW | Explicit anti-scope for smart merge |
| Agent/RAG can retrieve wiki pages later | Compiled entities become searchable knowledge without graph UI | LOW after index | No new agent tools required for MVP |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems — especially when bolted onto a mature RAG app.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Replace RAG with wiki-only Q&A | “Compile once” narrative from llm_wiki | Breaks citation trust; regressions on chunk-level answers; fights Core Value | Keep RAG primary; wiki is optional additive sources |
| Knowledge graph UI / Louvain / “graph insights” | Obsidian / llm_wiki showcase | High FE+algo cost; weak MVP signal; out of plan | Export vault → open in Obsidian |
| LanceDB / second vector store | Semantic search over wiki tree | Violates single-DB-owner; duplicates sqlite-vec | Index `WikiPage` into existing store |
| Bidirectional Obsidian sync | “My vault is source of truth” | Conflict resolution, watchers, delete semantics | One-way zip/folder export |
| Chrome clipper / Deep Research / web ingest | Grow corpus from web | New products; auth, cost, flaky CI | Use existing ingest + later skills |
| Auto-compile all sources on by default | “Magic wiki builds itself” | LLM cost, noise, surprise writes | Default `enabled=false`; `auto_on_insights` default false |
| Full `schema.md` / `purpose.md` / lint/review queues | Faithful Karpathy/llm_wiki product | Large surface; slow v1.10 | Hard-coded render contract + unit tests |
| Cross-corpus entity merge (one “Acme” page from 50 docs) | Reflect/Memo entity pages | Dedup, merge, conflict; hard to test | **Per-source compile** first; shared entities later |
| Live LLM in unit/E2E | “Real quality” | Flaky CI, cost | MockChatModel + fixed JSON fixtures |
| Editing wiki inside Jarvis as a second notes app | Compete with Obsidian | Scope explosion vs Library+export | Disk Markdown + Obsidian; Jarvis compiles |

## Feature Dependencies

```
WikiConfig.enabled
    └──requires──> AppConfig nested serde (v1.9)
    └──gates──> Library compile UI / export UI / IPC

SourceKind::WikiPage
    └──requires──> store types + schema kind string
    └──requires──> sourceDisplay label
    └──enhances──> Library list + RAG retrieval

analyze_source_for_wiki (LLM JSON)
    └──requires──> ChatModel + indexed source chunks (Store)
    └──requires──> insights truncate / InsightsError::Parse
    └──feeds──> render_wiki_pages (pure)

render_wiki_pages
    └──feeds──> write wiki/*.md + index.md
    └──feeds──> index_document (WikiPage, wiki:// URI)

compile_wiki_for_source
    └──requires──> analyze + render + Embedder + indexer hash skip
    └──enhances──> Library “生成笔记”

export_wiki_zip
    └──requires──> wiki_root on disk
    └──enhances──> Obsidian escape hatch

E2E wiki.spec
    └──requires──> toggle + compile + list + export + Mock sequence
    └──conflicts──> (soft) shared MockChatModel queues if not isolated

Wiki compile ──conflicts──> “RAG replacement” product framing
Wiki export ──conflicts──> bidirectional Obsidian sync (same milestone)
```

### Dependency Notes

- **Compile requires indexed sources:** analysis reads chunks from Store; skip if not `Indexed`.
- **WikiPage requires indexer + store kind:** same path as Memory (`normalize → Document → index_document`).
- **UI requires config flag:** default-off means full-ui must assert controls **hidden** until enabled.
- **Export requires files on disk:** can ship after compile path; stub IPC early if sequenced.
- **E2E requires dedicated mock replies:** wiki JSON must not collide with Q&A/agent mock sequences.
- **RAG primary conflicts with wiki-as-answer-layer:** do not change citation preference in v1.10.

## MVP Definition

### Launch With (v1.10)

Minimum viable product — ruthless for validating “optional wiki beside RAG.”

- [ ] `WikiConfig { enabled: false, auto_on_insights: false }` + Settings toggle — zero regression when off
- [ ] `SourceKind::WikiPage` + Library label — first-class sources
- [ ] `analyze` → `render` → write `{app_data}/wiki/` → index with `wiki://` + `content_hash` — core compile loop
- [ ] Frontmatter + `[[wikilinks]]` + `index.md` + `generated: true` overwrite rule — vault semantics users expect
- [ ] Library “生成笔记” (per indexed source) when enabled — one explicit user action
- [ ] Obsidian zip export (tree + minimal `.obsidian/`) — differentiator without graph UI
- [ ] E2E: enable → compile → see wiki page → export; qa/agent still green — ship gate

### Add After Validation (v1.x / v1.11+)

- [ ] `auto_on_insights` wired after summarize — only after cost/noise validated
- [ ] Related-docs / source-overlap panel — plan’s v1.11
- [ ] Read-only MCP `search` / `list_sources` — plan’s v1.11
- [ ] Cross-source entity merge / upsert by canonical name — after per-source MVP
- [ ] Agent tools that prefer or cite wiki entities — once retrieval quality known
- [ ] Bulk “compile all indexed” with progress events — reuse `IndexProgressEvent` patterns

### Future Consideration (v2+ / later)

- [ ] Graph UI / Louvain / graph insights — export to Obsidian instead for now
- [ ] Bidirectional Obsidian sync — conflict hell
- [ ] Chrome clipper, Deep Research, LanceDB — competitor parity, not Core Value
- [ ] Full schema/purpose/lint/review product surface — llm_wiki-scale app
- [ ] In-app Markdown editor for wiki pages — stay compiler + Library

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| WikiConfig default off + Settings toggle | HIGH | LOW | P1 |
| SourceKind::WikiPage + display label | HIGH | LOW | P1 |
| Deterministic Markdown renderer (frontmatter, links, index) | HIGH | LOW | P1 |
| LLM analyze → WikiAnalysis (Mock-tested) | HIGH | MEDIUM | P1 |
| Write wiki/ + index via existing pipeline | HIGH | MEDIUM | P1 |
| Library compile control + busy/error | HIGH | MEDIUM | P1 |
| Obsidian zip export | HIGH | MEDIUM | P1 |
| E2E wiki journey + mock isolation | HIGH | MEDIUM | P1 |
| Idempotent re-compile (hash / URI upsert) | HIGH | MEDIUM | P1 |
| Preserve RAG citations (regression) | HIGH | LOW | P1 |
| `auto_on_insights` | MEDIUM | LOW | P2 |
| Bulk compile + progress | MEDIUM | MEDIUM | P2 |
| Cross-source entity merge | MEDIUM | HIGH | P3 |
| Related-docs / MCP | MEDIUM | MEDIUM | P3 |
| Graph UI / LanceDB / clipper / Deep Research / bi-sync | LOW (for Core Value) | HIGH | P3 (defer / anti) |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | nashsu/llm_wiki | Obsidian / Memo–Reflect | Jarvis v1.10 approach |
|---------|-----------------|-------------------------|------------------------|
| Compile sources → Markdown wiki | Core product; often *instead of* RAG | Human-written notes; AI plugins optional | Optional compile **beside** existing RAG |
| Entity / concept pages | Yes, agent-maintained | Entity pages (Memo/Reflect) / wikilinks | Per-source entity/concept + source summary |
| YAML + `[[wikilinks]]` + index | Yes | Obsidian native | Yes (table stakes) |
| Source traceability | `sources: []` in frontmatter | Manual / plugins | Same contract |
| Vector / hybrid search | Optional LanceDB + chat over wiki | Plugins / separate | **Existing** sqlite-vec + FTS; index WikiPage |
| Knowledge graph UI | First-class Louvain graph | Obsidian graph | **Out of scope** — export vault |
| Web clipper / Deep Research | Shipped | Clipper ecosystem | **Out of scope** |
| Obsidian interop | Vault-compatible dir | Native | Zip + minimal `.obsidian/` (export-only) |
| Citations in Q&A | Wiki-centric chat | N/A | **RAG citations to originals remain primary** |
| Default on | Product is the wiki | User vault | **Default off** |

## Sources

- Implementation plan: `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md`
- Project brief: `.planning/PROJECT.md` (v1.10 Wiki Compile Layer)
- Competitor: [nashsu/llm_wiki](https://github.com/nashsu/llm_wiki) (Karpathy three-layer pattern, Obsidian vault, graph/Lance/clipper as expansions)
- Pattern origin: Karpathy LLM Wiki gist (raw → wiki → schema; ingest/query/lint)
- Inspiration: Obsidian vaults; Memo/Reflect-style entity pages
- Existing Jarvis: `crates/insights` summarize/tasks; `SourceKind::Memory`; indexer hash skip; Library/Settings; `JARVIS_E2E=1`

---
*Feature research for: Jarvis Wiki Compile Layer (v1.10)*
*Researched: 2026-07-17*
