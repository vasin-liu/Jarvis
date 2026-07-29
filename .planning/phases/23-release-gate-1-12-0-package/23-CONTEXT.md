# Phase 23: Release gate + 1.12.0 package — Context

**Gathered:** 2026-07-30  
**Status:** Ready for planning  
**Source:** Design + implementation plan express path (`/gsd-plan-phase 23 --chain`)

## Phase Goal

Ship a trusted Windows **1.12.0** local package only after reliability smoke passes (SHIP-* + TRUST-*).

## Constraints

- Depends on Phase 22 (Settings readiness UI) — complete
- Product versions today: `package.json` + `tauri.conf.json` still **1.8.0** until this gate
- Spec: `docs/superpowers/specs/2026-07-29-release-hardening-design.md` §6 gate order
- Plan draft: `docs/superpowers/plans/2026-07-29-release-hardening.md` Task 4
- **Do not bump** until smoke checklist records PASS
- No GitHub Release; Windows local package only
- Do not change RAG / `RetrieverConfig` defaults / citation behavior
- CI/E2E must not download real FastEmbed (TRUST-02)
- `store` remains sole SQLite owner

## Decisions

### D-01: Gate order (hard)
**Choice:** (1) Smoke checklist file + automated Mock-path green → (2) Human/agent fills cold-start smoke PASS → (3) bump to 1.12.0 + CHANGELOG → (4) `tauri build` → (5) final install/launch smoke recorded  
**Why:** Spec §6; SHIP-01 before SHIP-02

### D-02: Smoke artifact path
**Choice:** `docs/release/v1.12.0-smoke.md` with checkboxes + Result PASS/FAIL  
**Why:** Plan Task 4 contract; release authority for FastEmbed cold-start

### D-03: Version files
**Choice:** Bump only `package.json` and `src-tauri/tauri.conf.json` to `1.12.0` after smoke PASS  
**Why:** SHIP-02; no Cargo.toml workspace version required unless already synced (do not invent extra bumps)

### D-04: Changelog scope
**Choice:** Create root `CHANGELOG.md` with 1.12.0 hardening + summary of 1.9–1.11 since 1.8.0  
**Why:** SHIP-03; versions were never bumped product-side through GSD milestones

### D-05: Automated trust suite (pre-bump)
**Choice:** Before recording smoke PASS / bump, run:
- `cargo test -p embedder -- --test-threads=4`
- `cargo test -p tauri-app readiness_ -- --test-threads=4`
- `npx vitest run src/views/SettingsView.test.tsx`
**Why:** TRUST-01 Mock-path green; no FastEmbed download in CI commands

### D-06: Build command
**Choice:** `npm run tauri build` (Tauri 2); artifact under `src-tauri/target/release/bundle/`  
**Why:** Project script + plan; SHIP-04

### D-07: Human smoke checkpoint
**Choice:** Cold-start interactive smoke is a **checkpoint** — executor must not invent PASS; if smoke not filled, stop before bump  
**Why:** Window interactivity cannot be faked; chain may pause for operator

### D-08: Scope fence
**Choice:** No RAG changes; no GitHub Release; no multi-OS; no `reload_providers` deferred rewrite; no auto-fallback embedder  
**Why:** Milestone trust freeze + deferred debt

## Claude's Discretion

- Optional one-line README pointer to CHANGELOG / smoke doc
- Exact date string in CHANGELOG (use smoke/build date)
- Whether to run a thin `npm run test:e2e:local` smoke subset if time allows (not required if Settings Vitest + cargo readiness green satisfy TRUST-01 for this milestone)

## Deferred Ideas

- Annotated `v1.12.0` git tag — only if user asks
- `git push` / GitHub Release — out of scope
- Deferred `reload_providers` — carryover debt

## Success Criteria (from ROADMAP)

1. `docs/release/v1.12.0-smoke.md` exists and cold-start smoke is recorded PASS before bump
2. `package.json` + `tauri.conf.json` are **1.12.0** only after smoke PASS
3. `CHANGELOG.md` covers 1.9–1.11 summary + 1.12.0 hardening
4. `tauri build` produces a Windows artifact that launches; Mock-path tests stay green; no CI FastEmbed download required
