# Phase 5: Agent Protocol - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-27
**Phase:** 05-agent-protocol
**Areas discussed:** JSON 工具调用形态, 解析失败 UX, XML 兼容, 编排 warnings

---

## JSON 工具调用形态

| Option | Description | Selected |
|--------|-------------|----------|
| 严格一行裸 JSON | 与「仅回复一行 tool_call」一致 | ✓ |
| 裸 JSON 或 ```json 代码块 | 兼容 markdown 习惯 | |
| 全文扫描首个合法对象 | 最宽松 | |

**User's choice:** 严格一行裸 JSON（trim 首尾空格）

| Option | Description | Selected |
|--------|-------------|----------|
| 多个 JSON → Failed | 与单行契约一致 | ✓ |
| 取第一个 | | |
| 取最后一个 | | |

**User's choice:** 多个合法对象 → Failed + warnings

| Option | Description | Selected |
|--------|-------------|----------|
| prompt 仅 JSON 行 | 无 XML 示例 | ✓ |
| JSON 在前 + deprecated XML 示例 | | |
| 你决定 | | |

**User's choice:** `build_tools_prompt` 只教 JSON 行

| Option | Description | Selected |
|--------|-------------|----------|
| strict name + arguments | 缺字段 → Failed | ✓ |
| 更严格仅两键 | | |
| 宽松 arguments 可缺 | | |

**User's choice:** 严格 schema，多余字段忽略

| Option | Description | Selected |
|--------|-------------|----------|
| trim 后解析 | 容忍首尾空格 | ✓ |
| 精确整行 | | |

**User's choice:** trim line

| Option | Description | Selected |
|--------|-------------|----------|
| ```json → Failed + warning | 不剥 fence | ✓ |
| NotFound | | |
| 自动剥 fence | | |

**User's choice:** fence 视为工具意图但格式错误

---

## 解析失败 UX

| Option | Description | Selected |
|--------|-------------|----------|
| 展示原文 + amber 警告 | | ✓ |
| 不展示 malformed 回复 | | |
| 固定文案替换 | | |

**User's choice:** show_with_warning

| Option | Description | Selected |
|--------|-------------|----------|
| 停止 loop，当最终回答 | 不自动重试 | ✓ |
| retry 一次 | | |
| error 当 tool result 继续 | | |

**User's choice:** stop_final

| Option | Description | Selected |
|--------|-------------|----------|
| inline amber panel | data-testid=agent-tool-parse-warnings | ✓ |
| 全局 banner | | |
| inline + chat-error | | |

**User's choice:** inline_amber

| Option | Description | Selected |
|--------|-------------|----------|
| 启发式检测工具意图 | tool_call / fence / {"name" 行 | ✓ |
| 仅 XML/完整 JSON 行失败 | | |
| 任何 JSON 失败都 Failed | | |

**User's choice:** intent_heuristic

---

## XML 兼容

| Option | Description | Selected |
|--------|-------------|----------|
| one release + eprintln | 与 Phase 4 一致 | ✓ |
| 长期保留 | | |
| 本 phase 删除 | | |

**User's choice:** one_release

| Option | Description | Selected |
|--------|-------------|----------|
| eprintln 一行 deprecated | | ✓ |
| eprintln + snippet | | |
| 仅 UI warnings | | |

**User's choice:** eprintln

| Option | Description | Selected |
|--------|-------------|----------|
| 仅 unit test XML | E2E JSON 主路径 | ✓ |
| 额外 E2E XML | | |
| 不测 XML | | |

**User's choice:** unit_only

| Option | Description | Selected |
|--------|-------------|----------|
| 标签内 JSON 无效 → Failed | | ✓ |
| NotFound | | |

**User's choice:** failed

---

## 编排 warnings

| Option | Description | Selected |
|--------|-------------|----------|
| 合并所有 step + agent 名前缀 | | ✓ |
| 仅最后一步 | | |
| 仅单 agent 模式 | | |

**User's choice:** merge_all + `[agent_name]:` prefix

| Option | Description | Selected |
|--------|-------------|----------|
| 仅最终消息下方 | | ✓ |
| 每 step 卡片内 | | |
| 两者 | | |

**User's choice:** final_only

| Option | Description | Selected |
|--------|-------------|----------|
| 流式与非流式均返回 metadata | | ✓ |
| 仅非流式 | | |
| Tauri event | | |

**User's choice:** same_metadata

---

## Claude's Discretion

- Line-splitting when reply has multiple lines but only one should be JSON.
- Warning message copy and snippet truncation.
- Module layout for parser types.

## Deferred Ideas

- Auto-retry on parse failure.
- Per-step orchestration warning UI.
- Stream events for warnings.
- Native function-calling API.
