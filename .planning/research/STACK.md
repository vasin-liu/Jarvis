# Stack Research — Jarvis v1.9 Incremental Refactor

**Analysis Date:** 2026-06-17  
**Scope:** Refactor-specific additions to existing v1.8.0 stack (not greenfield)  
**Confidence:** HIGH (existing codebase + established ecosystem)

---

## Standard Stack (Preserved)

No changes to core stack — refactor is structural only.

| Layer | Technology | Location |
|-------|------------|----------|
| Desktop | Tauri 2 | `src-tauri/` |
| Backend | Rust stable, edition 2021, MSRV 1.85 | `crates/*`, `rust-toolchain.toml` |
| Frontend | React 19 + TypeScript + Vite 7 | `src/` |
| Styling | Tailwind CSS v4 + Motion | `src/index.css`, `package.json` |
| Storage | rusqlite (bundled FTS5) + sqlite-vec | `crates/store/` |
| Testing | cargo test, Vitest 3, WebdriverIO 9 + tauri-driver | `e2e/` |

---

## Refactor-Specific Additions

### OS Keychain for secrets

| Crate | Version | Purpose |
|-------|---------|---------|
| `keyring` | latest stable (~3.x) | Cross-platform credential store for `cloud_api_key` |

**Implementation:** `crates/config/src/secrets.rs` — read/write API key via keyring; `config.json` stores key reference or omits secret field. Migrate on load: if plaintext key exists, write to keyring and strip from JSON.

**Files:** `crates/config/src/file.rs`, `crates/config/src/types.rs`, Settings UI in `src/views/SettingsView.tsx`

### React view extraction (no new deps)

Use existing React 19 patterns — no router library required for v1.9 (view state stays in `App.tsx` shell).

| Pattern | Where |
|---------|-------|
| View components | `src/views/{Chat,Library,Tasks,Memory,Settings}View.tsx` |
| IPC hooks | `src/hooks/use{Chat,Library,Tasks,Memory,Config}.ts` |
| Typed invoke wrappers | `src/lib/tauri.ts` |

### Tauri command modules (no new deps)

Rust module split only — `tauri::generate_handler!` unchanged at registration site.

| Module | Commands |
|--------|----------|
| `src-tauri/src/commands/chat.rs` | Session CRUD, ask, stream |
| `src-tauri/src/commands/index.rs` | Rebuild, sync, progress |
| `src-tauri/src/commands/lark.rs` | Lark auth, fetch |
| `src-tauri/src/commands/memory.rs` | Memory CRUD, learn |
| `src-tauri/src/commands/agent.rs` | Agent profiles, orchestration |
| `src-tauri/src/commands/config.rs` | get/set config, providers |
| `src-tauri/src/state.rs` | `AppState` struct |

### Structured agent tool calls

| Approach | When |
|----------|------|
| OpenAI `response_format: json_object` / tool_calls API | Cloud providers (`crates/llm/`) |
| Strict JSON line protocol | Mock + E2E (`JARVIS_E2E=1`) |
| XML parser fallback | Legacy path during migration (`crates/agent/src/tools.rs`) |

**No new crate** — extend `ChatModel` trait with optional structured completion; `ToolCallParser` trait in `crates/agent/`.

### Memory URI scheme

No new dependencies — use existing `store` sources table with `uri = memory://{uuid}` convention.

---

## What NOT to Use

| Avoid | Why |
|-------|-----|
| React Router / TanStack Router | Overkill for 5-view tab nav; adds migration risk |
| Redux / Zustand (new) | App already uses local state; introduce only if hook extraction proves insufficient |
| New workspace crate for "commands" | Tauri commands belong in `src-tauri`, not a separate crate |
| Diesel / SQLx | `store` owns rusqlite; no ORM migration |
| Electron migration | Out of scope |
| Full rewrite to Tauri plugins architecture | Incremental only |

---

## Version Verification

| Package | Verify before adding |
|---------|---------------------|
| `keyring` | crates.io latest stable; test Windows Credential Manager |
| `tauri` | Stay on 2.x line already pinned in workspace |

---

## Confidence Levels

| Recommendation | Confidence | Notes |
|----------------|------------|-------|
| Preserve existing stack | HIGH | Brownfield; no stack debate needed |
| `keyring` for API secrets | HIGH | Standard pattern for desktop apps |
| View/hook extraction without router | HIGH | Matches current tab-based UI |
| `commands/*` module split | HIGH | Standard Rust module pattern |
| Structured tool calls via provider JSON mode | MEDIUM | Provider-specific quirks need per-provider tests |
| `memory://` URI without new table | MEDIUM | May need dedicated table in v2 if collisions persist |

---

*Stack research: 2026-06-17*
