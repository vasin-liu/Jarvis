# 03-04 Summary: Tasks View Extraction

**Status:** Complete  
**Plan:** 03-04 (wave 4)

## Goal

Extract Tasks UI from `App.tsx` into controlled `TasksView` + dedicated `useTasks` hook; route task IPC through `src/lib/tauri.ts`; preserve E2E testids.

## Delivered

| Artifact | Path |
|----------|------|
| `Task` type | `src/types/tasks.ts` |
| IPC wrappers | `listTasks`, `updateTaskStatus`, `deleteTask` in `src/lib/tauri.ts` |
| Hook | `src/hooks/useTasks.ts` |
| Hook tests | `src/hooks/useTasks.test.ts` |
| Controlled view | `src/views/TasksView.tsx` |
| App wiring | `src/App.tsx` — `useTasks` as single source of truth |

## Verification

- `npm test` — 14 passed (5 files, includes `useTasks.test.ts`)
- `npx tsc --noEmit` — clean
- Tasks testids in `TasksView.tsx`: `task-list`, `task-list-items`, `toggle-task-{id}`
- No task JSX / testids remain in `App.tsx`
- `useTasks` independent of `useLibrary` (no cross-import)

## Notes

- `TaskItem` in `src/types/ipc.ts` re-exports `Task` from `src/types/tasks.ts` for backward compatibility
- `refreshLibrary` calls `refreshTasks` from hook; extract/insights flows unchanged via App handlers
