# Phase 08: Deterministic Markdown renderer - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-18
**Phase:** 08-deterministic-markdown-renderer
**Areas discussed:** Wikilink style, Page tree / slugs, Frontmatter, index.md shape

---

## Wikilink style

| Option | Description | Selected |
|--------|-------------|----------|
| `[[Acme]]` | Title-only Obsidian links | |
| `[[entities/acme]]` | Path-only links | |
| `[[entities/acme\|Acme]]` | Path + display alias | ✓ |
| You decide | Claude picks | |

**User's choice:** Path|display alias form  
**Notes:** Language preference for session: Chinese replies

| Option | Description | Selected |
|--------|-------------|----------|
| Concepts same + outbound only | Summary → entities/concepts only | |
| Concepts same + bidirectional | Also entity/concept → summary | ✓ |
| Concepts title-only, entities path\|display | Inconsistent | |
| You decide | | |

**User's choice:** Bidirectional + concepts same style

| Option | Description | Selected |
|--------|-------------|----------|
| `[[sources/{slug}\|{title}]]` | Path\|display back-link | ✓ |
| `[[{title}]]` | Title-only back-link | |
| `[[sources/{slug}\|来源]]` | Fixed display「来源」 | |
| You decide | | |

**User's choice:** `[[sources/{slug}|{source_title}]]`

---

## Page tree / slugs

| Option | Description | Selected |
|--------|-------------|----------|
| sources/ + entities/ + concepts/ + root index | Three dirs | ✓ |
| Summary at root; entities/concepts subdirs | Flatter root | |
| All flat at root | Prefix naming | |
| You decide | | |

**User's choice:** Three directories + root index

| Option | Description | Selected |
|--------|-------------|----------|
| Same slugify as entities (hash fallback) | No pinyin crate | ✓ |
| Hash from source_uri | Title-independent | |
| Force pinyin | New dependency | |
| You decide | | |

**User's choice:** Shared slugify + `e-{hash6}`

| Option | Description | Selected |
|--------|-------------|----------|
| Suffix dedupe `-2`, `-3` by order | Deterministic | ✓ |
| Always short hash suffix | Ugly paths | |
| Last write wins | Data loss | |
| You decide | | |

**User's choice:** Suffix dedupe by input order

---

## Frontmatter

| Option | Description | Selected |
|--------|-------------|----------|
| title/type/sources/generated:true; omit content_hash | Phase 10 fills hash | ✓ |
| Same + empty content_hash placeholder | | |
| title/type/sources only | generated deferred | |
| You decide | | |

**User's choice:** Omit content_hash; include generated: true

| Option | Description | Selected |
|--------|-------------|----------|
| source_summary / entity / concept | snake_case | ✓ |
| source / entity / concept | Shorter | |
| Chinese type labels | Brittle | |
| You decide | | |

**User's choice:** snake_case type literals

| Option | Description | Selected |
|--------|-------------|----------|
| One-line JSON-style array | `sources: ["…"]` | ✓ |
| YAML multiline list | | |
| Single string | | |
| You decide | | |

**User's choice:** One-line array

---

## index.md shape

| Option | Description | Selected |
|--------|-------------|----------|
| ## Sources / Entities / Concepts (EN) | Sectioned bullets | ✓ |
| Flat single list | | |
| Chinese section headings | | |
| You decide | | |

**User's choice:** English section headings

| Option | Description | Selected |
|--------|-------------|----------|
| Omit empty sections | | ✓ |
| Always three sections | | |
| You decide | | |

**User's choice:** Omit empty sections

| Option | Description | Selected |
|--------|-------------|----------|
| `# Wiki` only, no frontmatter | Catalog string | ✓ |
| Frontmatter type: index | | |
| `# {source_title} · Wiki` | | |
| You decide | | |

**User's choice:** `# Wiki` without YAML

---

## Claude's Discretion

- Body prose under frontmatter
- Module split inside insights
- Test file placement
- sha2/hex helper details

## Deferred Ideas

- content_hash / disk index merge — Phase 10
- LLM analyze — Phase 09
- UI / zip / E2E — Phases 11–13
- Aggressive escaping for `|` / `]` in display names — note for planner
