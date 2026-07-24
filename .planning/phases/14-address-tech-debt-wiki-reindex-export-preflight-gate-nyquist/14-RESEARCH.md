# Phase 14: Tech debt closeout - Research

**Researched:** 2026-07-24
**Domain:** Brownfield tech-debt closeout (WikiPage reindex soft-skip, export preflight `wiki.enabled` gate, Nyquist doc flip, milestone re-audit)
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### WikiPage「重新同步」
- **D-01:** `SourceKind::WikiPage` reindex path is a **soft skip** — do **not** call `mark_failed`; return success-skip semantics consistent with “nothing to do”.
- **D-02:** Soft skip is **silent** — no toast, no error bar.
- **D-03:** In `LibraryView`, **hide** the「重新同步」/「重试」button for `wiki_page` rows (`retry-source-{id}`); backend soft-skip remains as IPC safety net.
- **D-04:** Tests: **Rust unit** (soft skip does not set Failed) + **Vitest** (button hidden for wiki_page). **No new E2E** for this behavior.
- **D-05:** **Bulk** rebuild/retry that encounters `WikiPage` uses the **same soft skip** — no Failed status, do not abort the batch.

#### Export preflight when wiki disabled
- **D-06:** When `wiki.enabled` is false, `wiki_export_preflight_cmd` returns a **hard error** with **`InsightsError::WikiDisabled`** (parity with `export_wiki_zip`).
- **D-07:** Prefer **crate reuse** — gate via enabled flag and map the same `WikiDisabled` error string (`wiki is disabled`) as export.
- **D-08:** FE keeps existing `catch` → `reportError(error)`; **no** special Chinese mapping for this string.
- **D-09:** Tests: **Rust unit only** for disabled → WikiDisabled; **no new Vitest** required.

#### Nyquist phases 12 / 13
- **D-10:** Set `nyquist_compliant: true` on `12-VALIDATION.md` and `13-VALIDATION.md` based on existing VERIFICATION + green tests; **do not** require a full `/gsd-validate-phase` re-run.
- **D-11:** **Only** flip `nyquist_compliant`; leave `status` and `wave_0_complete` unchanged (Phase 11 pattern).
- **D-12:** Add a **short body note** in each VALIDATION.md citing VERIFICATION + green tests as evidence for the nyquist backfill.

#### Audit / milestone closeout bar
- **D-13:** After Phase 14 is verified, **must** re-run `/gsd-audit-milestone` expecting **`status: passed`**, then `/gsd-complete-milestone v1.10`.
- **D-14:** Audit may still list **WIKI-F01** and **dual `index.md` writers** as Deferred / Out of Scope backlog; clearing the four Phase-14 items is enough for `passed`.
- **D-15:** Phase 14 verify gate = relevant **cargo test** + **Vitest** green + VALIDATION/ROADMAP docs updated; **no new E2E**.

### Claude's Discretion
- Exact placement of soft-skip helper vs inline match arm in `index_ops.rs`.
- Whether preflight gate lives only in Tauri cmd (reading `cfg.wiki.enabled`) or a thin crate helper used by both preflight and export — as long as D-06/D-07 hold.
- Exact wording of the VALIDATION.md evidence note (D-12).
- How ROADMAP Phase 14 plan checkboxes are filled after planning (not locked here).

### Deferred Ideas (OUT OF SCOPE)
- **WIKI-F01** `auto_on_insights` UX — post-v1.10 backlog
- **Dual `index.md` writers** consolidation (Phase 09 test helper vs compile `rebuild_index_md_from_disk`) — intentional; leave deferred
- True disk re-index of wiki Markdown / re-compile from parent source — out of Phase 14 scope
- Chinese mapping for `wiki is disabled` in FE — rejected for this phase (D-08)
- New E2E for retry-button absence — rejected (D-04, D-15)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| Audit closeout (no new WIKI-IDs) | Clear four Phase-14 tech-debt items so milestone audit can reach `passed` | Soft-skip before `delete_chunks`; preflight `WikiDisabled` gate; Nyquist frontmatter flip + evidence note; ROADMAP Coverage already Complete |
| WIKI-F01 / dual `index.md` writers | Stay deferred (D-14) | Do not plan tasks; audit may still list as backlog |
</phase_requirements>

## Summary

Phase 14 is **documentation + surgical brownfield fixes**, not a new product surface. Four audit items block `passed`: (1) WikiPage reindex fail-closed stub in `reindex_source`, (2) ungated `wiki_export_preflight_cmd`, (3) `nyquist_compliant: false` on phases 12/13 VALIDATION.md, (4) ROADMAP WIKI-08/09 coverage (already synced to Complete in tree). WIKI-F01 and dual `index.md` writers remain intentional backlog.

**Critical implementation hazard:** `reindex_source` calls `delete_chunks_for_source` **before** the `SourceKind` match. A naive soft-skip that only replaces the WikiPage arm with `Ok(false)` (no `mark_failed`) still **wipes chunks/vectors**. Soft skip must **early-return before** chunk deletion. [VERIFIED: codebase]

**Primary recommendation:** Early-return `Ok(false)` for `WikiPage` before `delete_chunks`; hide `retry-source-*` for `wiki_page` mirroring compile hide; add crate-level enabled gate (or shared check) so preflight returns `InsightsError::WikiDisabled` / `"wiki is disabled"`; flip only `nyquist_compliant` on 12/13 VALIDATION with a short evidence note; verify gate = cargo + Vitest (no new E2E); then re-audit to `passed`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| WikiPage reindex soft skip | API / Backend (`src-tauri` index_ops) | Database (`store` upsert/status) | Reindex semantics live in `reindex_source`; soft skip must not mutate Failed or delete chunks |
| Hide retry button for wiki_page | Browser / Client (`LibraryView`) | — | UX gate; backend soft-skip is IPC safety net (D-03) |
| Export preflight `wiki.enabled` gate | API / Backend (`commands/wiki` + `insights`) | Browser (existing `catch` → `reportError`) | Parity with `export_wiki_zip`; FE already reports English IPC errors (D-08) |
| Nyquist 12/13 flip | Docs / planning artifacts | — | Frontmatter + short evidence note only (D-10..D-12) |
| Milestone re-audit / complete | Process (`/gsd-audit-milestone`) | — | D-13 closeout bar after Phase 14 VERIFICATION |

## Standard Stack

No new dependencies. Use existing workspace stack. [VERIFIED: codebase]

### Core

| Library / Tool | Version (workspace) | Purpose | Why Standard |
|----------------|---------------------|---------|--------------|
| Rust / cargo | stable (env: rustc 1.97.0) | Soft-skip + preflight unit tests | Project MSRV 1.85; Tauri shell owns reindex |
| `thiserror` / `InsightsError` | workspace | `WikiDisabled` Display `"wiki is disabled"` | Already used by export/compile gates |
| React + Vitest | React 19 / Vitest ^3.2.4 | Hide retry button test | Existing `LibraryView.test.tsx` matrix |
| Tauri 2 IPC | `tauri = "2"` | Thin cmds in `commands/wiki.rs` | Thin shell, fat crates |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tempfile` + `MockEmbedder` | workspace | `index_ops` unit tests | Soft-skip preserves Indexed + chunks |
| `store::{Source, SourceKind, IndexStatus}` | path crate | Seed WikiPage row for test | Sole DB owner |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Soft skip | True disk re-index of wiki MD | Explicitly deferred (CONTEXT) |
| Crate preflight helper | Gate only in Tauri cmd | Cmd-only harder to unit-test without Tauri; Discretion allows either if D-06/D-07 hold |
| New E2E | Vitest + Rust only | Locked D-04/D-15; overrides default e2e-required for this debt item |

**Installation:** none — no new packages.

**Package Legitimacy Audit:** N/A — this phase installs **zero** external packages. [VERIFIED: phase scope]

## Architecture Patterns

### System Architecture Diagram

```
Library「重新同步」/ bulk rebuild
        │
        ▼
 retry_source_by_id / rebuild_all_sources
        │
        ▼
   reindex_source(source)
        │
        ├── [NEW] if WikiPage → Ok(false)  ◄── BEFORE delete_chunks
        │         (no mark_failed; status unchanged)
        │
        ├── delete_chunks_for_source
        └── match kind → reindex / mark_failed+Ok(false)

Library「导出 Wiki」
        │
        ▼
 wiki_export_preflight_cmd
        │
        ├── [NEW] if !wiki.enabled → Err(WikiDisabled)  // "wiki is disabled"
        └── wiki_has_exportable_notes → { hasNotes }

 export_wiki_zip_cmd (already gated)
        └── export_wiki_zip(..., enabled) → WikiDisabled if false
```

### Recommended Project Structure (touchpoints only)

```
src-tauri/src/index_ops.rs          # soft skip early-return + Rust unit test
src-tauri/src/commands/wiki.rs      # preflight enabled gate
crates/insights/src/wiki_export.rs  # optional thin helper / reuse WikiDisabled
crates/insights/src/error.rs        # WikiDisabled Display (unchanged)
src/views/LibraryView.tsx           # hide retry for wiki_page
src/views/LibraryView.test.tsx      # Vitest assert queryByTestId null
.planning/phases/12-*/12-VALIDATION.md  # nyquist_compliant: true + note
.planning/phases/13-*/13-VALIDATION.md  # nyquist_compliant: true + note
```

### Pattern 1: Soft skip = early return before destructive work

**What:** For kinds that cannot/should not reindex, return `Ok(false)` **without** calling `mark_failed` and **before** `delete_chunks_for_source`.
**When to use:** WikiPage per D-01/D-05.
**Callers already treat `Ok(false)` as skip:** `rebuild_all_sources` / Lark rebuild / `retry_source_by_id` increment `skipped`, emit `outcome: "skipped"`, do **not** abort the batch. [VERIFIED: codebase]

### Pattern 2: Hide (not disable) inappropriate Library controls

**What:** Conditionally omit the button in JSX (same as「生成笔记」).
**When to use:** `retry-source-{id}` when `s.kind === "wiki_page"` (D-03).
**Existing compile gate:** `config?.wiki?.enabled === true && s.kind !== "wiki_page"`. Retry today has **no** kind gate — always rendered. [VERIFIED: codebase]

### Pattern 3: Export enabled gate via `InsightsError::WikiDisabled`

**What:** `if !wiki_enabled { return Err(InsightsError::WikiDisabled); }` then continue.
**When to use:** Preflight parity with `export_wiki_zip` (D-06/D-07).
**Display string:** `#[error("wiki is disabled")]` → IPC `to_string()` = `"wiki is disabled"`. [VERIFIED: codebase]

### Pattern 4: Nyquist backfill (Phase 11 shape)

**What:** Set `nyquist_compliant: true` only; leave `status: draft` and `wave_0_complete: false`; add short body evidence note (D-11/D-12 — Phase 11 has frontmatter flip but no evidence note; 12/13 need the note). [VERIFIED: codebase]

### Anti-Patterns to Avoid

- **Soft-skip after `delete_chunks`:** Leaves WikiPage sources Indexed-looking but chunkless / unsearchable.
- **Calling `mark_failed` then `Ok(false)`:** Current stub; creates permanent Failed + error `"wiki page reindex not implemented"`.
- **New E2E / Chinese error mapping / true reindex:** Explicitly deferred or rejected in CONTEXT.
- **Flipping `status` / `wave_0_complete` on VALIDATION:** Violates D-11.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Disabled wiki rejection | Custom error string / new enum variant | `InsightsError::WikiDisabled` | Same Display as export; FE already `reportError` |
| Empty-tree check | Reimplement MD scan | `wiki_has_exportable_notes` | Already shared by preflight |
| Soft-skip progress UX | New toast / FE branch | Existing `Ok(false)` → skipped + hide button | Silent per D-02 |
| Nyquist compliance | Full `/gsd-validate-phase` re-run | Frontmatter flip + VERIFICATION cite | Locked D-10 |
| Wiki reindex product | Disk/MD reingest pipeline | Soft skip only | Deferred |

**Key insight:** Reuse skip/outcome/`WikiDisabled`/hide patterns; do not invent new UX or error types.

## Runtime State Inventory

Behavior-fix phase (status / chunk mutation). Categories checked:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | User `kb.sqlite` may contain `wiki_page` rows with `status=Failed` and `error=wiki page reindex not implemented` from the current stub | **No migration required** for Phase 14. Soft skip stops new Failures. Optional heal (recompile / leave Failed) is out of scope — mention only if audit asks |
| Live service config | None for this debt | None — verified (local desktop config.json only; no cloud wiki service) |
| OS-registered state | None | None — verified |
| Secrets/env vars | None renamed | None — verified |
| Build artifacts | None | None — docs + source only |

## Common Pitfalls

### Pitfall 1: Soft skip after chunk deletion (CRITICAL)

**What goes wrong:** WikiPage soft-skip returns `Ok(false)` but chunks were already deleted → silent RAG hole.
**Why it happens:** `delete_chunks_for_source` is unconditional before the `match` (~L555–557).
**How to avoid:** Branch WikiPage **before** delete (early `return Ok(false)`), or restructure so destructive work is kind-gated.
**Warning signs:** Unit test that only asserts `status != Failed` but does not assert chunk count / vector presence still green.

### Pitfall 2: Treating other `Ok(false)` arms as soft-skip analogs

**What goes wrong:** Copy LocalFile/Memory/Cursor missing-file pattern.
**Why it happens:** Those arms call `mark_failed` then `Ok(false)` — fail-closed skip, not soft skip.
**How to avoid:** Soft skip = **no** `mark_failed`, preserve prior `IndexStatus` / error / chunks.

### Pitfall 3: Preflight returns Ok when disabled

**What goes wrong:** Disabled wiki still returns `{ hasNotes: true/false }`; FE may open Save then fail on export (or succeed if someone invokes preflight alone).
**Why it happens:** Today preflight never reads `cfg.wiki.enabled`. [VERIFIED: codebase]
**How to avoid:** Hard error before notes scan; assert `err.to_string() == "wiki is disabled"` or `matches!(WikiDisabled)`.

### Pitfall 4: Over-editing VALIDATION frontmatter

**What goes wrong:** Changing `status` / `wave_0_complete` confuses audit diffs.
**How to avoid:** Only `nyquist_compliant: false` → `true` + short body note (D-11/D-12).

### Pitfall 5: Re-opening deferred debt in plan

**What goes wrong:** Tasks for WIKI-F01 or dual index writers block closeout.
**How to avoid:** D-14 — audit `passed` with those remaining as Deferred.

### Pitfall 6: Assuming ROADMAP Coverage still Pending

**What goes wrong:** Duplicate sync work.
**Why:** Audit text (2026-07-24) said Pending; current `ROADMAP.md` Coverage already lists WIKI-08/09 **Complete**. [VERIFIED: codebase] Confirm in verify; no rewrite needed unless drift returns.

## Code Examples

### Current WikiPage stub (replace)

```653:656:src-tauri/src/index_ops.rs
        SourceKind::WikiPage => {
            mark_failed(store, source, "wiki page reindex not implemented")?;
            Ok(false)
        }
```

### Prescribed soft skip (early return — illustratively)

```rust
// Source: codebase pattern — MUST run before delete_chunks_for_source
async fn reindex_source(...) -> Result<bool, String> {
    if source.kind == SourceKind::WikiPage {
        return Ok(false); // soft skip: no mark_failed, no chunk wipe
    }
    // existing memory_text / delete_chunks / match arms...
}
```

[ASSUMED] exact helper vs inline `if` — Discretion; both OK if early-before-delete holds.

### Existing soft-skip consumer (bulk does not abort)

```125:135:src-tauri/src/index_ops.rs
            Ok(false) => {
                skipped += 1;
                on_progress(IndexProgressEvent {
                    phase: phase.to_string(),
                    current,
                    total,
                    source_title: source.title.clone(),
                    outcome: Some("skipped".into()),
                    message: source.error.clone(),
                });
            }
```

### WikiDisabled Display + export gate

```13:14:crates/insights/src/error.rs
    #[error("wiki is disabled")]
    WikiDisabled,
```

```37:40:crates/insights/src/wiki_export.rs
pub fn export_wiki_zip(wiki_root: &Path, dest_zip: &Path, wiki_enabled: bool) -> Result<()> {
    if !wiki_enabled {
        return Err(InsightsError::WikiDisabled);
```

### Current preflight (ungated) vs export cmd (gated)

```44:50:src-tauri/src/commands/wiki.rs
pub async fn wiki_export_preflight_cmd(
    state: State<'_, AppState>,
) -> Result<WikiExportPreflight, String> {
    let wiki_root = wiki_root_from_state(&state)?;
    Ok(WikiExportPreflight {
        has_notes: wiki_has_exportable_notes(&wiki_root),
    })
}
```

```54:65:src-tauri/src/commands/wiki.rs
pub async fn export_wiki_zip_cmd(
    dest_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let cfg = state.config();
    let wiki_root = wiki_root_from_state(&state)?;
    export_wiki_zip(
        &wiki_root,
        std::path::Path::new(&dest_path),
        cfg.wiki.enabled,
    )
    .map_err(|e| e.to_string())
}
```

**Recommended Discretion choice for testability (D-09):** add a thin crate helper, e.g. `wiki_export_preflight(wiki_root, enabled) -> Result<bool>` that gates then calls `wiki_has_exportable_notes`, unit-test disabled → `WikiDisabled` beside `export_wiki_rejects_when_disabled`. Tauri cmd stays thin. [ASSUMED] name/signature — planner picks; behavior locked.

### Hide compile pattern → mirror for retry

```238:259:src/views/LibraryView.tsx
                    {config?.wiki?.enabled === true && s.kind !== "wiki_page" && (
                      <button
                        ...
                        data-testid={`wiki-compile-${s.id}`}
                        ...
                      >
                        生成笔记
                      </button>
                    )}
                  </>
                )}
                <button
                  type="button"
                  className="btn-ghost text-xs"
                  disabled={busy}
                  data-testid={`retry-source-${s.id}`}
                  onClick={() => onRetrySource(s.id)}
                >
                  {s.status === "failed" ? "重试" : "重新同步"}
                </button>
```

Prescribed: wrap retry in `s.kind !== "wiki_page"` (hide entirely). Do **not** require `wiki.enabled` for hide — wiki pages should not offer reindex regardless of toggle (D-03).

### Vitest pattern to extend

```94:102:src/views/LibraryView.test.tsx
  it("hides compile for indexed wiki_page even when enabled", () => {
    ...
    expect(screen.queryByTestId("wiki-compile-wiki-1")).toBeNull();
  });
```

Add sibling: render `indexedWikiPage`, `expect(screen.queryByTestId("retry-source-wiki-1")).toBeNull()`; optionally assert local_file still has `retry-source-src-1`.

### FE error path (no change)

```154:179:src/hooks/useLibrary.ts
  const exportWiki = useCallback(
    async ({ onSuccess }: ExportWikiOptions = {}) => {
      try {
        const preflight = await wikiExportPreflight();
        ...
      } catch (error) {
        reportError(error);
      }
    },
```

Disabled preflight will hit `catch` with `"wiki is disabled"` (or Error wrapping) — acceptable per D-08. UI normally hides export when disabled, so this is IPC defense-in-depth.

### Nyquist frontmatter targets

Phase 11 (pattern — already `true`):

```yaml
status: draft
nyquist_compliant: true
wave_0_complete: false
```

Phases 12/13 today:

```yaml
status: draft
nyquist_compliant: false
wave_0_complete: false
```

After Phase 14: only flip `nyquist_compliant` to `true`; add ~2–4 line body note citing `12-VERIFICATION.md` / `13-VERIFICATION.md` + green suite evidence (D-12). Example wording (Discretion):

> **Nyquist backfill (Phase 14):** Marked compliant from existing VERIFICATION (`status: passed`) and green automated tests; full `/gsd-validate-phase` re-run not required.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| WikiPage reindex → `mark_failed` | Soft skip early-return | Phase 14 | No permanent Failed stub |
| Preflight ungated | Gate with WikiDisabled | Phase 14 | IPC parity with export |
| Nyquist false on 12/13 | Frontmatter true + note | Phase 14 | Audit nyquist overall can go full |
| ROADMAP WIKI-08/09 Pending | Complete in Coverage | Already done | Confirm only |

**Deprecated/outdated:**
- Accepting tech_debt and shipping without clearing the four Phase-14 items — superseded by Phase 14 + D-13 re-audit.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Thin crate helper `wiki_export_preflight(root, enabled)` is preferred over Tauri-only gate for D-09 unit tests | Code Examples / Discretion | Low — Tauri-only still satisfies D-06 if tested another way |
| A2 | Existing Failed wiki_page rows need no migration | Runtime State Inventory | Low — user may still see Failed until recompile; out of scope |
| A3 | Re-audit will accept `passed` with F01 + dual writers remaining | D-14 / Summary | Medium if auditor tooling treats any tech_debt YAML as fail — planner should ensure audit artifact clears Phase-14 items explicitly |

**If empty table were required:** A1–A3 are the only assumptions; all code behavior claims are codebase-verified.

## Open Questions

1. **Heal existing Failed WikiPage rows?**
   - What we know: Stub may have written Failed + error string into user DBs.
   - What's unclear: Whether closeout expects any cleanup.
   - Recommendation: Out of scope (deferred true reindex); soft skip prevents new Failures. Document in VERIFICATION if seen in manual smoke.

2. **Should progress event for soft skip include a message?**
   - What we know: Callers pass `source.error.clone()` on `Ok(false)` — usually `None` for Indexed wiki pages.
   - Recommendation: Leave as-is (silent / no FE toast). Do not add toast (D-02).

## Environment Availability

Step 2.6: minimal — code/docs only; toolchain checked on research host.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc / cargo | Soft-skip + insights unit tests | ✓ | 1.97.0 | — |
| Node / npm | Vitest | ✓ | v24.5.0 | — |
| New crates/npm pkgs | — | N/A | — | Do not add |

**Missing dependencies with no fallback:** none

## Validation Architecture

> `workflow.nyquist_validation: true` in `.planning/config.json` — section required. [VERIFIED: codebase]

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (`src-tauri` index_ops + `insights`) + Vitest ^3.2.4 |
| Config file | workspace `Cargo.toml` / `vite.config.ts` (`test.include`: `src/**/*.test.ts`) |
| Quick run command | `cargo test -p tauri-app-lib --lib index_ops -- --nocapture` (adjust binary/package name to workspace member that owns `index_ops`) && `cargo test -p insights wiki_export -- --nocapture` && `npx vitest run src/views/LibraryView.test.tsx` |
| Full suite command (Phase 14 gate) | Relevant cargo filters above green + `npx vitest run src/views/LibraryView.test.tsx` (and preflight tests); **no** `npm run test:e2e:local` (D-15) |

**Note for planner:** Confirm exact `cargo test` package name for `src-tauri` (`tauri-app` / `tauri-app-lib`) when writing `<automated>` blocks — tests live in `src-tauri/src/index_ops.rs` `mod tests`. [ASSUMED] package id — verify with `cargo metadata` / existing CI scripts when planning.

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| Soft skip (D-01/D-05) | WikiPage reindex does not set Failed; chunks preserved; bulk continues | unit (Rust) | `cargo test` filter on new `index_ops` test | ❌ Wave 0 — add beside existing `index_ops` tests |
| Hide retry (D-03/D-04) | `retry-source-{id}` absent for `wiki_page` | unit (Vitest) | `npx vitest run src/views/LibraryView.test.tsx` | ✅ file exists; ❌ case missing |
| Preflight gate (D-06/D-09) | `enabled=false` → `WikiDisabled` / `"wiki is disabled"` | unit (Rust) | `cargo test -p insights` filter on preflight/disabled | ❌ Wave 0 — add in `wiki_export.rs` tests (or cmd helper tests) |
| Nyquist 12/13 (D-10..D-12) | `nyquist_compliant: true` + evidence note | docs check | Manual/read frontmatter in verify | ✅ VALIDATION files exist |
| ROADMAP Coverage | WIKI-08/09 Complete | docs check | Diff `ROADMAP.md` Coverage | ✅ already Complete |
| E2E | — | — | **Not required** (D-04, D-15) | N/A |

### Sampling Rate

- **Per task commit:** task-scoped cargo filter **or** single Vitest file (≤30s)
- **Per wave merge:** both Rust soft-skip + preflight tests + LibraryView Vitest
- **Phase gate:** D-15 — cargo + Vitest green + VALIDATION/ROADMAP docs; then `/gsd-verify-work` → `/gsd-audit-milestone` expecting `passed`

### Wave 0 Gaps

- [ ] Rust unit: WikiPage soft skip — assert `IndexStatus` not Failed, chunks still present, `RebuildReport.skipped` increments / batch continues
- [ ] Rust unit: preflight/disabled → `InsightsError::WikiDisabled` (crate helper preferred)
- [ ] Vitest: hide `retry-source-wiki-1` for `wiki_page`
- [ ] Docs tasks: flip 12/13 `nyquist_compliant` + evidence notes (not tests)

*(Framework install: none — already present.)*

**E2E policy note for planner Dimension 8:** Project rule `.cursor/rules/e2e-required.mdc` normally requires E2E for user-facing changes; **locked CONTEXT D-04/D-15 explicitly waive new E2E** for hide-button + soft-skip. Existing `wiki.spec.ts` already covers export/compile journeys. Plans must cite the waiver so plan-checker does not demand a new spec.

## Security Domain

`security_enforcement` not set to `false` in config → include. [VERIFIED: `.planning/config.json`]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Local desktop; no new auth |
| V3 Session Management | no | — |
| V4 Access Control | yes (feature flag) | `wiki.enabled` hard gate on preflight + export; UI hide is UX-only |
| V5 Input Validation | yes (light) | Existing path-safe zip / preflight notes scan unchanged |
| V6 Cryptography | no | — |

### Known Threat Patterns for this phase

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Invoke preflight/export while wiki disabled (IPC bypass of hidden UI) | Elevation of Privilege / Tampering | Hard `WikiDisabled` on both cmds |
| Soft skip that deletes chunks | Tampering / Denial of Service (silent knowledge loss) | Early-return before `delete_chunks` |
| Error string leakage | Information Disclosure | Reuse existing English `wiki is disabled`; no new secrets |

## Project Constraints (from .cursor/rules/)

| Rule | Actionable directives for Phase 14 |
|------|-------------------------------------|
| `e2e-required.mdc` | Default: user-facing needs E2E. **Override:** CONTEXT D-04/D-15 — no new E2E; cite waiver in plans |
| `tdd-goal-driven.mdc` | Failing tests first; Rust unit + Vitest for this phase; bugfix = regression test first |
| `karpathy-guidelines.mdc` | Surgical diffs only; no speculative reindex product; surface `delete_chunks` hazard |
| `jarvis-stack.mdc` | No new deps; thin Tauri cmds; domain logic in crates (`insights` for WikiDisabled reuse) |
| `frontend-taste.mdc` | Touch LibraryView only for hide pattern; no redesign |

Additional architecture constraints from project docs: sole DB owner = `store`; map errors to `String` at Tauri boundary via `.to_string()` / `map_err`.

## Sources

### Primary (HIGH confidence)

- `src-tauri/src/index_ops.rs` — WikiPage stub, `delete_chunks` order, bulk/retry `Ok(false)` handling, `mark_failed`
- `src-tauri/src/commands/wiki.rs` — preflight ungated vs export gated
- `crates/insights/src/error.rs` / `wiki_export.rs` — `WikiDisabled`, export gate, `wiki_has_exportable_notes`
- `src/views/LibraryView.tsx` + `.test.tsx` — hide compile / always-show retry
- `src/hooks/useLibrary.ts` — preflight → catch → `reportError`
- `.planning/phases/11|12|13-*/` VALIDATION + VERIFICATION — nyquist pattern + evidence
- `.planning/v1.10-MILESTONE-AUDIT.md`, `ROADMAP.md` Phase 14, `REQUIREMENTS.md`, `14-CONTEXT.md`, `.planning/config.json`
- `.cursor/rules/*.mdc`

### Secondary (MEDIUM confidence)

- None required (brownfield; no external stack research)

### Tertiary (LOW confidence)

- Exact `cargo test -p <tauri package>` name for `index_ops` — confirm at plan time

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new packages; existing stack verified
- Architecture: HIGH — touchpoints and delete_chunks hazard verified in source
- Pitfalls: HIGH — primary failure mode is concrete line-order bug
- Nyquist/docs: HIGH — frontmatter and VERIFICATION status verified
- Cargo package id for tauri tests: LOW — confirm when writing PLAN automated commands

**Research date:** 2026-07-24
**Valid until:** 2026-08-24 (stable brownfield; re-check if `reindex_source` is refactored)

## RESEARCH COMPLETE
