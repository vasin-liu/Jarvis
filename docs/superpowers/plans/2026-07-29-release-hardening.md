# v1.12.0 Release Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make FastEmbed deferred init observable and failure-visible in the UI, verify release cold-start responsiveness, then bump to **1.12.0** and produce a local Windows install package with changelog.

**Architecture:** Keep existing `DeferredEmbedder` + deferred `initial_scan`. Add `ready_state()` + injectable wait timeout for tests; keep an `Option<Arc<DeferredEmbedder>>` watch handle on `AppState`; expose `get_embedder_readiness` IPC; show Pending/Ready/Failed in Settings. Only after automated tests + release smoke pass: version bump, `CHANGELOG.md`, `tauri build`.

**Tech Stack:** Rust workspace (`embedder`, `tauri-app`), React 19 + Vitest, Tauri 2 bundling on Windows.

**Spec:** `docs/superpowers/specs/2026-07-29-release-hardening-design.md`

## Global Constraints

- Product version target: **1.12.0** (do not bump until Task 4 gate)
- Platform: Windows local install package only; no GitHub Release requirement
- Do not change RAG / `RetrieverConfig::default` / citation behavior
- Do not auto-fallback embedder provider on FastEmbed failure
- E2E/CI must not download real FastEmbed models; release smoke is the FastEmbed cold-start authority
- `store` remains sole SQLite owner
- TDD: failing test before production code each behavioral task

## File map

| File | Responsibility |
|------|----------------|
| `crates/embedder/src/deferred.rs` | `ready_state`, injectable wait timeout, unit tests |
| `crates/embedder/src/lib.rs` | Re-export `EmbedderReadyState` |
| `src-tauri/src/state.rs` | Hold `deferred_embedder: Option<Arc<DeferredEmbedder>>`; `embedder_readiness()` |
| `src-tauri/src/commands/config.rs` | `get_embedder_readiness` command + `EmbedderReadinessView` |
| `src-tauri/src/commands/mod.rs` | Re-export new command |
| `src-tauri/src/lib.rs` | Register command in `generate_handler!` |
| `src/types/ipc.ts` | `EmbedderReadinessView` type |
| `src/lib/tauri.ts` | `getEmbedderReadiness()` wrapper |
| `src/hooks/useJarvisConfig.ts` | Load/poll readiness for Settings |
| `src/views/SettingsView.tsx` | Readiness banner + `data-testid`s |
| `src/views/SettingsView.test.tsx` | Vitest for pending/ready/failed UI |
| `docs/release/v1.12.0-smoke.md` | Release cold-start smoke checklist |
| `CHANGELOG.md` | Create; summarize 1.9–1.11 + 1.12.0 |
| `package.json`, `src-tauri/tauri.conf.json` | Version → `1.12.0` (Task 4 only) |

---

### Task 1: DeferredEmbedder readiness + testable timeout

**Files:**
- Modify: `crates/embedder/src/deferred.rs`
- Modify: `crates/embedder/src/lib.rs`
- Test: inline `#[cfg(test)]` in `deferred.rs`

**Interfaces:**
- Consumes: existing `DeferredEmbedder::{new, fulfill, fail}`, `EmbedError`
- Produces:
  - `pub enum EmbedderReadyState { Pending, Ready, Failed { message: String } }`
  - `DeferredEmbedder::with_wait_timeout(id, dim, timeout: Duration) -> Self`
  - `DeferredEmbedder::new` delegates to `with_wait_timeout(..., Duration::from_secs(300))`
  - `DeferredEmbedder::ready_state(&self) -> EmbedderReadyState` (non-blocking snapshot)
  - Existing `fulfill` / `fail` / `embed` behavior unchanged except using stored timeout

- [ ] **Step 1: Write failing tests**

Append to `deferred.rs` tests module (keep existing two tests):

```rust
#[test]
fn ready_state_pending_then_ready() {
    let deferred = DeferredEmbedder::new("deferred:test", 8);
    assert!(matches!(
        deferred.ready_state(),
        EmbedderReadyState::Pending
    ));
    deferred.fulfill(Arc::new(MockEmbedder::new(8)));
    assert!(matches!(deferred.ready_state(), EmbedderReadyState::Ready));
}

#[test]
fn ready_state_failed_includes_message() {
    let deferred = DeferredEmbedder::new("deferred:test", 8);
    deferred.fail("onnx missing");
    match deferred.ready_state() {
        EmbedderReadyState::Failed { message } => {
            assert!(message.contains("onnx missing"));
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[tokio::test]
async fn short_timeout_errors_while_pending() {
    let deferred = DeferredEmbedder::with_wait_timeout(
        "deferred:test",
        8,
        Duration::from_millis(30),
    );
    let err = deferred
        .embed(&[String::from("hi")])
        .await
        .expect_err("should time out");
    assert!(
        err.to_string().contains("timed out"),
        "got: {err}"
    );
}
```

- [ ] **Step 2: Run tests — expect compile/fail**

Run: `cargo test -p embedder ready_state_ -- --nocapture`
Expected: FAIL (missing `EmbedderReadyState` / `ready_state` / `with_wait_timeout`)

Also: `cargo test -p embedder short_timeout_errors_while_pending -- --nocapture`
Expected: FAIL similarly

- [ ] **Step 3: Minimal implementation**

In `deferred.rs`:

1. Add and `pub use` via `lib.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmbedderReadyState {
    Pending,
    Ready,
    Failed { message: String },
}
```

2. Add field `wait_timeout: Duration` on `DeferredEmbedder`.

3. Constructors:

```rust
pub fn new(id: impl Into<String>, dim: usize) -> Self {
    Self::with_wait_timeout(id, dim, Duration::from_secs(300))
}

pub fn with_wait_timeout(
    id: impl Into<String>,
    dim: usize,
    wait_timeout: Duration,
) -> Self {
    Self {
        id: id.into(),
        dim,
        wait_timeout,
        pair: Arc::new((Mutex::new(Slot::Pending), Condvar::new())),
    }
}
```

4. `ready_state`:

```rust
pub fn ready_state(&self) -> EmbedderReadyState {
    let (lock, _) = &*self.pair;
    let slot = lock.lock().unwrap();
    match &*slot {
        Slot::Pending => EmbedderReadyState::Pending,
        Slot::Ready(_) => EmbedderReadyState::Ready,
        Slot::Failed(msg) => EmbedderReadyState::Failed {
            message: msg.clone(),
        },
    }
}
```

5. In `wait_ready`, replace `Duration::from_secs(300)` with `self.wait_timeout`.

6. In `lib.rs`: `pub use deferred::{DeferredEmbedder, EmbedderReadyState};`

- [ ] **Step 4: Run tests — expect pass**

Run: `cargo test -p embedder -- --test-threads=4`
Expected: all embedder tests PASS (including prior fulfill/fail tests)

- [ ] **Step 5: Commit**

```bash
git add crates/embedder/src/deferred.rs crates/embedder/src/lib.rs
git commit -m "feat(embedder): expose DeferredEmbedder ready_state and testable timeout"
```

---

### Task 2: AppState watch handle + `get_embedder_readiness` IPC

**Files:**
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/commands/config.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs` (`generate_handler!`)
- Test: add unit-style test in `src-tauri` if a small test module exists; otherwise add `crates/embedder` coverage already done — add a focused test in `state.rs` under `#[cfg(test)]` **or** verify via `cargo check -p tauri-app` + manual invoke in Task 3 Vitest mocks. Prefer a pure helper test:

**Interfaces:**
- Consumes: `Arc<DeferredEmbedder>`, `EmbedderReadyState`
- Produces:
  - `AppState.deferred_embedder: Option<Arc<DeferredEmbedder>>`
  - `AppState::embedder_readiness(&self) -> EmbedderReadyState` — `Ready` when `None`
  - `EmbedderReadinessView { state: String, message: Option<String> }` with `#[serde(rename_all = "camelCase")]`
  - `get_embedder_readiness(state) -> Result<EmbedderReadinessView, String>`

Mapping helper (put in `config.rs` or `state.rs`):

```rust
fn readiness_view(state: EmbedderReadyState) -> EmbedderReadinessView {
    match state {
        EmbedderReadyState::Pending => EmbedderReadinessView {
            state: "pending".into(),
            message: None,
        },
        EmbedderReadyState::Ready => EmbedderReadinessView {
            state: "ready".into(),
            message: None,
        },
        EmbedderReadyState::Failed { message } => EmbedderReadinessView {
            state: "failed".into(),
            message: Some(message),
        },
    }
}
```

- [ ] **Step 1: Write failing shell compile check for new API**

Add to `src-tauri/src/state.rs` at end:

```rust
#[cfg(test)]
mod readiness_tests {
    use super::*;
    use embedder::{DeferredEmbedder, EmbedderReadyState, MockEmbedder};
    use std::sync::Arc;

    #[test]
    fn none_watch_means_ready() {
        // Will fail until embedder_readiness exists — construct minimal stub in test
        // by calling a free function once extracted, OR skip and rely on Step 2 compile.
    }
}
```

Prefer extracting a pure function to avoid constructing full `AppState`:

```rust
// in state.rs
pub(crate) fn readiness_from_watch(
    deferred: Option<&DeferredEmbedder>,
) -> EmbedderReadyState {
    match deferred {
        None => EmbedderReadyState::Ready,
        Some(d) => d.ready_state(),
    }
}
```

Failing test:

```rust
#[cfg(test)]
mod readiness_tests {
    use super::readiness_from_watch;
    use embedder::{DeferredEmbedder, EmbedderReadyState, MockEmbedder};
    use std::sync::Arc;

    #[test]
    fn no_deferred_is_ready() {
        assert_eq!(
            readiness_from_watch(None),
            EmbedderReadyState::Ready
        );
    }

    #[test]
    fn deferred_pending_and_failed() {
        let d = DeferredEmbedder::new("deferred:t", 8);
        assert_eq!(readiness_from_watch(Some(&d)), EmbedderReadyState::Pending);
        d.fail("boom");
        assert!(matches!(
            readiness_from_watch(Some(&d)),
            EmbedderReadyState::Failed { .. }
        ));
        let _ = MockEmbedder::new(8); // keep import used if needed
        let _ = Arc::new(());
    }
}
```

(Adjust unused imports — if `MockEmbedder` unused, omit it.)

- [ ] **Step 2: Run test — expect fail/compile error**

Run: `cargo test -p tauri-app readiness_from_watch -- --nocapture`
Expected: FAIL until `readiness_from_watch` exists

- [ ] **Step 3: Implement state + IPC**

1. `AppState` add field:

```rust
pub(crate) deferred_embedder: Option<Arc<DeferredEmbedder>>,
```

2. In `init_state` FastEmbed branch: after creating `deferred`, set `deferred_embedder: Some(deferred.clone())` and still put `deferred as Arc<dyn Embedder>` into `embedder` mutex. Non-deferred branch: `deferred_embedder: None`.

3. Add `readiness_from_watch` +:

```rust
impl AppState {
    pub(crate) fn embedder_readiness(&self) -> EmbedderReadyState {
        readiness_from_watch(self.deferred_embedder.as_deref())
    }
}
```

4. In `commands/config.rs`:

```rust
use embedder::EmbedderReadyState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedderReadinessView {
    pub state: String,
    pub message: Option<String>,
}

#[tauri::command]
pub fn get_embedder_readiness(
    state: State<'_, AppState>,
) -> Result<EmbedderReadinessView, String> {
    Ok(match state.embedder_readiness() {
        EmbedderReadyState::Pending => EmbedderReadinessView {
            state: "pending".into(),
            message: None,
        },
        EmbedderReadyState::Ready => EmbedderReadinessView {
            state: "ready".into(),
            message: None,
        },
        EmbedderReadyState::Failed { message } => EmbedderReadinessView {
            state: "failed".into(),
            message: Some(message),
        },
    })
}
```

5. Re-export from `commands/mod.rs` and register in `lib.rs` `generate_handler!` next to `get_index_status`.

6. **Important:** `reload_providers` / `save_config` leave `deferred_embedder` as-is for this milestone (spec: optional debt). Document with a one-line comment above the field: `// Cold-start watch only; Settings reload stays sync build_embedder (v1.12 debt).`

- [ ] **Step 4: Verify**

Run:
```
cargo test -p tauri-app readiness_ -- --nocapture
cargo check -p tauri-app
```
Expected: PASS / Finished

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/src/commands/config.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(tauri): expose get_embedder_readiness IPC for deferred FastEmbed"
```

---

### Task 3: Settings readiness UI + Vitest

**Files:**
- Modify: `src/types/ipc.ts`
- Modify: `src/lib/tauri.ts`
- Modify: `src/hooks/useJarvisConfig.ts`
- Modify: `src/views/SettingsView.tsx`
- Modify: `src/views/SettingsView.test.tsx`

**Interfaces:**
- Consumes: `get_embedder_readiness` → `{ state, message }`
- Produces: UI with `data-testid="embedder-readiness"`, `"embedder-readiness-pending"`, `"embedder-readiness-ready"`, `"embedder-readiness-failed"`

- [ ] **Step 1: Add types + wrapper**

`src/types/ipc.ts`:

```ts
export type EmbedderReadyState = "pending" | "ready" | "failed";

export interface EmbedderReadinessView {
  state: EmbedderReadyState;
  message: string | null;
}
```

`src/lib/tauri.ts`:

```ts
export function getEmbedderReadiness() {
  return invoke<EmbedderReadinessView>("get_embedder_readiness");
}
```

- [ ] **Step 2: Write failing Vitest**

In `SettingsView.test.tsx`, extend props if Settings receives readiness via props — **prefer props injection** for testability:

Add optional prop to `SettingsView`:

```ts
embedderReadiness?: EmbedderReadinessView | null;
```

Render banner when `embedderReadiness` is non-null.

Failing tests:

```ts
it("shows pending embedder readiness", () => {
  render(
    <SettingsView
      {...minimalProps}
      embedderReadiness={{ state: "pending", message: null }}
    />,
  );
  expect(screen.getByTestId("embedder-readiness-pending")).toBeTruthy();
});

it("shows failed embedder readiness with message", () => {
  render(
    <SettingsView
      {...minimalProps}
      embedderReadiness={{
        state: "failed",
        message: "onnx missing",
      }}
    />,
  );
  const el = screen.getByTestId("embedder-readiness-failed");
  expect(el.textContent).toMatch(/onnx missing/);
  expect(el.textContent).toMatch(/Settings|Mock|Ollama|缓存|模型/); // actionable hint in Chinese copy
});
```

Adapt `minimalProps` to whatever the existing test harness already passes (`config`, handlers, etc.).

Run: `npx vitest run src/views/SettingsView.test.tsx`
Expected: FAIL (prop/UI missing)

- [ ] **Step 3: Implement UI + wire hook**

Settings banner (place above or inside the index-status accordion), Chinese copy:

- pending: 「本地嵌入模型加载中…」
- ready: 「本地嵌入模型已就绪」
- failed: 「本地嵌入模型加载失败：{message}。请检查模型/缓存目录，或在上方改用 Mock / Ollama。」

`useJarvisConfig`:

- state: `embedderReadiness`
- `refreshEmbedderReadiness` calling `getEmbedderReadiness()`
- On settings refresh, also fetch readiness
- While `state === "pending"`, `setInterval` 1000ms refresh; clear on unmount or when not pending

Pass `embedderReadiness` into `SettingsView` from `App.tsx` / wherever Settings is mounted (follow existing prop pattern for `indexStatus`).

- [ ] **Step 4: Run Vitest — expect pass**

Run: `npx vitest run src/views/SettingsView.test.tsx`
Expected: PASS

Also: `cargo check -p tauri-app` still green.

- [ ] **Step 5: Commit**

```bash
git add src/types/ipc.ts src/lib/tauri.ts src/hooks/useJarvisConfig.ts src/views/SettingsView.tsx src/views/SettingsView.test.tsx src/App.tsx
git commit -m "feat(ui): show FastEmbed deferred readiness in Settings"
```

(Only stage files actually touched.)

---

### Task 4: Release gate — smoke doc, bump, changelog, build

**Files:**
- Create: `docs/release/v1.12.0-smoke.md`
- Create: `CHANGELOG.md`
- Modify: `package.json` (version)
- Modify: `src-tauri/tauri.conf.json` (version)
- Optional: `README.md` one-line pointer to changelog / smoke doc

**Gate rule:** Do **not** bump versions until Steps 1–2 evidence exists in the smoke doc (checkboxes filled by the human or agent who ran smoke).

- [ ] **Step 1: Write smoke checklist (empty results)**

Create `docs/release/v1.12.0-smoke.md`:

```markdown
# v1.12.0 Release Smoke (Windows)

**Build:** `npm run tauri build` (or project-equivalent) — release profile  
**Machine:** ________  
**Date:** ________

## Checks

- [ ] Cold start: main window paints and accepts clicks within ~5s while FastEmbed may still load
- [ ] Settings → embedder readiness shows pending then ready (FastEmbed default)
- [ ] With broken/missing FastEmbed cache (optional): readiness shows failed + message; app does not freeze
- [ ] Existing chat/library smoke: open Library, open Chat (Mock chat OK)
- [ ] Installer/portable artifact launches after install/extract

## Artifacts

- Installer path: ________
- `package.json` version after bump: 1.12.0

## Result

- [ ] PASS — proceed to version bump + changelog commit
- [ ] FAIL — do not bump; file issues under `.planning/` or fix first
```

Commit smoke template first (before bump):

```bash
git add docs/release/v1.12.0-smoke.md
git commit -m "docs: add v1.12.0 Windows release smoke checklist"
```

- [ ] **Step 2: Run pre-bump verification**

Run:
```
cargo test -p embedder -- --test-threads=4
cargo test -p tauri-app readiness_ -- --test-threads=4
npx vitest run src/views/SettingsView.test.tsx
```
Expected: all PASS

Then perform **manual** release smoke on a release build (dev `tauri build` as needed). Fill checkboxes in `docs/release/v1.12.0-smoke.md`.

If FAIL → stop; do not continue to Step 3.

- [ ] **Step 3: Bump version + changelog**

Set `"version": "1.12.0"` in:
- `package.json`
- `src-tauri/tauri.conf.json`

Create `CHANGELOG.md`:

```markdown
# Changelog

## 1.12.0 — 2026-07-29

### Fixed / Hardened
- FastEmbed deferred init readiness visible in Settings (pending / ready / failed)
- Release cold-start path verified via Windows smoke checklist

### Included since 1.8.0 (summary)
- **1.9** Structural refactor (views/hooks, Tauri commands, keychain, `memory://`, JSON agent protocol)
- **1.10** Wiki compile layer + Obsidian zip export + citation E2E trust
- **1.11** Library related-docs panel + read-only `jarvis-mcp` (`search` / `list_sources`)

See `.planning/MILESTONES.md` for planning archives.
```

- [ ] **Step 4: Build package + final smoke**

Run: `npm run tauri build`  
Expected: exit 0; artifact under `src-tauri/target/release/bundle/` (nsis/msi/exe per `targets: all`)

Re-run install/open smoke; mark artifact path in smoke doc.

- [ ] **Step 5: Commit release metadata**

```bash
git add package.json src-tauri/tauri.conf.json CHANGELOG.md docs/release/v1.12.0-smoke.md
git commit -m "chore: release 1.12.0 — hardening + Windows package metadata"
```

Optional annotated tag (only if user asks; config may already want tags):

```bash
git tag -a v1.12.0 -m "v1.12.0 Release Hardening"
```

Do **not** `git push` unless user requests.

---

## Spec coverage checklist

| Spec requirement | Task |
|------------------|------|
| Cold-start responsive (deferred path kept) | 1–2 (preserve), 4 smoke |
| Failed init user-visible | 2 IPC + 3 UI |
| Gate: no bump before reliability evidence | 4 Steps 1–2 before 3 |
| 1.12.0 + changelog + Windows package | 4 |
| Unit tests fulfill/fail/timeout | 1 |
| No RAG default changes | Global Constraints |
| No GitHub Release / multi-platform | Task 4 scope |
| reload_providers sync debt deferred | Task 2 comment |

## Placeholder / consistency self-review

- No TBD steps; concrete APIs (`EmbedderReadyState`, `get_embedder_readiness`, testids)
- `ready_state` / `EmbedderReadyState` names consistent across tasks
- Version bump isolated to Task 4 after smoke
