# Pitfalls Research

**Domain:** Related-docs (source-overlap) UI + read-only MCP for an existing Tauri+SQLite RAG hub (Jarvis v1.11)
**Researched:** 2026-07-25
**Confidence:** HIGH (Jarvis invariants); MEDIUM (MCP ecosystem security patterns)

**Milestone:** v1.11 — related-docs panel + read-only MCP `search` / `list_sources`  
**Constraints:** No write/mutate MCP tools; only `crates/store` opens SQLite; Core Value = citation trust; E2E with `JARVIS_E2E=1` mocks (no live LLM); reuse existing retriever/agent read paths where possible.

**Provisional phase key** (roadmap may renumber; use these topics when planning):

| Phase | Focus |
|-------|--------|
| **15** | Overlap scoring API (pure logic + Store/retriever queries) |
| **16** | Related-docs Library panel UI (`data-testid`, navigate/open) |
| **17** | MCP transport + **structural** read-only enforcement |
| **18** | MCP tools: `search` + `list_sources` (thin wrappers over existing APIs) |
| **19** | E2E (panel + MCP) + citation/RAG regression gate |

---

## Critical Pitfalls

### Pitfall 1: “Read-only MCP” enforced only by hints or docs

**What goes wrong:**
Server advertises `readOnlyHint` / README “read-only,” but still registers mutate tools (`add_memory`, `complete_task`, plugin `shell_exec`, index rebuild, config write). A client or prompt-injected agent calls them → KB / filesystem mutation from an external Cursor/Claude session.

**Why it happens:**
MCP tool annotations are **advisory, not a security boundary**. Teams wrap the full `agent::tools` match arm “for reuse,” or add “just one helper” write later. Spec/docs say read-only while the allowlist is open.

**How to avoid:**
- **Structural allowlist:** compile-time or startup registry with exactly `{search, list_sources}` (plus optional future read tools). No default “all agent tools.”
- Separate crate/module (`mcp_readonly`) that cannot call mutate Store methods — not a flag on the agent tool loop.
- Unit test: tool list length/names fixed; invoking unknown / write names returns hard error without side effects.
- Never treat `readOnlyHint` as enforcement.

**Warning signs:**
- MCP `tools/list` includes anything beyond search/list.
- Shared `execute_tool(name)` path with agent mode.
- “We’ll add write tools behind a config later” without a separate binary/capability.

**Phase to address:**
**17** (scaffold + allowlist); re-verify in **18** and **19**.

---

### Pitfall 2: Second SQLite connection / long lock holds (DB contention)

**What goes wrong:**
MCP process or Tauri command opens its own `rusqlite::Connection`, or overlap scoring holds `Store`’s `Mutex<Connection>` across embed + large scans. UI freezes during Library select; indexer/scheduler hit `SQLITE_BUSY` / mutex poison; chat and MCP stall together.

**Why it happens:**
Jarvis `Store` is a single `Mutex<Connection>` (v1.9 deferred WAL/pool). New features “need their own connection for concurrency.” Related-docs N² chunk compares or “embed title then vector-search all” run while holding the mutex. MCP stdio server runs in-process and blocks the same mutex on every tool call during indexing.

**How to avoid:**
- **Only** `crates/store` opens SQLite — MCP and related-docs call `Arc<Store>` methods (same rule as wiki).
- Keep Store methods short: query → release lock → compute overlap in memory; never embed inside a held connection lock.
- Cap work: top-K candidates, timeout, cancel on source deselect; avoid full-corpus pairwise on every click.
- Do not open a second app-data `kb.sqlite` connection from an MCP sidecar unless deliberately designed (WAL + busy_timeout + single-writer) — out of scope for v1.11; prefer in-process Store.
- Integration stress: select source while index/sync runs → UI stays responsive; no new `rusqlite` deps outside `store`.

**Warning signs:**
- `rusqlite` in MCP / UI / new crate manifests.
- Library selection freezes for seconds on large libraries.
- `database is locked` during MCP search + background sync.
- Overlap query embeds under `conn.lock()`.

**Phase to address:**
**15** (API design); **17–18** (MCP wiring); verify under load in **19**.

---

### Pitfall 3: Overlap scoring garbage (false “related” noise)

**What goes wrong:**
Panel shows unrelated sources as “related” (generic intros, same language boilerplate, WikiPage summaries, Memory stubs). Users distrust the panel and, worse, treat it as retrieval truth. Raw cosine “87%” displayed as confidence.

**Why it happens:**
Single signal (title embed cosine or one centroid) with a low static threshold; no exclusion of self / same-URI / WikiPage→original pairs; embeddings from dense Chinese/English clusters sit at ~0.8+ for random pairs; length bias and “hub” docs (getting-started) dominate.

**How to avoid:**
- Prefer **multi-signal, rank-based** overlap: e.g. shared chunk hits via existing hybrid retrieve-as-query (title+summary excerpt), or Jaccard on significant terms + vector — not raw cosine alone.
- Hard filters: exclude self `source_id`; optionally demote/exclude `SourceKind::WikiPage` and `Memory` from “related originals” (or label derived kinds clearly).
- Cap results (e.g. 5–8); empty state when below threshold — **empty is better than garbage**.
- Do not show raw cosine as percent; show ordinal “related” / shared-topic without fake precision — or omit scores.
- Unit tests with fixture pairs: known-related vs known-unrelated; threshold must separate them under MockEmbedder.

**Warning signs:**
- Every source relates to every other.
- Wiki/Memory always top related.
- UI shows “92% similar” for boilerplate.
- No negative (unrelated) test cases.

**Phase to address:**
**15** (scoring + tests); **16** (honest UI); spot-check in **19**.

---

### Pitfall 4: E2E / MCP tests that need a live LLM or live MCP client matrix

**What goes wrong:**
Related-docs or MCP specs call real Ollama/cloud, or require full Cursor/Claude Desktop in CI → flaky Windows E2E, secrets, timeouts. Green only on developer machines with API keys.

**Why it happens:**
“MCP must talk to a real client”; overlap scoring mistakenly routed through ChatModel; forgetting `JARVIS_E2E=1` / MockEmbedder for any embed path; no in-process MCP tool invoke harness.

**How to avoid:**
- Related-docs: deterministic MockEmbedder (or pure lexical overlap in tests) — **no ChatModel required**.
- MCP: unit/integration invoke tool handlers directly; optional stdio smoke with a tiny scripted client — not live Claude.
- E2E: panel visibility + navigate with `JARVIS_E2E_FIXTURE`; extend `e2e.rs` only if new IPC needed; add focused spec (e.g. `related-docs.spec.ts`) + row in e2e-required map; keep `qa.spec` / `full-ui` offline.
- Never gate CI on network LLM.

**Warning signs:**
- Spec skips without `OPENAI_API_KEY`.
- E2E waits on streaming tokens for related-docs.
- MCP test spawns external IDE.

**Phase to address:**
**15** (unit), **16**/**18** (IPC), **19** (E2E gate).

---

### Pitfall 5: Citation / RAG trust regressions

**What goes wrong:**
Shipping related-docs or MCP “improves discovery” but Q&A starts citing wrong sources, WikiPages, or MCP-shaped excerpts. Core Value (“answers from my data”) erodes. v1.10 citation E2E goes red or is skipped “because MCP.”

**Why it happens:**
Overlap feature mutates retriever defaults / RRF weights; MCP `search` reimplements retrieval with different top-k or prompt injection into shared cache; UI wires “open related” into ask pipeline as forced context; WikiPage still crowds hybrid search (deferred hard filter from v1.10).

**How to avoid:**
- Related-docs and MCP are **read side-channels** — must not change `rag::ask` / default retriever config.
- MCP `search` = thin wrapper over existing `retriever` + same citation-shaped excerpts as agent `search_knowledge` (or shared helper) — one implementation.
- Phase **19** must re-run `qa.spec` / `full-ui` / wiki citation trust; treat regressions as ship blockers.
- Do not auto-inject related-docs into the next user question without explicit user action.

**Warning signs:**
- Diff touches `crates/rag` defaults “for better MCP.”
- QA citations change with related-docs panel unused.
- Duplicate retrieve implementations diverge in tests.

**Phase to address:**
**15**/**18** (shared retrieve helper); **19** (mandatory regression).

---

### Pitfall 6: Unbounded `list_sources` / `search` payloads (exfil + DoS)

**What goes wrong:**
MCP returns entire library (titles, URIs, paths, memory text) or huge chunk dumps per search. External agent context floods; sensitive local paths leak; host CPU spikes on malicious/repeated calls.

**Why it happens:**
Copying agent `list_sources` string dump without pagination; search returns full chunk bodies; no rate/size limits on MCP tool results.

**How to avoid:**
- Cap list (e.g. title + kind + id; truncate URI); paginate or hard max (e.g. 200) with “truncated” notice.
- Search: same top-k as in-app retrieve; excerpt length limit (agent already truncates ~200 chars — reuse).
- Consider redacting `file://` absolute paths to basename for MCP if product accepts (document the choice).
- Test: fixture library size bound on tool output length.

**Warning signs:**
- Multi-MB MCP tool responses.
- Memory source full bodies in `list_sources`.
- No max on `search` hits.

**Phase to address:**
**18**; abuse cases in **19**.

---

### Pitfall 7: MCP transport exposure (loopback / auth assumptions)

**What goes wrong:**
HTTP/SSE MCP bound to `0.0.0.0` without auth → LAN peers query the personal KB. Or DNS-rebinding / confused-deputy patterns against a local server. Stdio is safer but misconfigured wrappers re-expose HTTP.

**Why it happens:**
Copy-paste MCP server examples use open binds; “local app” assumed unreachable; annotations confused with auth.

**How to avoid:**
- Prefer **stdio** MCP launched by the trusted client for v1.11.
- If HTTP later: bind `127.0.0.1` only + explicit opt-in; document no auth = loopback-only.
- Do not enable remote MCP in this milestone.

**Warning signs:**
- Default listen address `0.0.0.0`.
- “Open in browser” MCP URL on LAN interface.

**Phase to address:**
**17**.

---

### Pitfall 8: Related-docs UI without loading / empty / error (and selection storms)

**What goes wrong:**
Selecting sources fires overlapping requests; stale responses reorder “related” list; blank panel with no empty state; errors silent. Feels broken; E2E flaky.

**Why it happens:**
No request id / abort; fetch on every hover; missing `data-testid` for empty/loading/error (violates frontend taste + e2e-required).

**How to avoid:**
- Debounce or fetch-on-select with abortable invoke; ignore stale responses.
- Explicit empty (“暂无相关文档”) and error states with stable testids.
- E2E: select fixture source → related panel shows expected peer or empty — never hang.

**Warning signs:**
- Related list flickers between two sources.
- No testids on panel.
- Network tab / logs show stacked `related_docs` invokes.

**Phase to address:**
**16**, **19**.

---

## Moderate Pitfalls

### Pitfall 9: Duplicating agent tool semantics under new names

**What goes wrong:**
MCP `search` vs agent `search_knowledge` diverge (different k, filters, Wiki inclusion). Users and agents get inconsistent answers in-app vs in Cursor.

**Prevention:** Shared retrieve helper used by agent tools, Tauri commands, and MCP. One golden integration test for hit ordering on fixtures.

**Phase:** **18**.

---

### Pitfall 10: Treating related-docs as a graph product

**What goes wrong:**
Scope creeps into Louvain/graph UI (explicitly out of v1.11 / v1.10 exclusions). Milestone slips; overlap v1 quality suffers.

**Prevention:** Panel = ranked list + navigate only. No force-directed graph.

**Phase:** **16** (scope lock).

---

### Pitfall 11: Config / Settings surprise for MCP

**What goes wrong:**
MCP starts on every app launch without consent, or port conflicts; users don’t know KB is exposed to other tools.

**Prevention:** Explicit enable in Settings (default off) if MCP is a long-running side server; stdio-only may be client-spawned — document clearly. Golden config: upgrade does not auto-enable network MCP.

**Phase:** **17**, **16** if Settings toggle.

---

## Minor Pitfalls

### Pitfall 12: Missing `data-testid` / spec map row

**Prevention:** Add related-docs (and MCP status if UI) testids; update `.cursor/rules/e2e-required.mdc` spec map; extend `full-ui` only if primary journey.

**Phase:** **16**, **19**.

### Pitfall 13: Exhaustive UI match only for new IPC shapes

**Prevention:** Serde camelCase DTOs for related hits; Vitest for any display helpers; don’t break Library selection types.

**Phase:** **16**.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Wrap full `agent::tools` for MCP | Fast parity | Accidental writes | **Never** for v1.11 |
| Trust `readOnlyHint` only | Spec checkbox | No real boundary | **Never** |
| Second SQLite connection for MCP | Feels concurrent | Lock/BUSY + ownership break | **Never** in v1.11 |
| Raw cosine as “% related” | Pretty UI | Trust destruction | **Never** |
| Change RAG defaults for MCP quality | One knobs turn | Citation regression | **Never** without Phase 19 proof |
| Live LLM in related-docs E2E | Demo polish | Flaky CI | **Never** |
| Graph UI “while we’re here” | Wow factor | Scope blowout | **Never** this milestone |
| Unbounded `list_sources` | Simple code | Exfil / context DoS | Cap always |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `Store` / SQLite | MCP opens own `Connection` | `Arc<Store>` only; short lock scopes |
| `retriever` / RRF | Fork scoring for MCP | Shared retrieve helper; same top-k policy |
| `agent::tools` | Re-export mutate tools | Dedicated read-only registry |
| Library UI | Related fetch changes ask context | Navigate/open only; no silent RAG inject |
| WikiPage / Memory | Rank as top “related” | Filter or label derived kinds |
| Embedder | Live embed in E2E | MockEmbedder / deterministic fixture |
| Indexer / scheduler | Overlap holds lock during sync | Compute after unlock; cap work |
| MCP transport | Bind `0.0.0.0` | stdio or `127.0.0.1` + opt-in |
| E2E harness | Skip qa because “MCP tested” | Phase 19 citation gate mandatory |
| Config | Auto-enable MCP on upgrade | Default off / client-spawned only |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| O(n) or O(n²) overlap on every select | UI hitch | Candidate cap + cheap prefilter | Libraries ≫ hundreds of sources |
| Embed-per-related-query under mutex | Global freeze | Unlock before embed; cache source centroids if needed | Concurrent chat + MCP + select |
| MCP search with huge top-k | Slow tools, huge JSON | Match in-app k; truncate excerpts | Agent clients with large context |
| No debounce on rapid selection | Request storms | Abort + debounce | Power users clicking through Library |
| Full `list_sources` every MCP turn | Latency + tokens | Cap + pagination | Large indexed corpora |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Mutate tools on “read-only” server | KB/fs corruption via prompt injection | Structural allowlist; tests |
| Advisory `readOnlyHint` as ACL | False sense of safety | Code-enforced registry |
| Non-loopback MCP HTTP | LAN exfil of personal KB | stdio / 127.0.0.1 only |
| Dumping absolute paths + memory bodies | Privacy leak to external agent logs | Truncate/redact; excerpt caps |
| Tool results with instruction-like text | Indirect prompt injection into client | Return data-shaped text; no “now call write_…” |
| Opening DB outside `store` | Schema drift + lock races | Single DB owner rule |
| Plugin/shell tools reachable via MCP | RCE class risk | Never register plugins on MCP |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Garbage related list | Distrust panel | Strict threshold + empty state |
| Fake % scores | Misplaced confidence | Rank only or qualitative label |
| Silent MCP enable | Surprise data sharing | Settings/docs clarity; default safe |
| Stale related results | Confusion | Abort in-flight; show loading |
| Related opens wrong source | Lost orientation | Use source_id navigation already in Library |
| No error when overlap fails | “Broken Library” | Error state + retry |

## "Looks Done But Isn't" Checklist

- [ ] **Read-only structure:** `tools/list` === `{search, list_sources}` only — verify unit test
- [ ] **No agent mutate path:** MCP module cannot call add/update/delete Store APIs — verify API surface / compile boundaries
- [ ] **Single DB owner:** No new `rusqlite` outside `crates/store` — verify deps + review
- [ ] **Short locks:** Overlap/MCP search do not embed while holding `Mutex<Connection>` — verify code + contention test
- [ ] **Overlap quality:** Unrelated fixtures do not appear — verify unit/integration
- [ ] **Derived kinds:** Wiki/Memory policy explicit (filter or labeled) — verify tests
- [ ] **Payload caps:** `list_sources` / `search` bounded — verify size test
- [ ] **Transport:** stdio or loopback-only — verify config defaults
- [ ] **UI states:** loading / empty / error testids — verify E2E
- [ ] **No RAG default changes:** `rag`/retriever config untouched — verify diff + qa E2E
- [ ] **E2E offline:** `JARVIS_E2E=1`, no live LLM — verify CI
- [ ] **Citation gate:** `qa.spec` / `full-ui` (and wiki trust) green — verify Phase 19
- [ ] **Spec map:** e2e-required.mdc updated for related-docs — verify rule file

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Mutate tool shipped on MCP | HIGH | Disable MCP; patch allowlist; audit Store for unexpected writes; notify users |
| DB lock storms | MEDIUM | Kill MCP side server; reduce overlap work; restart app; avoid second connection |
| Garbage related panel | LOW | Raise threshold / ship empty-preferring patch; hide panel behind flag if needed |
| Citation regression | HIGH | Revert retriever/RAG diffs; confirm qa green; re-ship MCP as wrapper only |
| LAN MCP exposure | HIGH | Bind localhost / disable HTTP; rotate any secrets that may have leaked via KB content |
| E2E flaky live LLM | LOW | Force Mock providers; delete network assumptions from specs |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Read-only only via hints | **17** | Fixed tool allowlist test; no mutate names |
| DB lock / second connection | **15**, **17–18** | No rusqlite outside store; select-during-index smoke |
| Overlap scoring garbage | **15**, **16** | Related vs unrelated fixtures; honest UI |
| E2E needs live LLM | **15**, **19** | Offline unit + `related-docs` / MCP harness |
| Citation / RAG regression | **18**, **19** | Shared retrieve; qa/full-ui green |
| Unbounded MCP payloads | **18** | Output length caps |
| Transport exposure | **17** | stdio / 127.0.0.1 default |
| UI stale/empty/error gaps | **16**, **19** | testids + E2E journey |
| Divergent search semantics | **18** | Shared helper golden test |
| Graph scope creep | **16** | List-only panel review |
| MCP auto-enable surprise | **17** | Config default test |

## Sources

- Project: `.planning/PROJECT.md` (v1.11 goals, out-of-scope write MCP, citation Core Value)
- Constraints: `.cursor/rules/e2e-required.mdc` (mocks, spec map, CI Windows E2E)
- Architecture: `crates/store` single `Mutex<Connection>`; agent `search_knowledge` / `list_sources` in `crates/agent/src/tools.rs`
- Prior pitfalls: v1.10 wiki PITFALLS (citation crowding, store ownership, E2E mocks) — still apply at edges
- MCP security: annotations advisory / confused-deputy discussions; Red Hat & industry writeups on over-privileged tools and non-loopback binds (ecosystem — MEDIUM confidence)
- Similarity UX: cosine ≠ probability; static thresholds and hub docs cause false positives (domain literature — MEDIUM)
- SQLite: single-writer; extra connections → BUSY; keep short critical sections (HIGH for this codebase)

---
*Pitfalls research for: Jarvis Related-docs + Read-only MCP (v1.11)*
*Researched: 2026-07-25*
*)
