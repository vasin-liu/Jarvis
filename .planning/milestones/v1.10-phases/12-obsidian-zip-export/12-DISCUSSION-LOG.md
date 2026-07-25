# Phase 12: Obsidian zip export - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-23
**Phase:** 12-obsidian-zip-export
**Areas discussed:** Export button placement, Save destination UX, Empty/missing wiki behavior, `.obsidian` stub & zip payload

---

## Export button placement

| Option | Description | Selected |
|--------|-------------|----------|
| Settings only | Put export in「Wiki 笔记」section | |
| Library toolbar only | Vault-level action next to index tools | ✓ |
| Both | Settings + Library | |

**User's choice:** Library toolbar only
**Notes:** After「选择文件索引」; label「导出 Wiki」/`wiki-export`; hide when `wiki.enabled !== true`

---

## Save destination UX

| Option | Description | Selected |
|--------|-------------|----------|
| Save dialog | User picks path via system save | ✓ (with cancel = no-op) |
| Fixed app-data path + reveal | Write under exports/ then reveal | |
| Cancel = no side effects | Explicit no invoke/write on cancel | ✓ (combined with save) |

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed `jarvis-wiki.zip` | Short default name | |
| Dated `jarvis-wiki-YYYY-MM-DD.zip` | Avoid overwrite | ✓ |
| You decide | Fixed name OK | |

| Option | Description | Selected |
|--------|-------------|----------|
| Silent success | Like「生成笔记」 | |
| Brief success message | Show path/filename | ✓ |
| Reveal in explorer | Open folder after save | |

| Option | Description | Selected |
|--------|-------------|----------|
| Global App `busy` | Disable toolbar during export | ✓ |
| Only disable export button | Other actions stay clickable | |
| You decide | Match compile busy | |

**User's choice:** Save + cancel no-op; dated default; brief success; global busy
**Notes:** First use of dialog `save` in this app (open directory already used)

---

## Empty / missing wiki behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Block before save dialog | Message, no dialog | ✓ |
| Stub-only zip allowed | Export empty vault | |
| Save then backend error | Discover empty after path pick | |

| Option | Description | Selected |
|--------|-------------|----------|
| Empty = no md in sources/entities/concepts | index-only counts empty | ✓ |
| Empty = no md at all including index | Any md allows export | |
| You decide | Prefer section scan rule | |

| Option | Description | Selected |
|--------|-------------|----------|
| Backend-only check after save | Conflicts with pre-dialog block | |
| Preflight IPC (disk) | FE checks before dialog | ✓ |
| FE guesses via wiki_page sources | Approximate, may drift | |

| Option | Description | Selected |
|--------|-------------|----------|
| 「还没有可导出的笔记，请先生成笔记」 | Friendly copy | ✓ |
| Technical empty-dir wording | Mentions section dirs | |
| You decide + hard reject on export | Bundle | (hard reject locked as defense-in-depth) |

**User's choice:** Preflight + friendly copy; section-based empty; export hard-rejects empty
**Notes:** —

---

## `.obsidian` stub & zip payload

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal `app.json` only | `{"legacyEditor":false}` | ✓ |
| Richer stub | + appearance/core-plugins | |
| You decide | Minimal OK | |

| Option | Description | Selected |
|--------|-------------|----------|
| Inject stub into zip only | Don’t write to disk wiki/ | ✓ |
| Write stub to wiki_root then zip | Leaves `.obsidian` on disk | |
| You decide | Zip-only | |

| Option | Description | Selected |
|--------|-------------|----------|
| Always use Jarvis stub in zip | Ignore on-disk `.obsidian` | ✓ |
| Prefer on-disk `.obsidian` if present | Non-deterministic | |
| You decide | Deterministic stub | |

| Option | Description | Selected |
|--------|-------------|----------|
| All regular files under wiki_root + stub | Skip on-disk `.obsidian` | ✓ |
| Only `.md` + stub | Cleaner, drops attachments | |
| You decide | All files | |

**User's choice:** Minimal stub zip-only; deterministic stub; all files + path-safe
**Notes:** Matches plan Task 6 sketch with discussion overrides for UI placement and empty/preflight

---

## Claude's Discretion

- Preflight command name / return shape
- Exact success-message UI presentation slot
- `zip` crate version pin (latest stable; plan suggests `zip = "2"`)

## Deferred Ideas

- Phase 13: `wiki.spec.ts` full journey + citation trust (WIKI-08/09)
- No Settings export button (rejected)
- No bidirectional Obsidian sync (product out of scope)
