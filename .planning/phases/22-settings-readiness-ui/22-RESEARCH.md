# Phase 22: Settings readiness UI — Research

**Researched:** 2026-07-29  
**Domain:** React Settings + Vitest + Tauri invoke wrapper  
**Confidence:** HIGH

## Summary

Phase 21 exposes `get_embedder_readiness` → `{ state, message }` camelCase. Settings already loads index status via `useJarvisConfig` and mocks that hook in Vitest. Phase 22 should add a thin TS type + `getEmbedderReadiness` wrapper, wire refresh/poll in the hook, and render a three-state banner near `index-status` with Chinese copy and stable testids.

## Standard Stack

| Layer | Choice | Notes |
|-------|--------|-------|
| Types | `src/types/ipc.ts` | Match Rust serde camelCase |
| IPC | `src/lib/tauri.ts` `invoke` | Same pattern as `getIndexStatus` |
| State | `useJarvisConfig` | Owns refresh + poll while pending + settingsActive |
| UI | `SettingsView.tsx` | Banner near `data-testid="index-status"` |
| Test | Vitest + Testing Library | Mock hook return values |

## Architecture Patterns

### Hook return extension
Add to return object:
- `embedderReadiness: EmbedderReadinessView | null`
- `refreshEmbedderReadiness: () => Promise<void>`

`refreshSettingsData` should also refresh readiness when settings opens.

### Poll
`useEffect` when `settingsActive && embedderReadiness?.state === "pending"` → `setInterval(refresh, 1000)` → clear on cleanup or non-pending.

### Banner
Conditional render by `state`:
- `pending` → testid `embedder-readiness-pending`
- `ready` → `embedder-readiness-ready`
- `failed` → `embedder-readiness-failed` including message + hint

Outer wrapper: `data-testid="embedder-readiness"`.

## Don't Do

- Don't bump package version here
- Don't add FastEmbed E2E smoke (Phase 23)
- Don't redesign Settings layout
- Don't call invoke from SettingsView directly (use hook)

## Open Questions

None — CONTEXT + design lock copy and testids.
