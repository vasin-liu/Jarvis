---
status: clean
phase: 08
resolved: "uniquify HashSet reservation + YAML/display newline escape (post-review fix)"
---

# Phase 08 Review — Deterministic Markdown Renderer

Scope: `crates/insights/src/wiki.rs`, `lib.rs`, `Cargo.toml` vs CONTEXT D-01–D-13. Advisory only.

## Findings

crates/insights/src/wiki.rs:225: 🔴 bug: `uniquify_slug` increments count on `base` but never reserves the emitted `{base}-{n}` key. Input order `foo`, `foo`, `foo-2` (or `Foo 2` → slug `foo-2`) yields two drafts with slug segment `foo-2` — violates D-07 uniqueness; Phase 10 disk write would clobber. Fix: after emit, insert/reserve the final slug string in `seen` (or track a `HashSet` of taken segments and bump until free).

crates/insights/src/wiki.rs:235: 🟡 risk: YAML injection via unescaped newlines/CR in `title` / `source_uri`. Only `\` and `"` are escaped; `title`/`uri` with embedded `\n` break the double-quoted scalar and can inject extra frontmatter keys (LLM/Phase 09 adversarial names/URIs). Fix: reject or escape `\n`/`\r` (e.g. replace with space, or YAML `\n` escapes) before `format!`.

crates/insights/src/wiki.rs:243: 🟡 risk: wikilink display sanitize strips `|`/`]` but not newlines; `[[path|disp\n]]…` can split the alias and forge extra markdown/wikilink lines in consumers. Fix: strip or flatten `\n`/`\r` in `sanitize_display` (same discretion as Pitfall 4).

## D-01–D-13 notes (no extra findings)

- D-01–D-04, D-06, D-08–D-13: match CONTEXT (path|display, bidirectional, tree prefixes, frontmatter keys, omit `content_hash`, index sections/omit-empty/no FM).
- D-05: slug alphabet + `e-{hash6}` OK; `/` `\` dropped — path traversal via slug segments mitigated for Phase 10.
- D-07: per-directory maps OK; suffix reservation hole above is the gap.
- `lib.rs` / `Cargo.toml`: exports + workspace `sha2`/`hex` only — fine.

## REVIEW COMPLETE

status: issues
totals: 1🔴 2🟡
