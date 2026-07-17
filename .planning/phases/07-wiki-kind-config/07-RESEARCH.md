# Phase 07: Wiki kind + config — Research

**Phase:** 07 — Wiki kind + config  
**Requirements:** WIKI-01, WIKI-02  
**Research date:** 2026-07-17  
**Researcher:** gsd-phase-researcher  
**Question:** What do I need to know to PLAN this phase well?

---

## User Constraints

> Verbatim from `07-CONTEXT.md` — decisions D-01..D-15, discretion, and deferred are locked. Do not explore alternatives.

### Library label
- **D-01:** `sourceKindLabel("wiki_page")` returns exactly **「笔记页」** (Chinese, consistent with「记忆」「本地文件」).
- **D-02:** No bilingual UI label; no Wiki chip/suffix in Phase 07.
- **D-03:** Vitest asserts exact equality to `"笔记页"`.

### config.json shape
- **D-04:** Nested object — `"wiki": { "enabled": bool, "auto_on_insights": bool }` on `AppConfig` as `pub wiki: WikiConfig` with `#[serde(default)]`. **Do not** `#[serde(flatten)]` WikiConfig (would collide with ambiguous top-level field names).
- **D-05:** Missing `wiki` key → `WikiConfig::default()` (`enabled: false`, `auto_on_insights: false`).
- **D-06:** Unknown fields inside `wiki` are ignored (serde default forward-compat).
- **D-07:** Config tests cover (a) JSON without `wiki` and (b) explicit `wiki` object round-trip.

### auto_on_insights field timing
- **D-08:** Include `auto_on_insights: bool` on `WikiConfig` in Phase 07 (default `false`).
- **D-09:** Do **not** wire `auto_on_insights` to insights/compile in this milestone (Future / later phase).
- **D-10:** Phase 07 does not expose Settings UI for wiki (Settings toggle is Phase 11; auto switch not required in 07).
- **D-11:** Tests: default `auto_on_insights == false`; explicit JSON `true` round-trips.

### Unknown kind / old UI
- **D-12:** Pre-update frontend showing raw `wiki_page` string is acceptable (existing `default: return kind`).
- **D-13:** Keep `SourceKind::parse` → `None` for unknown strings; callers degrade as today. No `Unknown` variant.
- **D-14:** Adding `WikiPage` must fix all exhaustive matches so `cargo test -p store -p config` (and any crate that match on `SourceKind`) compile and pass.
- **D-15:** Keep FE `default` branch as `return kind`; only add the `wiki_page` case.

### Claude's Discretion
- Exact placement of unit tests (inline `#[cfg(test)]` vs `crates/*/tests/`) — follow local crate conventions.
- Whether `WikiConfig` lives in `types.rs` vs a small submodule — prefer existing `types.rs` pattern unless file is already oversized.

### Deferred Ideas
- Settings toggle for `wiki.enabled` — Phase 11
- Wiring `auto_on_insights` into insights_ops — Future / post-MVP
- Wiki chip / icon beside label — Phase 11+ UI polish if desired
- Compile / renderer / zip / E2E — Phases 08–13

---

## Executive Summary

Phase 07 is a **foundation-only** slice: add `SourceKind::WikiPage` ↔ `"wiki_page"`, nested `WikiConfig` (both flags default `false`, no flatten), and FE label「笔记页」. No compile pipeline, no Settings UI, no E2E wiki journey. [CITED] `.planning/ROADMAP.md` Phase 07; `07-CONTEXT.md`.

The riskiest compile break is the **exhaustive** `match source.kind` in `src-tauri/src/index_ops.rs::reindex_source` — it must gain a `WikiPage` arm or `cargo build` for the app fails. [VERIFIED] Grep of workspace `SourceKind` matches.

Nested `wiki` (non-flattened) is a deliberate break from Phase 02’s flatten pattern for Embedding/Chat/Lark/Sync/Agent — planner must not copy `#[serde(flatten)]`. [CITED] D-04; [VERIFIED] `crates/config/src/types.rs` currently flattens all five buckets.

---

## Standard Stack

[VERIFIED] From workspace / `.cursor/rules/jarvis-stack.mdc` — **no new crates for Phase 07**.

| Layer | Use |
|-------|-----|
| Rust | `stable`, MSRV 1.85, edition 2021 |
| Serde | Existing `serde` / `serde_json` on `AppConfig` / `SourceKind` |
| Store | `crates/store` — `SourceKind` + `as_str` / `parse` (DB string, not schema migration) |
| Config | `crates/config` — nested struct + `#[serde(default)]` |
| Frontend | `src/lib/sourceDisplay.ts` + Vitest |
| Tests | `cargo test -p store -p config`; `npm test` (Vitest) |

**Do not add:** `zip`, `walkdir`, insights wiki module, Tauri wiki commands, E2E wiki.spec — those are Phases 08–13. [CITED] ROADMAP + plan Task 1.

---

## Architecture Patterns

### 1. `SourceKind::WikiPage` mirrors Memory [VERIFIED]

Precedent: `SourceKind::Memory` ↔ `"memory"` via manual `as_str` / `parse` (SQLite column stores snake_case strings). [VERIFIED] `crates/store/src/types.rs`; `crates/store/src/store.rs` uses `s.kind.as_str()` on write and `SourceKind::parse(...).unwrap_or(LocalFile)` on read.

**Planner must:**
1. Add `WikiPage` variant.
2. Add `"wiki_page"` arms in `as_str` and `parse`.
3. Extend `source_kind_roundtrips` array **and** add dedicated `wiki_page_kind_roundtrips` from the implementation plan. [CITED] `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` Task 1.
4. Keep `parse("bogus") → None` (D-13).

**No schema migration:** kind is a free TEXT column; new string values are accepted as soon as parse/as_str know them. [ASSUMED] Confirmed by Memory addition pattern — no ALTER for kind enum.

**Serde note:** `SourceKind` derives `Serialize`/`Deserialize` **without** `rename_all = "snake_case"` (unlike `TaskStatus`). IPC may emit PascalCase variant names (`"Memory"`) while DB/`sourceKindLabel` use snake_case. [VERIFIED] `types.rs`. Phase 07 success criteria only require Vitest on `sourceKindLabel("wiki_page")` — do **not** expand scope to fix global SourceKind serde unless a plan task explicitly chooses `#[serde(rename_all = "snake_case")]` (would be a wire-format change for all kinds).

### 2. Nested `WikiConfig` — do NOT flatten [VERIFIED + CITED]

Current `AppConfig`:

```128:145:crates/config/src/types.rs
pub struct AppConfig {
    #[serde(flatten)]
    pub embedding: EmbeddingConfig,
    #[serde(flatten)]
    pub chat_cfg: ChatConfig,
    #[serde(flatten)]
    pub lark: LarkConfig,
    #[serde(flatten)]
    pub sync: SyncConfig,
    #[serde(flatten)]
    pub agent_cfg: AgentConfig,
    #[serde(skip)]
    pub fastembed_cache_dir: Option<std::path::PathBuf>,
}
```

**Target shape (Task 1 / D-04):**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct WikiConfig {
    pub enabled: bool,
    /// When true, compile runs after insights — field only in Phase 07; not wired (D-09).
    pub auto_on_insights: bool,
}

impl Default for WikiConfig {
    fn default() -> Self {
        Self { enabled: false, auto_on_insights: false }
    }
}

// On AppConfig — NOT flatten:
#[serde(default)]
pub wiki: WikiConfig,
```

On-disk JSON becomes mixed: flat legacy keys **plus** nested `"wiki": { ... }`. That is intentional. [CITED] D-04, plan Task 1.

Existing test `config_flat_json_roundtrip` asserts `output.get("sync").is_none()` — **update/add** tests so:
- Pre-v1.10 JSON (no `wiki`) → `wiki.enabled == false` && `wiki.auto_on_insights == false` (success criterion 1).
- Explicit `"wiki": { "enabled": true, "auto_on_insights": true }` round-trips.
- Serialize emits nested `"wiki"` object (assert `output.get("wiki").is_some()`).
- Extra key inside wiki (e.g. `"future_flag": 1`) does not fail deserialize (D-06).

Also update `AppConfig::default()` to set `wiki: WikiConfig::default()`, and re-export `WikiConfig` from `crates/config/src/lib.rs`. [VERIFIED] `lib.rs` currently exports `AppConfig` but not nested config types except via `types`.

### 3. Exhaustive Rust matches — blast radius [VERIFIED]

| Location | Match style | Phase 07 action |
|----------|-------------|-----------------|
| `crates/store/src/types.rs` `as_str` / `parse` | Exhaustive | Add `WikiPage` / `"wiki_page"` |
| `crates/store/src/types.rs` `source_kind_roundtrips` | Array of variants | Include `WikiPage` |
| **`src-tauri/src/index_ops.rs` `reindex_source`** | **Exhaustive `match source.kind`** | **Must add `SourceKind::WikiPage` arm** or app crate fails to compile |
| `src-tauri/src/index_ops.rs` ~368 | Or-pattern of Lark kinds only | No change |
| `crates/memory/*` | Equality checks `== Memory` | No change |
| `src-tauri/src/commands/lark.rs` | Lark URI → kind | No change |

**Prescriptive stub for `WikiPage` reindex arm (Phase 07):**  
Wiki pages do not exist yet and reindex-from-disk is Phase 10. Add an arm that fails closed without treating the URI as a local file:

```rust
SourceKind::WikiPage => {
    mark_failed(store, source, "wiki page reindex not implemented")?;
    Ok(false)
}
```

Do **not** fall through to `LocalFile` / `index_path`. [CITED] research PITFALLS (compile loops / wrong path); [ASSUMED] Phase 10 will replace this stub.

### 4. Frontend label only [VERIFIED]

```1:21:src/lib/sourceDisplay.ts
export function sourceKindLabel(kind: string): string {
  switch (kind) {
    // ...
    case "memory":
      return "记忆";
    default:
      return kind;
  }
}
```

Add `case "wiki_page": return "笔记页";` — keep `default: return kind` (D-15).  
Extend Vitest with `expect(sourceKindLabel("wiki_page")).toBe("笔记页")` (D-03).

`LibraryView.SourceKindIcon` has its own `default → IconFile`. Per D-02, **do not** add a wiki icon/chip this phase. [VERIFIED] `src/views/LibraryView.tsx`.

### 5. FE `AppConfig` pass-through (surgical, no Settings UI) [VERIFIED risk]

Rust `get_config` / `set_config` round-trip the full `AppConfig`. [VERIFIED] `src-tauri/src/commands/config.rs`.

TS `AppConfig` in `src/types/ipc.ts` is **flat** and has **no** `wiki` field. `useJarvisConfig.handleSaveConfig` spreads FE config into `set_config`. [VERIFIED] `src/hooks/useJarvisConfig.ts`, `src/types/config.ts` (`flatToNested` / `nestedToFlat`).

**Pitfall:** After Phase 07, if a user (or later Phase 11) sets `"wiki": { "enabled": true }` in `config.json`, the next Settings save from the UI would deserialize **without** `wiki` → serde default → **both flags forced back to false**.

**Prescribe for Phase 07 (no Settings UI, but config integrity):**
1. Add to `src/types/ipc.ts`:
   ```ts
   wiki?: { enabled: boolean; auto_on_insights: boolean };
   ```
2. Preserve `wiki` through `flatToNested` / `nestedToFlat` (or leave on flat object and spread unchanged).
3. Ensure `handleSaveConfig` / sample fixtures in `config.test.ts` do not drop `wiki`.
4. Still **no** Settings toggle UI (D-10).

If planner prefers absolute minimal FE surface, document the wipe as a **known Phase 11 blocker** — but pass-through is ~10 lines and prevents WIKI-01 footgun. Prefer pass-through. [ASSUMED] Discretion: pass-through is in scope as config plumbing, not Settings UI.

---

## Don't Hand-Roll

| Concern | Use instead |
|---------|-------------|
| Kind string mapping | Existing `as_str` / `parse` pattern on `SourceKind` — not a new DB enum table |
| Config defaults / missing key | `#[serde(default)]` on `wiki` field + `Default for WikiConfig` — not a file migration |
| Unknown fields in wiki | Default serde behavior (no `deny_unknown_fields`) — D-06 |
| Library label | One `switch` case in `sourceKindLabel` — not i18n framework |
| Feature flag gating compile | **Out of phase** — later phases read `config.wiki.enabled`; Phase 07 only stores the flag |
| Exhaustive match discovery | `rg "match .*kind\|SourceKind::"` + `cargo build` — compiler is the source of truth |

---

## Common Pitfalls

### P1: Flattening `WikiConfig` [HIGH] [CITED]
Copying Phase 02 `#[serde(flatten)]` would push `enabled` / `auto_on_insights` to top-level JSON and collide with future fields / ambiguous names. **Never flatten wiki** (D-04).

### P2: Missing `reindex_source` arm [HIGH] [VERIFIED]
Adding the enum variant without updating `index_ops.rs` → `non-exhaustive patterns` → Tauri app does not compile. Stub arm required in the same PR as the variant.

### P3: Default-on flags [HIGH] [CITED]
Both `enabled` and `auto_on_insights` must default **false**. Success criterion 1 is upgrade safety. Do not “helpfully” default `enabled: true`.

### P4: Wiring `auto_on_insights` early [MEDIUM] [CITED]
Field ships dormant (D-08/D-09). Do not touch `insights_ops.rs` this phase.

### P5: FE config wipe on Settings save [HIGH] [VERIFIED]
See Architecture §5. Pass through `wiki` on FE `AppConfig` without building Settings UI.

### P6: Changing FE `default` branch [LOW] [CITED]
D-15: keep `default: return kind`. Only add the explicit `wiki_page` case.

### P7: Scope creep into compile / E2E [MEDIUM] [CITED]
Phases 08–13 own renderer, LLM, persist, UI, zip, E2E. Phase 07 done when store+config+Vitest criteria pass.

### P8: Treating Phase 07 as user-facing E2E-required [LOW] [CITED]
No Settings/Library compile UI yet; users cannot create `WikiPage` sources via UI. E2E gate is Phase 13 (`wiki.spec.ts`). Unit/Vitest suffice for 07. [CITED] `.cursor/rules/e2e-required.mdc` — applies when feature is user-facing.

---

## Code Examples

### Store test (failing first — TDD)

```rust
#[test]
fn wiki_page_kind_roundtrips() {
    assert_eq!(SourceKind::WikiPage.as_str(), "wiki_page");
    assert_eq!(SourceKind::parse("wiki_page"), Some(SourceKind::WikiPage));
}
```

Also extend the existing loop in `source_kind_roundtrips` to include `SourceKind::WikiPage`.

### Config tests

```rust
#[test]
fn pre_v110_config_defaults_wiki_off() {
    let json = r#"{
        "watch_folders": [],
        "embedder": "mock",
        "chat": "mock",
        "mock_embed_dim": 4,
        "ollama_base_url": "http://127.0.0.1:11434",
        "ollama_embed_model": "nomic-embed-text",
        "ollama_chat_model": "llama3.2",
        "ollama_embed_dim": 768,
        "lark_cli_bin": "lark-cli"
    }"#;
    let cfg: AppConfig = serde_json::from_str(json).unwrap();
    assert!(!cfg.wiki.enabled);
    assert!(!cfg.wiki.auto_on_insights);
}

#[test]
fn wiki_config_explicit_roundtrip() {
    let json = r#"{
        "watch_folders": [],
        "embedder": "mock",
        "chat": "mock",
        "mock_embed_dim": 4,
        "ollama_base_url": "http://127.0.0.1:11434",
        "ollama_embed_model": "nomic-embed-text",
        "ollama_chat_model": "llama3.2",
        "ollama_embed_dim": 768,
        "lark_cli_bin": "lark-cli",
        "wiki": { "enabled": true, "auto_on_insights": true, "ignored_future": 1 }
    }"#;
    let cfg: AppConfig = serde_json::from_str(json).unwrap();
    assert!(cfg.wiki.enabled);
    assert!(cfg.wiki.auto_on_insights);
    let out = serde_json::to_value(&cfg).unwrap();
    assert!(out.get("wiki").is_some());
    assert!(out.get("wiki").unwrap().get("enabled").unwrap().as_bool().unwrap());
}
```

Place in `crates/config/src/types.rs` `#[cfg(test)]` (or `file.rs` tests) per crate convention — discretion. Prefer `types.rs` next to existing `config_flat_json_roundtrip`.

### Frontend

```ts
case "wiki_page":
  return "笔记页";
```

```ts
expect(sourceKindLabel("wiki_page")).toBe("笔记页");
```

---

## Validation Architecture

### Requirement → test map

| Requirement / criterion | Layer | Command | Target |
|-------------------------|-------|---------|--------|
| WIKI-01 default off (pre-v1.10 JSON) | Unit (Rust) | `cargo test -p config` | `pre_v110_config_defaults_wiki_off` (or equiv.) |
| WIKI-01 explicit wiki round-trip + unknown field | Unit (Rust) | `cargo test -p config` | `wiki_config_explicit_roundtrip` |
| WIKI-02 kind string | Unit (Rust) | `cargo test -p store` | `wiki_page_kind_roundtrips` + updated `source_kind_roundtrips` |
| WIKI-02 Library label | Unit (TS) | `npm test` | `src/lib/sourceDisplay.test.ts` |
| Exhaustive matches compile | Compile | `cargo build --manifest-path src-tauri/Cargo.toml` or `cargo test -p store` + tauri-app build | `reindex_source` WikiPage arm |
| No Settings UI regression | Manual/optional | — | Confirm Settings has no wiki toggle (D-10) |
| E2E wiki journey | **Deferred** | Phase 13 | `e2e/specs/wiki.spec.ts` |

### Nyquist validation checklist [VERIFIED against ROADMAP success criteria]

```
Goal: Users can turn wiki on/off in config with no surprise behavior on upgrade;
      wiki pages have a first-class source kind and Library label「笔记页」.
Verify:
 - [ ] cargo test -p store  (wiki_page_kind_roundtrips + source_kind_roundtrips includes WikiPage)
 - [ ] cargo test -p config (missing wiki → both false; explicit wiki round-trip; nested "wiki" in serialize)
 - [ ] cargo build -p tauri-app (or workspace build) — reindex_source WikiPage arm present
 - [ ] npm test — sourceKindLabel("wiki_page") === "笔记页"
 - [ ] npx tsc --noEmit — if AppConfig gains optional wiki
 - [ ] No Settings wiki toggle (D-10)
 - [ ] auto_on_insights not referenced from insights_ops (D-09)
 - [ ] E2E: N/A this phase (Phase 13)
```

---

## Project Constraints

From `.cursor/rules/` and project invariants — planner/executor must honor:

| Rule | Implication for Phase 07 |
|------|---------------------------|
| **tdd-goal-driven** | Write failing store/config/Vitest tests first; then minimal impl. |
| **karpathy-guidelines** | Surgical diff only — no Settings UI, no compile stub beyond reindex arm, no drive-by refactors. |
| **jarvis-stack** | No new deps; Rust stable; keep logic in crates (`store`, `config`), not fat Tauri commands. |
| **e2e-required** | User-facing features need E2E — Phase 07 has no user-facing wiki UX yet; E2E deferred to Phase 13. Do not invent `wiki.spec.ts` early. |
| **frontend-taste** | N/A for Settings; label-only change in `sourceDisplay`. |
| **Single DB owner** | Only `store` opens SQLite — Phase 07 does not add tables or open connections elsewhere. [CITED] ARCHITECTURE / AGENTS.md |
| **Default-off upgrade** | `PROJECT.md` / WIKI-01 — existing installs see no behavior change. |

---

## Key File Map (prescriptive)

| File | Action |
|------|--------|
| `crates/store/src/types.rs` | Add `WikiPage`; `as_str`/`parse`; tests |
| `src-tauri/src/index_ops.rs` | Add `SourceKind::WikiPage` reindex stub arm |
| `crates/config/src/types.rs` | Add `WikiConfig`; `AppConfig.wiki` with `#[serde(default)]` **not flatten**; `Default`; tests |
| `crates/config/src/lib.rs` | Re-export `WikiConfig` (optional but useful for later phases) |
| `src/lib/sourceDisplay.ts` | `wiki_page` → `笔记页` |
| `src/lib/sourceDisplay.test.ts` | Exact `"笔记页"` assert |
| `src/types/ipc.ts` | Optional `wiki?: { enabled; auto_on_insights }` |
| `src/types/config.ts` + `config.test.ts` | Preserve `wiki` in nested/flat helpers if added |
| **Do not touch** | `insights_ops.rs`, Settings UI, `e2e/specs/*`, zip, wiki write path |

---

## Planner Guidance (ordered plans)

Recommend **one plan** (or two tiny plans if preferred):

1. **07-01 — Store kind + reindex stub** → verify: `cargo test -p store`; tauri-app compiles.
2. **07-02 — WikiConfig + FE label (+ optional AppConfig pass-through)** → verify: `cargo test -p config`; `npm test`.

Or a single plan covering both if the diff stays small (~same PR as plan Task 1 commit message).

**Out of plan:** anything that reads `wiki.enabled` to change runtime behavior (compile, UI gate) — that is Phase 10/11.

---

## Confidence

| Area | Tag | Notes |
|------|-----|-------|
| Locked decisions | [CITED] | `07-CONTEXT.md` D-01..D-15 |
| Store/config touch points | [VERIFIED] | Read current `types.rs` files |
| Exhaustive match list | [VERIFIED] | Workspace grep; only `reindex_source` is full match |
| Nested vs flatten | [CITED]+[VERIFIED] | D-04 + current flatten on other buckets |
| FE config wipe | [VERIFIED] | `set_config` + flat TS AppConfig |
| SourceKind IPC serde casing | [ASSUMED] | Pre-existing; Vitest uses snake_case string; do not block Phase 07 |
| Reindex stub message | [ASSUMED] | Phase 10 replaces |

**Overall:** HIGH — Phase 07 is standard patterns; research flags are compile exhaustiveness + nested serde + FE pass-through.

---

## Sources

| Claim | Source | Confidence |
|-------|--------|------------|
| Success criteria / WIKI-01/02 | `.planning/ROADMAP.md`, `REQUIREMENTS.md` | [CITED] |
| Locked decisions | `07-CONTEXT.md` | [CITED] |
| Task 1 interfaces | `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` | [CITED] |
| Pitfall: exhaustive SourceKind | `.planning/research/PITFALLS.md` Pitfall 11 | [CITED] |
| Current SourceKind / AppConfig | `crates/store/src/types.rs`, `crates/config/src/types.rs` | [VERIFIED] |
| `reindex_source` exhaustive match | `src-tauri/src/index_ops.rs` ~559–653 | [VERIFIED] |
| sourceDisplay + tests | `src/lib/sourceDisplay.ts`, `.test.ts` | [VERIFIED] |
| FE config save path | `src/hooks/useJarvisConfig.ts`, `src/types/ipc.ts` | [VERIFIED] |
| store roundtrip test green today | `cargo test -p store source_kind_roundtrips` | [VERIFIED] |

---

*Research complete. Ready for `/gsd-plan-phase 07`.*
