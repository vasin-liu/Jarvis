# Phase 6: Settings + Architecture Review - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-30
**Phase:** 06-settings-architecture-review
**Areas discussed:** App.tsx终态, Settings信息架构, 定时同步错误UX, 飞书手动URL, 架构审查QA-02, lib.rs瘦身, TypeScript AppConfig, E2E/Playwright, 全局busy/err, 配置保存, 文案, 启动编排

---

## App.tsx 终态

| Option | Description | Selected |
|--------|-------------|----------|
| 抽 AppShell | App.tsx 仅路由+协调 | ✓ |
| 最小 Sidebar | 布局壳留 App | |
| 你决定 | 以实现为准 | |

**User's choice:** Full AppShell; App = router only; soft line-count target; useAppEvents hook; AppShell = nav+err+busy; view switch stays App; migrate chat handlers to useChat; no barrel exports.

---

## Settings 信息架构

| Option | Description | Selected |
|--------|-------------|----------|
| 手风琴 | 默认展开提供商+索引 | ✓ |
| 单页滚动 | 最小改动 | |
| Tab | 顶部分类 | |

**User's choice:** Accordion; order providers-first; Agent CRUD in advanced section; useJarvisConfig hook-owned state; watch folders in sync section; add settings-section-* testids; keep rebuild in Settings.

---

## 定时同步错误 UX

| Option | Description | Selected |
|--------|-------------|----------|
| 最后一条错误 | meta + 时间戳 | ✓ |
| 历史环缓冲 | N条记录 | |

**User's choice:** Dual meta keys; partial failures count; extend SyncStatusView; display in sync accordion; clear on success; keep manual sync button; extend settings.spec.ts.

---

## 飞书手动 URL

| Option | Description | Selected |
|--------|-------------|----------|
| 折叠高级 | 默认收起 | ✓ |
| 完全移除 | 仅 lark-cli | |
| 保持可见 | 与 Phase 3 冲突 | |

**User's choice:** 「高级 · 手动同步」collapsed; primary = detect + sync + scheduled; preserve testids; errors split Lark vs sync sections; identity/CLI path in main Lark section.

---

## 架构审查 QA-02

| Option | Description | Selected |
|--------|-------------|----------|
| CONCERNS+ROADMAP+REQ | 全量清单 | ✓ |
| 仅 ROADMAP | 最小 | |

**User's choice:** Mark FE-01/SHELL-01 Complete; Known Remaining for non-structural debt; v1.9.6 target; VERIFICATION like Phase 5; QA-04 strict refactor-only.

---

## lib.rs 瘦身

| Option | Description | Selected |
|--------|-------------|----------|
| bootstrap.rs + events.rs | 模块拆分 | ✓ |
| 留 lib.rs | 最小风险 | |

**User's choice:** bootstrap.rs for seeds; events.rs for emit; init_state → state.rs; E2E cmds → commands/chat.rs.

---

## TypeScript AppConfig

| Option | Description | Selected |
|--------|-------------|----------|
| 镜像 Rust 嵌套 | types/config.ts | ✓ |
| 保持扁平 | 仅 Rust 嵌套 | |

**User's choice:** Nested hook API; Vitest roundtrip tests.

---

## E2E / Playwright

| Option | Description | Selected |
|--------|-------------|----------|
| 扩 WDIO | CI 主门禁 | ✓ |
| Playwright 替换 | 大范围 | |

**User's choice:** WDIO extends settings + full-ui; playwright-cli supplemental in e2e/playwright/ (full-nav smoke); Playwright in phase but not CI; harmonized Q1+Q4 as supplement_no_ci.

---

## 补充区

**busy/err:** AppShell holds busy via context/props; global err banner in AppShell; immediate config save; concise 中文 copy.

**启动编排:** Store → config → keychain → memory migration → providers → watcher → scheduler.

---

## Claude's Discretion

Accordion testid suffixes; context vs props for busy; sync error message formatting; playwright script layout; chat-error vs AppShell err split.

## Deferred Ideas

Playwright CI gate; react-router; full Lark manual removal; global sync error banner; CONCERNS full closure; v2.0 bump.
