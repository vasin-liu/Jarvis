# Phase 14: Tech debt closeout - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-24
**Phase:** 14-tech-debt-closeout
**Areas discussed:** WikiPage重新同步, 导出preflight禁用态, Nyquist12/13, 审计收口标准
**Language:** 中文讨论

---

## WikiPage「重新同步」

| Option | Description | Selected |
|--------|-------------|----------|
| 真·重索引 | 从磁盘 Markdown 再 index_document | |
| 软跳过 | 不标 Failed | ✓ (via 你来定) |
| UI 隐藏 | 不显示重新同步 | ✓ (follow-up) |
| 你来定 | Claude 拍板 | ✓ (first Q) |

**User's choice:** 4 → Claude: soft skip; then 静默; then 隐藏按钮; Rust+Vitest; 批量同软跳过
**Notes:** 不做 LLM 重编译；无 toast

---

## 导出 preflight 禁用态

| Option | Description | Selected |
|--------|-------------|----------|
| 硬错误 WikiDisabled | 与 export 对称 | ✓ |
| 软 hasNotes:false | | |
| 扩展 {enabled,hasNotes} | | |
| 你来定 | | |

**User's choice:** 硬错误 → 复用 crate → FE reportError 原样 → 仅 Rust 单测
**Notes:** —

---

## Nyquist 12/13

| Option | Description | Selected |
|--------|-------------|----------|
| 文档对齐回填 | frontmatter true + 依据 VERIFICATION | ✓ |
| 正式 /gsd-validate-phase | | |
| 显式豁免 false | | |

**User's choice:** 文档对齐 → 只改 nyquist_compliant → 正文加短注释
**Notes:** status/wave_0 不动

---

## 审计收口标准

| Option | Description | Selected |
|--------|-------------|----------|
| 必须重跑 audit → passed | 再 complete-milestone | ✓ |
| 信任 Phase 14 验证即可 | | |

**User's choice:** 必须重审计 → F01/双 writer 可 backlog → Phase 14 无新 E2E
**Notes:** —

---

## Claude's Discretion

- Soft-skip implementation细节（helper vs 内联）
- Preflight gate 放 Tauri vs crate 薄封装（须满足 WikiDisabled 对称）
- VALIDATION 证据注释措辞
- ROADMAP Phase 14 plan 勾选由 plan-phase 决定

## Deferred Ideas

- WIKI-F01 auto_on_insights
- Dual index.md writers consolidation
- True disk reindex / parent recompile
- FE 中文映射 wiki is disabled
- E2E for hidden retry button
