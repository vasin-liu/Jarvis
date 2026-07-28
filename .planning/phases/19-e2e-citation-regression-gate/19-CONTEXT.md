# Phase 19: E2E + citation regression gate - Context

**Gathered:** 2026-07-28
**Status:** Ready for planning
**Mode:** --auto (recommended defaults from v1.11 research + ROADMAP TRUST-01/02)

<domain>
## Phase Boundary

Ship gate for v1.11: prove **related-docs** and **MCP** did not regress **RAG citation trust** or user-facing coverage. Re-run citation E2E under `JARVIS_E2E=1` mocks; keep focused related-docs journey green; verify MCP offline via cargo; confirm e2e-required map + MCP docs remain accurate.

**This phase is primarily a verification / hardening gate** — not a new product feature. Prefer surgical E2E/docs fixes over product behavior changes.

**Out of scope:** New MCP tools; MCP WebDriver; Settings/MCP-F02; graph UI; WikiPage soft-exclude (REL-F01); changing default RAG/citation fusion “for quality”; live LLM in CI.

</domain>

<decisions>
## Implementation Decisions

### Trust freeze (TRUST-01)
- **D-01:** Do **not** change default `rag::ask` / citation fusion / `RetrieverConfig::default` / related scoring knobs in this phase **unless** a failing gate test proves a regression — then fix surgically with a regression test, no opportunistic refactors.
- **D-02:** Citation gate suite (must stay green under `JARVIS_E2E=1`): **`qa.spec.ts`**, **`full-ui.spec.ts`** (citation/trust paths), and **`wiki.spec.ts`** citation trust journey. Treat failures as ship blockers.

### Related-docs coverage (TRUST-02)
- **D-03:** Keep and require green **`e2e/specs/related-docs.spec.ts`** (panel visibility + navigate) with existing dual-fixture seed; no live LLM.
- **D-04:** Extend **`full-ui.spec.ts`** with a **short** related-docs step (select indexed source → panel → optional navigate) so the primary journey covers Library related discovery — matches e2e-required “extend full-ui if part of primary journey.” Do not duplicate the entire focused spec.

### MCP offline happy path (TRUST-02)
- **D-05:** MCP verification remains **`cargo test -p mcp`** (plus `retriever`/`agent` if shared helpers touched). **No WebDriver / stdio-host E2E** for MCP in v1.11.
- **D-06:** Confirm `docs/mcp.md` + README MCP pointer still match Phase 18 live tools (no stub wording drift). Docs-only edits allowed; no Settings UI.

### Spec map & harness
- **D-07:** Confirm `.cursor/rules/e2e-required.mdc` Related-docs row stays accurate; update only if paths/names changed.
- **D-08:** Ship gate commands: `npm run test:e2e:local` covering related-docs + citation specs (or equivalent focused wdio runs after `build:e2e`), plus cargo MCP suite. Document in SUMMARY/VERIFICATION what ran.

### UI / product
- **D-09:** **No new Library UI chrome** this phase (UI hint on ROADMAP means E2E/UX verification of existing panel, not redesign). Vitest updates only if a gate fix requires them.

### Claude's Discretion
- Exact full-ui related-docs assertion depth (minimum: panel displayed after select)
- Whether to run `npm run test:e2e:local` (all specs) vs explicit `--spec` list for faster iteration — final VERIFICATION must still prove D-02/D-03 suites green
- Windows path / `data-testid` click helpers: reuse Phase 16 patterns; fix flakiness surgically

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase / milestone
- `.planning/ROADMAP.md` — Phase 19 goal, success criteria, TRUST-01/02
- `.planning/REQUIREMENTS.md` — TRUST-01, TRUST-02
- `.planning/research/SUMMARY.md` — Phase 19 deliverables; joint ship gate
- `.planning/research/PITFALLS.md` — Pitfall 5 citation regression; looks-done checklist; qa/full-ui mandatory
- `.planning/research/STACK.md` — WebDriver panel vs cargo MCP
- `.planning/phases/16-related-docs-library-panel/16-CONTEXT.md` — related-docs E2E / testids
- `.planning/phases/16-related-docs-library-panel/16-VERIFICATION.md` — prior E2E evidence
- `.planning/phases/18-mcp-tools-search-list-sources/18-CONTEXT.md` — MCP cargo-only testing lock

### Code / harness
- `e2e/specs/related-docs.spec.ts` — focused panel journey
- `e2e/specs/qa.spec.ts` — citation excerpt regression
- `e2e/specs/full-ui.spec.ts` — primary journey (+ wiki citation paths)
- `e2e/specs/wiki.spec.ts` — citation URI trust
- `e2e/wdio.conf.ts` / `package.json` `test:e2e:local`
- `.cursor/rules/e2e-required.mdc` — spec map
- `docs/mcp.md`, `README.md` — MCP host docs
- `src-tauri/src/e2e.rs` — fixtures / `JARVIS_E2E=1`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `related-docs.spec.ts` already green from Phase 16 (Windows path-safe click helpers)
- Dual E2E fixture seed (`sample.md` + `related-neighbor.md`) in `e2e.rs`
- `qa` / `full-ui` / `wiki` citation journeys from v1.10 Phase 13
- `cargo test -p mcp` tools_kb + allowlist from Phase 18
- e2e-required map already has Related-docs row

### Established Patterns
- `JARVIS_E2E=1` + Mock providers; never live LLM in CI
- Focused specs + optional thin full-ui extension
- MCP stays cargo-level (stdio); panel uses WebDriver

### Integration Points
- full-ui Library section → add related-docs assertions
- VERIFICATION.md must record gate command results
- Docs drift check after Phase 18 wording

</code_context>

<specifics>
## Specific Ideas

[--auto] Selected all gray areas: Trust freeze, Citation suite scope, Related-docs E2E, full-ui extension, MCP cargo-only, Spec map/docs, Ship gate commands, No new UI.

[auto] Trust freeze — Q: "Change RAG defaults?" → Selected: "No unless gate proves regression; surgical fix only" (recommended)
[auto] Citation suite — Q: "Which specs?" → Selected: "qa + full-ui + wiki citation trust" (recommended; Pitfall 5)
[auto] Related-docs — Q: "Coverage?" → Selected: "Keep related-docs.spec green" (recommended)
[auto] full-ui — Q: "Extend primary journey?" → Selected: "Yes — short related-docs step" (recommended; e2e-required)
[auto] MCP — Q: "How to verify?" → Selected: "cargo test -p mcp only; no WebDriver MCP" (recommended)
[auto] Docs/map — Q: "What to update?" → Selected: "Confirm e2e-required + docs/mcp.md + README accuracy" (recommended)
[auto] UI redesign — Q: "New panel chrome?" → Selected: "No — verification only" (recommended)

</specifics>

<deferred>
## Deferred Ideas

- MCP stdio WebDriver / Cursor live-client smoke
- REL-F01 WikiPage soft-exclude in related-docs
- MCP-F01/F02/F03
- Hard WikiPage RAG citation filter
- Cross-corpus entity merge

</deferred>

---

*Phase: 19-e2e-citation-regression-gate*
*Context gathered: 2026-07-28*
