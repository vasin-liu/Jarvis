---
phase: 05-agent-protocol
plan: "04"
subsystem: ui
tags: [react, chat, e2e]
provides:
  - ChatView amber agent-tool-parse-warnings panel
  - App.tsx lastToolParseWarnings wiring from AskResponse
key-files:
  created: [src/views/ChatView.test.tsx]
  modified: [src/views/ChatView.tsx, src/App.tsx, src/types/ipc.ts, vite.config.ts, .planning/phases/05-agent-protocol/05-VALIDATION.md]
requirements-completed: [AGT-02, AGT-03]
completed: 2026-06-29
---

# Plan 05-04 Summary

**Parse warnings visible in Chat UI via `agent-tool-parse-warnings` panel; frontend types and tests updated.**

## Accomplishments

- `AskResponse.toolParseWarnings` in `ipc.ts`
- `lastToolParseWarnings` state in App.tsx (clear on ask, set from IPC)
- Amber warnings panel in ChatView below assistant area
- `ChatView.test.tsx` asserts testid + warning text
- Updated `05-VALIDATION.md` task map

## Self-Check: PASSED

- `npm test` and `npx tsc --noEmit` green
- E2E: `agent.spec.ts` 3/3, `full-ui.spec.ts` 7/7 (E2E handle-count fix for agent submit)
