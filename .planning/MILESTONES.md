# Project Milestones: Jarvis

## v1.10 Wiki Compile Layer (Shipped: 2026-07-25)

**Delivered:** Optional, rebuildable Markdown wiki layer beside RAG — `WikiConfig` (default off), LLM analyze → deterministic render → `wiki://` index, Library/Settings compile + Obsidian zip export, E2E journey with citation trust — plus pre-ship tech-debt closeout (soft-skip reindex, preflight gate, Nyquist backfill).

**Phases completed:** 07–14 (8 phases, 18 plans, 41 tasks)

**Key accomplishments:**
- Nested `WikiConfig` default-off + `SourceKind::WikiPage` with Library label「笔记页」
- Pure `render_wiki_pages` + LLM `analyze_source_for_wiki` with fail-closed zero-file parse
- `compile_wiki_for_source` → `{app_data}/wiki/` + `wiki://{slug}` hash-skip index
- Settings/Library gated「生成笔记」+ path-safe Obsidian `export_wiki_zip`
- `wiki.spec.ts` enable→compile→笔记页→export + non-`wiki://` citation trust
- Phase 14 closeout: WikiPage soft-skip, preflight WikiDisabled gate, Nyquist 12/13

**Stats:**
- 8 phases, 18 plans, 41 tasks
- Requirements: 9/9 Complete (WIKI-01..WIKI-09)
- Audit: `v1.10-MILESTONE-AUDIT.md` — **passed** (re-audit after Phase 14)
- Closeout: verified_closeout

**Deferred (accepted):** WIKI-F01 `auto_on_insights` UX; dual `index.md` writers; WIKI-08 ranking-dependent citation trust (no hard RAG filter)

**What's next:** Define next milestone via `/gsd-new-milestone` (e.g. related-docs / MCP, release packaging)

---

## v1.9 Structural Refactor (Shipped: 2026-07-17)

**Delivered:** Incremental structural refactor — modular React views/hooks, thin Tauri shell, nested config + OS keychain secrets, `memory://` URIs, JSON-first agent tool protocol with parse warnings, Settings extraction and architecture sign-off — without breaking local RAG / Agent user behavior.

**Phases completed:** 1-6 (22 plans total)

**Key accomplishments:**
- Extracted Chat / Library / Tasks / Memory / Settings views with dedicated hooks; slimmed `App.tsx`
- Split Tauri `AppState` + `commands/*`; registration-only `lib.rs` path
- Migrated `cloud_api_key` to OS keychain; nested `AppConfig` with backward-compatible serde
- Hardened memory identity with `memory://{uuid}` + migration
- Replaced fragile agent XML tool parsing with structured JSON + UI parse warnings
- Architecture review + sync error surfacing in Settings; Phase 05 human UAT passed

**Stats:**
- 6 phases, 22 plans, 22 SUMMARY.md artifacts
- Requirements: 22/22 Complete
- Closeout: verified_closeout (all phases verification=passed)
- Known override: no formal `v1.9-MILESTONE-AUDIT.md` at close

**What's next:** v1.10 Wiki Compile Layer (optional compile notes + Obsidian export) — see `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md`

---
