# Phase 16: Related-docs Library panel - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-26
**Phase:** 16-Related-docs Library panel
**Areas discussed:** 面板位置, 选中模型, 行内容与空状态文案, 点击导航

---

## 面板位置

| Option | Description | Selected |
|--------|-------------|----------|
| 右侧分栏 | 列表左、相关右 | |
| 选中行下方展开条 | 贴在当前来源下 | |
| 你来定 | 实现方按单列 Library 选最干净方案 | ✓ |

**User's choice:** 你来定
**Notes:** 未选中完全隐藏；长列表面板内独立滚动；轻量动效 + reduced-motion

---

## 选中模型

| Option | Description | Selected |
|--------|-------------|----------|
| 点击行空白选中 | 行内按钮不抢选中 | ✓ |
| 单独「相关」按钮 | 必须点相关才选中 | |
| 你来定 | — | |

**User's choice:** 点击行空白选中；非 indexed 仍可选（面板诚实空/错）；高亮描边/底色；再点同行取消选中

---

## 行内容与空状态文案

| Option | Description | Selected |
|--------|-------------|----------|
| 标题+类型+片段 | 对齐 REL-04，无分数 | ✓ |
| 标题+片段 | 类型仅图标 | |
| 你来定 | — | |

**User's choice:** 三件套；空「暂无相关文档」；非 indexed「仅已索引来源可查看相关文档」；加载/失败「加载相关文档…」/「相关文档加载失败」

---

## 点击导航

| Option | Description | Selected |
|--------|-------------|----------|
| 选中+滚动+重拉 | 相关链浏览 | ✓ |
| 只选中滚动不重拉 | 面板仍显示原种子邻居 | |
| 你来定 | — | |

**User's choice:** 选中+滚动+重拉；平滑滚动（reduced-motion 瞬间）；连点最后一次胜出；本阶段 focused E2E

---

## Claude's Discretion

- 面板具体构图（侧栏 vs 行下展开等），在「未选中隐藏 / 面板内滚动 / 轻量动效」约束内
- 非 indexed 是否跳过 IPC（文案仍用锁定句）
- `data-testid` 命名细节；Tauri 命令/serde 命名

## Deferred Ideas

- REL-F01 Wiki 邻居降权、MCP、Phase 19 citation gate、图谱 UI、分数阈值 — 均未纳入本阶段
