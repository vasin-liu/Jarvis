# 03-03 Summary — Library extraction + Settings Lark sync

## Goal
Extract Library UI from `App.tsx` into `LibraryView` + `useLibrary`, route library IPC through `tauri.ts`, relocate manual Lark sync to Settings.

## Done
- Added `src/types/library.ts` with `Source`, `CursorTranscriptSummary`, `InsightsReport`, `RebuildReport`; `ipc.ts` re-exports.
- Extended `src/lib/tauri.ts` with library/index/Lark IPC wrappers (+ API key helpers used by Settings).
- Added `useLibrary` hook + Vitest tests (TDD); hook does not own `busy`/`indexProgress` and does not register `index-progress`.
- Replaced `LibraryView` children stub with full controlled component (all Library testids preserved).
- Wired `App.tsx` via `useLibrary`; removed App-level `sources`/`cursorCandidates` state.
- Moved Lark paste UI (`lark-sync-url-input`, `lark-sync-submit` + sheet/mail/im rows) to Settings panel.
- Updated `e2e/specs/lark.spec.ts` and `full-ui.spec.ts` to sync from Settings then assert Library `source-list`.

## Verification
- `npm test` — 11 passed (4 files)
- `npx tsc --noEmit` — clean

## Files changed
- `src/types/library.ts` (new)
- `src/types/ipc.ts`
- `src/lib/tauri.ts`
- `src/hooks/useLibrary.ts` (new)
- `src/hooks/useLibrary.test.ts` (new)
- `src/views/LibraryView.tsx`
- `src/App.tsx`
- `e2e/specs/lark.spec.ts`
- `e2e/specs/full-ui.spec.ts`
