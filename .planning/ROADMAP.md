# Roadmap: Jarvis

**Updated:** 2026-07-29

## Milestones

- ✅ **v1.9 Structural Refactor** — Phases 01–06 (shipped 2026-07-17) — [archive](./milestones/v1.9-ROADMAP.md)
- ✅ **v1.10 Wiki Compile Layer** — Phases 07–14 (shipped 2026-07-25) — [archive](./milestones/v1.10-ROADMAP.md)
- ✅ **v1.11 Related-docs + MCP** — Phases 15–19 (shipped 2026-07-29) — [archive](./milestones/v1.11-ROADMAP.md)

## Phases

<details>
<summary>✅ v1.9 Structural Refactor (Phases 01–06) — SHIPPED 2026-07-17</summary>

- [x] Phase 01–06 — 22 plans — completed 2026-07-17
- Details: [v1.9-ROADMAP.md](./milestones/v1.9-ROADMAP.md) · phases: [v1.9-phases/](./milestones/v1.9-phases/)

</details>

<details>
<summary>✅ v1.10 Wiki Compile Layer (Phases 07–14) — SHIPPED 2026-07-25</summary>

- [x] Phase 07: Wiki kind + config (3/3) — completed 2026-07-18
- [x] Phase 08: Deterministic Markdown renderer (2/2) — completed 2026-07-19
- [x] Phase 09: LLM wiki analysis (2/2) — completed 2026-07-19
- [x] Phase 10: Persist + index (2/2) — completed 2026-07-21
- [x] Phase 11: Library / Settings UI (2/2) — completed 2026-07-23
- [x] Phase 12: Obsidian zip export (2/2) — completed 2026-07-23
- [x] Phase 13: E2E + citation trust (2/2) — completed 2026-07-24
- [x] Phase 14: Tech debt closeout (3/3) — completed 2026-07-25

Details: [v1.10-ROADMAP.md](./milestones/v1.10-ROADMAP.md) · phases: [v1.10-phases/](./milestones/v1.10-phases/) · audit: [v1.10-MILESTONE-AUDIT.md](./milestones/v1.10-MILESTONE-AUDIT.md)

</details>

<details>
<summary>✅ v1.11 Related-docs + MCP (Phases 15–19) — SHIPPED 2026-07-29</summary>

- [x] Phase 15: Overlap scoring API (2/2) — completed 2026-07-25
- [x] Phase 16: Related-docs Library panel (4/4) — completed 2026-07-28
- [x] Phase 17: MCP transport + read-only scaffold (4/4) — completed 2026-07-28
- [x] Phase 18: MCP tools search + list_sources (4/4) — completed 2026-07-28
- [x] Phase 19: E2E + citation regression gate (3/3) — completed 2026-07-28

Details: [v1.11-ROADMAP.md](./milestones/v1.11-ROADMAP.md) · audit: [v1.11-MILESTONE-AUDIT.md](./milestones/v1.11-MILESTONE-AUDIT.md)

</details>

## Progress

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 01–06 | v1.9 | 22/22 | Complete | 2026-07-17 |
| 07–14 | v1.10 | 18/18 | Complete | 2026-07-25 |
| 15. Overlap scoring API | v1.11 | 2/2 | Complete | 2026-07-25 |
| 16. Related-docs Library panel | v1.11 | 4/4 | Complete | 2026-07-28 |
| 17. MCP transport + read-only scaffold | v1.11 | 4/4 | Complete | 2026-07-28 |
| 18. MCP tools search + list_sources | v1.11 | 4/4 | Complete | 2026-07-28 |
| 19. E2E + citation regression gate | v1.11 | 3/3 | Complete | 2026-07-28 |

## Backlog (post-v1.11 / carryover)

- Bulk compile + `auto_on_insights` UX (WIKI-F01) — deferred OK through v1.10 audit (D-14)
- Cross-corpus entity merge
- Dual `index.md` writers consolidation (Phase 09 test helper vs compile rebuild) — intentional; defer (D-14)
- Ship DeferredEmbedder / deferred scan startup fix in a release build
- REL-F01: Soft-exclude/demote WikiPage neighbors in related-docs
- REL-F02: Open related source scoped into chat / ask
- MCP-F01: `get_chunk` / `read` document tools
- MCP-F02: Settings toggle to enable/disable MCP server
- MCP-F03: HTTP/SSE local transport
- REL-02 affinity score threshold (D-06 deferred) — prefer empty over weak neighbors

---

*Next:* `/gsd-new-milestone`
