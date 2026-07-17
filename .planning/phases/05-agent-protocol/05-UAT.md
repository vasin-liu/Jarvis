---
status: complete
phase: 05-agent-protocol
source: 05-01-SUMMARY.md, 05-02-SUMMARY.md, 05-03-SUMMARY.md, 05-04-SUMMARY.md
started: 2026-07-17T01:48:00Z
updated: 2026-07-17T04:37:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Open Chat and find agent mode
expected: Launch Jarvis. Open Chat. Agent mode toggle and chat input are visible; UI is usable (not black/blank/frozen).
result: pass
section: user-flow
note: |
  Prior blocker (window unresponsive) addressed mid-session: DeferredEmbedder loads
  FastEmbed off setup thread; initial_scan moved to background after app.manage.
  Retest passed.

### 2. Enable agent mode and submit
expected: Turn on Agent mode. Type a short question. Click agent submit. Busy clears; assistant reply or clear error appears (not endless loading).
result: pass
section: user-flow

### 3. Agent tools / structured protocol still works
expected: Agent-mode question that needs tools completes with a final readable answer. No crash. Tool-call UI may appear when tools ran.
result: pass
section: user-flow

### 4. Parse warnings surface when present
expected: When backend returns tool parse warnings, amber panel data-testid=agent-tool-parse-warnings shows under assistant area with warning text. Skip if cannot trigger this session.
result: skipped
reason: "???????????"
section: user-flow

### 5. Outcome - trust when parsing fails
expected: With warnings visible (or after failed parse), assistant reply and warning text remain readable; app does not silently pretend tools succeeded.
result: skipped
reason: "???????"
section: user-flow

### 6. Technical - agent.spec / JSON mock still green
expected: agent.spec.ts passes under JARVIS_E2E=1 with structured JSON Mock agent journey.
result: pass
section: technical

### 7. Coverage - user-story outcome
expected: Chat exposes agent mode; parse warnings can appear in UI; agent IPC works after commands/agent.rs move. Matches Phase 5 goal outcome.
result: pass
section: coverage

## Summary

total: 7
passed: 5
issues: 0
pending: 0
skipped: 2
blocked: 0

## Gaps

[none yet]
