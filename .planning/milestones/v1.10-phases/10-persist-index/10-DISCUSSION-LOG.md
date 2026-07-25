# Phase 10: Persist + index - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-21
**Phase:** 10-Persist + index
**Areas discussed:** index.md merge, stale page cleanup, content_hash placement, Phase-10 API surface

---

## index.md merge across compiles

| Option | Description | Selected |
|--------|-------------|----------|
| Whole-tree scan rebuild | After each compile, scan wiki tree and rewrite `index.md` | ✓ |
| Surgical upsert | Only upsert this compile’s entries into existing catalog | |
| You decide | Defer to implementer | |

**User's choice:** Whole-tree scan rebuild

### Follow-ups

| Option | Description | Selected |
|--------|-------------|----------|
| Standard dirs only | Scan only `sources/` · `entities/` · `concepts/` | ✓ |
| All root `*.md` except index | Include user-added root notes | |
| Display from frontmatter `title` | `[[path\|title]]`, slug fallback | ✓ |
| Display from filename only | Simpler, worse for CJK | |
| Sort by path/slug | Stable lexicographic | ✓ |
| Sort by title | More readable, can jitter | |

**Notes:** Empty sections omitted (carried from Phase 08). User moved on after four questions.

---

## Stale page cleanup

| Option | Description | Selected |
|--------|-------------|----------|
| Delete disappeared generated pages + index rows | Keep vault/index clean for this source | ✓ |
| Never delete | Only overwrite matching slugs | |
| Delete file XOR index only | Inconsistent | |
| You decide | | |

### Follow-ups

| Option | Description | Selected |
|--------|-------------|----------|
| Belong via `sources` URI in frontmatter | | ✓ |
| Separate per-source manifest | | |
| Never delete non-`generated: true` | | ✓ |
| Delete all disappeared regardless of generated | | |
| Skip write on user-edited same slug | | ✓ |
| Conflict suffix / abort compile | | |

**Notes:** User-edited notes are sacred for both cleanup and overwrite.

---

## content_hash placement

| Option | Description | Selected |
|--------|-------------|----------|
| DB/Document only (no YAML field) | | ✓ |
| Frontmatter + DB | Plan draft style | |
| You decide | | |

### Follow-ups

| Option | Description | Selected |
|--------|-------------|----------|
| Hash full on-disk Markdown incl. YAML | | ✓ |
| Hash body without frontmatter | | |
| On skip-write: still index from disk | | ✓ |
| On skip-write: leave index untouched | | |
| Do not index `index.md` | | ✓ |
| Index `index.md` as WikiPage | | |

**Notes:** Keeps Phase 08 “omit content_hash from render” permanent for v1.10.

---

## Phase-10 API surface

| Option | Description | Selected |
|--------|-------------|----------|
| Library + tests only (no Tauri) | | |
| Library + thin Tauri command (no UI) | | ✓ |
| You decide | | |

### Follow-ups

| Option | Description | Selected |
|--------|-------------|----------|
| Hard-reject when `wiki.enabled=false` | | ✓ |
| Allow compile when disabled (UI-only gate) | | |
| Hard-reject WikiPage inputs | | ✓ |
| Silent skip WikiPage inputs | | |
| Compact success summary | counts + wiki root | ✓ |
| Empty Ok / progress events | | |

**Notes:** UI wiring deferred to Phase 11; no compile progress events this phase.

---

## Claude's Discretion

- Tauri command / summary struct naming
- Exact error strings for disabled / WikiPage rejection
- Failure atomicity details vs current indexer APIs
- Test file placement conventions

## Deferred Ideas

- Phase 11 Library/Settings UX
- Phase 12 Obsidian zip
- Phase 13 E2E + citation trust
- Future: auto_on_insights, bulk compile progress, cross-corpus merge
