# Phase 07: Wiki kind + config - Pattern Map

**Mapped:** 2026-07-18
**Files analyzed:** 9
**Analogs found:** 9 / 9

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/store/src/types.rs` | model | transform | same file — `SourceKind::Memory` + tests | exact |
| `src-tauri/src/index_ops.rs` | service | request-response | same file — `reindex_source` + `mark_failed` | exact |
| `crates/config/src/types.rs` | config | file-I/O | same file — nested `*Config` + `Default` + `config_flat_json_roundtrip` | role-match* |
| `crates/config/src/lib.rs` | config | request-response | same file — `pub use types::{...}` | exact |
| `src/lib/sourceDisplay.ts` | utility | transform | same file — `case "memory"` | exact |
| `src/lib/sourceDisplay.test.ts` | test | transform | same file — kind label asserts | exact |
| `src/types/ipc.ts` | model | request-response | same file — `AppConfig` interface | exact |
| `src/types/config.ts` | utility | transform | same file — `flatToNested` / `nestedToFlat` | role-match |
| `src/types/config.test.ts` | test | transform | same file — `sampleFlat` + roundtrip test | exact |

\*WikiConfig must use **non-flattened** nested JSON (`#[serde(default)]` on `AppConfig.wiki`). Do **not** copy `#[serde(flatten)]` from Embedding/Chat/Lark/Sync/Agent.

---

## Pattern Assignments

### `crates/store/src/types.rs` (model, transform)

**Analog:** `crates/store/src/types.rs` — `SourceKind::Memory` variant + `as_str` / `parse` + `source_kind_roundtrips`

**Imports pattern** (lines 1–1):
```rust
use serde::{Deserialize, Serialize};
```

**Core kind mapping pattern** (lines 3–41) — add `WikiPage` / `"wiki_page"` the same way `Memory` / `"memory"` was added:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceKind {
    LocalFile,
    LarkDoc,
    LarkMsg,
    LarkSheet,
    LarkMail,
    LarkFile,
    CursorTranscript,
    Memory,
    // Phase 07: WikiPage,
}

impl SourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            // ...
            SourceKind::Memory => "memory",
            // SourceKind::WikiPage => "wiki_page",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            // ...
            "memory" => SourceKind::Memory,
            // "wiki_page" => SourceKind::WikiPage,
            _ => return None, // keep: unknown → None (D-13)
        })
    }
}
```

**Testing pattern** (lines 183–202) — extend array + add dedicated `wiki_page_kind_roundtrips`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_kind_roundtrips() {
        for k in [
            SourceKind::LocalFile,
            // ... existing variants ...
            SourceKind::Memory,
            // SourceKind::WikiPage,
        ] {
            assert_eq!(SourceKind::parse(k.as_str()), Some(k));
        }
        assert_eq!(SourceKind::parse("bogus"), None);
    }
}
```

**Do not:** add schema migration; kind is free TEXT. Do not add `Unknown` variant. Do not change serde rename on `SourceKind` unless a plan task explicitly expands wire-format scope.

---

### `src-tauri/src/index_ops.rs` (service, request-response)

**Analog:** `src-tauri/src/index_ops.rs` — exhaustive `match source.kind` + fail-closed `mark_failed` arms

**Core exhaustive match** (lines 559–653) — new variant requires a new arm or the app fails to compile:
```rust
match source.kind {
    SourceKind::LocalFile => { /* ... */ }
    // ... Lark*, CursorTranscript ...
    SourceKind::Memory => {
        let text = memory_text.unwrap_or_default();
        if text.trim().is_empty() {
            mark_failed(store, source, "memory text missing")?;
            return Ok(false);
        }
        // ...
        Ok(true)
    }
    // Phase 07: SourceKind::WikiPage => { mark_failed(...); Ok(false) }
}
```

**Fail-closed stub pattern** — copy style from LocalFile / CursorTranscript early returns (lines 560–564, 623–627) and `mark_failed` helper (lines 662–675):
```rust
SourceKind::LocalFile => {
    if !std::path::Path::new(&source.uri).is_file() {
        mark_failed(store, source, "local file missing")?;
        return Ok(false);
    }
    // ...
}

fn mark_failed(store: &Store, source: &Source, error: &str) -> Result<(), String> {
    let failed = Source {
        id: source.id.clone(),
        kind: source.kind,
        uri: source.uri.clone(),
        title: source.title.clone(),
        content_hash: source.content_hash.clone(),
        indexed_at: None,
        status: IndexStatus::Failed,
        error: Some(error.to_string()),
        summary: source.summary.clone(),
    };
    store.upsert_source(&failed).map_err(|e| e.to_string())
}
```

**Prescribed WikiPage arm** (from RESEARCH — do not fall through to `LocalFile` / `index_path`):
```rust
SourceKind::WikiPage => {
    mark_failed(store, source, "wiki page reindex not implemented")?;
    Ok(false)
}
```

**Error handling:** map store/index errors with `.map_err(|e| e.to_string())`; Tauri boundary already uses `Result<_, String>`.

---

### `crates/config/src/types.rs` (config, file-I/O)

**Analog (struct + Default + bool defaults):** `SyncConfig` / `AgentConfig` nested structs in same file  
**Anti-analog (do not copy):** `AppConfig`’s `#[serde(flatten)]` on embedding/chat/lark/sync/agent_cfg

**Nested config struct pattern** (lines 68–95) — bool fields with `#[serde(default)]` defaulting false in `Default`:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncConfig {
    pub watch_folders: Vec<String>,
    #[serde(default)]
    pub cursor_projects_root: String,
    #[serde(default)]
    pub auto_summarize_on_index: bool,
    #[serde(default)]
    pub auto_extract_tasks_on_index: bool,
    #[serde(default)]
    pub scheduled_sync_enabled: bool,
    // ...
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            watch_folders: Vec::new(),
            cursor_projects_root: String::new(),
            auto_summarize_on_index: false,
            auto_extract_tasks_on_index: false,
            scheduled_sync_enabled: false,
            // ...
        }
    }
}
```

**AppConfig field placement** (lines 128–145) — add `wiki` **without** flatten:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    // Phase 07 — NOT flatten:
    // #[serde(default)]
    // pub wiki: WikiConfig,
    #[serde(skip)]
    pub fastembed_cache_dir: Option<std::path::PathBuf>,
}
```

**Target WikiConfig shape** (RESEARCH / D-04 — first nested object on disk):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct WikiConfig {
    pub enabled: bool,
    pub auto_on_insights: bool,
}

impl Default for WikiConfig {
    fn default() -> Self {
        Self { enabled: false, auto_on_insights: false }
    }
}
```

**Default wiring** (lines 260–270) — extend `AppConfig::default()`:
```rust
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            embedding: EmbeddingConfig::default(),
            chat_cfg: ChatConfig::default(),
            lark: LarkConfig::default(),
            sync: SyncConfig::default(),
            agent_cfg: AgentConfig::default(),
            // wiki: WikiConfig::default(),
            fastembed_cache_dir: None,
        }
    }
}
```

**Testing pattern** (lines 287–323) — place new tests next to `config_flat_json_roundtrip`; keep asserting flatten for sync/embedder, **and** assert nested `"wiki"` when present:
```rust
#[test]
fn config_flat_json_roundtrip() {
    let cfg: AppConfig = serde_json::from_str(flat_json).expect("deserialize");
    let output = serde_json::to_value(&cfg).expect("serialize");
    assert!(output.get("sync").is_none(), "no 'sync' nesting object in JSON");
    // Phase 07: missing wiki in input → defaults false; serialize may emit "wiki" object
}
```

**Validation / serde:** no `deny_unknown_fields` — unknown keys inside `wiki` ignored (D-06). Missing `wiki` key → `WikiConfig::default()` via `#[serde(default)]` (D-05).

---

### `crates/config/src/lib.rs` (config, request-response)

**Analog:** `crates/config/src/lib.rs` re-export list (line 16)

```rust
pub use types::{AgentOrchestrationMode, AppConfig, ChatProvider, EmbedderProvider};
// Phase 07: add WikiConfig to this list
```

---

### `src/lib/sourceDisplay.ts` (utility, transform)

**Analog:** same file — `memory` case (lines 1–21)

```typescript
export function sourceKindLabel(kind: string): string {
  switch (kind) {
    case "local_file":
      return "本地文件";
    // ...
    case "memory":
      return "记忆";
    // case "wiki_page":
    //   return "笔记页";
    default:
      return kind; // keep (D-15)
  }
}
```

**Do not:** change `default`; add Wiki icon in `LibraryView` (D-02 deferred).

---

### `src/lib/sourceDisplay.test.ts` (test, transform)

**Analog:** same file (lines 9–15)

```typescript
describe("sourceDisplay", () => {
  it("maps source kinds to labels", () => {
    expect(sourceKindLabel("local_file")).toBe("本地文件");
    expect(sourceKindLabel("lark_doc")).toBe("飞书文档");
    expect(sourceKindLabel("cursor_transcript")).toBe("Cursor 会话");
    expect(sourceKindLabel("memory")).toBe("记忆");
    // expect(sourceKindLabel("wiki_page")).toBe("笔记页"); // D-03 exact
  });
});
```

---

### `src/types/ipc.ts` (model, request-response)

**Analog:** same file — flat `AppConfig` (lines 55–90)

```typescript
export interface AppConfig {
  watch_folders: string[];
  embedder: "mock" | "ollama" | "fast_embed" | "cloud";
  // ... flat flattened fields ...
  granted_plugin_permissions: string[];
  // Phase 07 pass-through (prevent Settings save wipe):
  // wiki?: { enabled: boolean; auto_on_insights: boolean };
}
```

Wiki is the **first nested object** on the IPC `AppConfig` shape (Rust serde emits nested `"wiki"` while other buckets stay flat).

---

### `src/types/config.ts` (utility, transform)

**Analog:** same file — `NestedAppConfig` + `flatToNested` / `nestedToFlat` (lines 57–129)

```typescript
export interface NestedAppConfig {
  embedding: EmbeddingConfig;
  chat: ChatConfig;
  lark: LarkConfig;
  sync: SyncConfig;
  agent: AgentConfig;
  // wiki?: { enabled: boolean; auto_on_insights: boolean };
}

export function flatToNested(flat: AppConfig): NestedAppConfig {
  return {
    embedding: { /* ... */ },
    chat: { /* ... */ },
    lark: { /* ... */ },
    sync: { /* ... */ },
    agent: { /* ... */ },
    // wiki: flat.wiki,  // preserve if present
  };
}

export function nestedToFlat(nested: NestedAppConfig): AppConfig {
  return {
    ...nested.embedding,
    ...nested.chat,
    ...nested.lark,
    ...nested.sync,
    ...nested.agent,
    // wiki: nested.wiki,  // must not drop
    agents: nested.agent.agents.map((agent) => ({ ...agent })),
    // ...
  };
}
```

**Save path consumer:** `src/hooks/useJarvisConfig.ts` `handleSaveConfig` spreads `AppConfig` into `set_config` (lines 141–155). If `wiki` is omitted from the payload after load, Rust deserializes defaults → flags forced false (RESEARCH P5). Pass-through is config plumbing, not Settings UI (D-10).

---

### `src/types/config.test.ts` (test, transform)

**Analog:** same file — `sampleFlat` + roundtrip (lines 12–54, 110–120)

```typescript
const sampleFlat: AppConfig = {
  watch_folders: ["/data/docs"],
  // ... all flat fields ...
  granted_plugin_permissions: ["shell_exec"],
  // wiki: { enabled: true, auto_on_insights: false }, // optional fixture
};

it("roundtrips flat IPC payload through nested shape without data loss", () => {
  const nested = flatToNested(sampleFlat);
  const roundtrip = nestedToFlat(nested);
  expect(roundtrip).toEqual(sampleFlat);
});
```

Extend fixture/assert so `wiki` survives roundtrip when present.

---

## Shared Patterns

### SourceKind snake_case strings
**Source:** `crates/store/src/types.rs` (`as_str` / `parse`)  
**Apply to:** store kind + FE `sourceKindLabel` input  
```rust
SourceKind::Memory => "memory",
// WikiPage => "wiki_page"
```
```typescript
case "memory":
  return "记忆";
```

### Exhaustive Rust matches on SourceKind
**Source:** `src-tauri/src/index_ops.rs` `reindex_source`  
**Apply to:** any new `SourceKind` variant (Phase 07: WikiPage stub arm)  
Compiler is source of truth — `cargo build` / match exhaustiveness.

### Nested config structs with Default + serde default
**Source:** `crates/config/src/types.rs` (`SyncConfig`, `AgentConfig`, `AppConfig::default`)  
**Apply to:** `WikiConfig` + `AppConfig.wiki`  
**Critical delta:** use `#[serde(default)]` on the **field**, never `#[serde(flatten)]` on wiki (D-04 / P1).

### Config unit tests colocated in types.rs
**Source:** `crates/config/src/types.rs` `#[cfg(test)] mod tests`  
**Apply to:** `pre_v110_config_defaults_wiki_off`, `wiki_config_explicit_roundtrip`

### FE label switch + Vitest exact equality
**Source:** `src/lib/sourceDisplay.ts` + `.test.ts`  
**Apply to:** `wiki_page` → `"笔记页"` only; keep `default: return kind`

### Error mapping at Tauri/service boundary
**Source:** `index_ops.rs` — `.map_err(|e| e.to_string())` + `mark_failed`  
**Apply to:** WikiPage reindex stub

### FE AppConfig pass-through on save
**Source:** `useJarvisConfig.handleSaveConfig` + `flatToNested`/`nestedToFlat`  
**Apply to:** optional `wiki` on IPC `AppConfig` so Settings save cannot wipe nested flags

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | All Phase 07 files have same-file or adjacent analogs. Closest gap: **no existing non-flattened nested bucket on `AppConfig`** — planner must follow RESEARCH target shape, not copy flatten. |

---

## Metadata

**Analog search scope:** `crates/store/src/types.rs`, `crates/config/src/{types,lib}.rs`, `src-tauri/src/index_ops.rs`, `src/lib/sourceDisplay*.ts`, `src/types/{ipc,config}*.ts`, `src/hooks/useJarvisConfig.ts`  
**Files scanned:** ~12 primary + RESEARCH/CONTEXT file maps  
**Pattern extraction date:** 2026-07-18
