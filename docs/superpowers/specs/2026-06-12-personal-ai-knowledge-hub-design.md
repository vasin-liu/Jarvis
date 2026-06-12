# 个人 AI 助理 · 本地知识中枢（第一版）设计文档

- 日期：2026-06-12
- 状态：已确认（待用户最终评审）
- 范围：第一版 MVP（本地知识中枢）。更大愿景拆分为后续独立子系统。

---

## 1. 背景与愿景

构建一个**个人 AI 助理**桌面应用，长期愿景包括：管理本地文档与知识库、管理本地 Cursor Agent 的会话与历史、通过 lark-cli 接入飞书（消息/文档/表格/邮件）、形成知识总结与任务、并持续学习与总结；远期演进为一个**应用内的 Agent 平台**。

该愿景包含多个相对独立的子系统，不适合单个 spec 一次性建成。本文档只覆盖**第一版：本地知识中枢**，其余子系统各自走 spec → plan → 实现。

## 2. 第一版范围（Scope）

第一版交付一个 Tauri 2 桌面应用，实现：

- **数据源**：本地文档（监听文件夹、增量索引） + 飞书内容（按需拉取）。
- **能力**：带可点击引用来源的 **RAG 问答**。
- **Provider**：可配置切换的 LLM 与 Embedding provider（本地 Ollama / 本地 ONNX / 云 API）。
- **形态**：Tauri 2（Rust 后端）+ React/TypeScript 前端的本地桌面应用，本地优先、隐私友好。

### 关键决策（已确认）

| 决策点 | 选择 |
| --- | --- |
| 第一版子系统 | 本地知识中枢 |
| 数据源 | 本地文档 + 飞书内容 |
| 交互形态 | 本地桌面应用（Web 界面跑在 Tauri webview） |
| 框架 | Tauri 2（Rust）+ React + TypeScript |
| LLM/Embedding | 可配置 provider 抽象，本地/云可切换 |
| 本地文档纳入方式 | 监听文件夹 + 增量重建 |
| 飞书拉取 | 按需拉取 |
| 知识加工深度 | 仅 RAG 问答（带引用） |
| 存储/检索引擎 | SQLite + sqlite-vec + FTS5 混合检索 |
| 默认本地 Embedding | fastembed-rs（ONNX） |

## 3. 整体架构

```
┌─────────────────────────────────────────────────┐
│  React + TS 前端 (Tauri webview)                  │
│  · 聊天问答界面（带引用来源）                       │
│  · 知识库浏览（来源列表 / 索引状态）                │
│  · 设置（监听文件夹、provider 配置、飞书源）        │
└───────────────┬─────────────────────────────────┘
                │  Tauri commands (IPC, 类型安全)
┌───────────────▼─────────────────────────────────┐
│  Rust 后端                                        │
│  ingest    —— 文档加载与解析（md/pdf/txt/docx）    │
│  chunker   —— 切分 + 元数据标注                    │
│  embedder  —— Embedding provider 抽象（trait）     │
│  store     —— SQLite + sqlite-vec + FTS5（唯一持库）│
│  retriever —— 混合检索（向量 + BM25，RRF 融合）     │
│  llm       —— 对话 provider 抽象（trait）           │
│  rag       —— 编排：检索→拼 prompt→生成→附引用      │
│  watcher   —— 文件夹监听，触发增量索引             │
│  lark      —— 调 lark-cli 子进程，拉取并归一化      │
│  config    —— 配置读写（provider/源/路径）          │
└──────────────────────────────────────────────────┘
```

设计原则：

- 每个模块对外暴露窄接口（trait + 简单结构体），内部实现可替换。
- `embedder`、`llm` 为 trait，本地/云实现插拔。
- `lark` 模块把飞书内容归一化为统一 `Document`，**与本地文档共用同一套索引/检索管线**，不分叉。
- `store` 是唯一持有 SQLite 的模块，避免并发写混乱。
- 前端薄：只发 Tauri command + 渲染流式事件，逻辑全在 Rust。

## 4. 数据流

### 4.1 索引管线（写入）

```
本地文件夹 ──watcher 监听变动──┐
飞书源 ──用户点"同步"──────────┤
                              ▼
                ingest（按类型解析为纯文本 + 元数据）
                              ▼
                chunker（按语义/字数切分，记录来源定位）
                              ▼
                embedder（批量生成向量，带本地缓存）
                              ▼
          store：chunks 表 + 向量(sqlite-vec) + FTS5(BM25)
```

- **增量**：每个来源记录内容哈希；未变跳过，变了只重嵌入受影响 chunk。
- **可观测**：索引进度/失败项以事件推送前端；设置页显示「已索引 N 个来源，M 个失败」。

### 4.2 查询管线（读取）

```
用户提问
   ▼
embedder（问题向量化）
   ▼
retriever：① 向量 top-k  ② BM25 top-k  →  RRF 融合  →  取前若干
   ▼
rag：拼装带来源标注的 prompt（含截断/长度预算控制）
   ▼
llm（流式生成）
   ▼
前端：流式渲染答案 + 可点击引用（定位原文档/飞书链接）
```

- 混合检索用 **RRF（Reciprocal Rank Fusion）** 融合语义与关键词两路。
- 每个 chunk 携带来源信息（文件路径+位置 / 飞书 URL），答案给**可点击引用**。
- 检索与生成均流式。

## 5. 数据模型（SQLite，单文件，位于 app data 目录）

```sql
sources(
  id, kind,                    -- local_file | lark_doc | lark_msg | lark_sheet | lark_mail
  uri,                         -- 本地绝对路径 或 飞书 URL/token
  title, content_hash,         -- 哈希用于增量判断
  indexed_at, status, error
)

chunks(
  id, source_id, ord,          -- ord: 来源内顺序
  text, loc,                   -- loc: 定位信息（行号范围 / 段落 / 锚点）
  token_count
)

vec_chunks(...)                -- sqlite-vec 虚拟表：chunk_id ↔ embedding
chunks_fts(...)                -- FTS5 虚拟表：BM25 全文索引

embed_cache(text_hash, vector) -- 相同文本不重复嵌入
meta(key, value)               -- schema 版本、embedding 模型标识等
```

约束：

- 删除来源时级联清理 chunks / 向量 / FTS 行。
- `meta` 记录 embedding 模型标识；模型变化（维度/模型不一致）触发重新嵌入，向量不可混用。

## 6. Provider 抽象

```rust
trait Embedder {
    fn id(&self) -> &str;          // 写入 meta，判断是否需重嵌
    fn dim(&self) -> usize;
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}

trait ChatModel {
    async fn complete_stream(&self, msgs: &[Message]) -> Stream<Token>;
}
```

第一版实现：

- `Embedder`：`FastEmbedLocal`（默认，本地 ONNX）、`OllamaEmbed`、`CloudEmbed`（OpenAI 兼容）。
- `ChatModel`：`OllamaChat`、`CloudChat`（OpenAI 兼容，覆盖多数云/国内厂商）。

配置在设置页切换并存入 `config`；切换 embedding 提示「需重建索引」。

## 7. 飞书接入（lark 模块）

- 把 lark-cli 当**外部子进程**调用，解析 JSON 输出，不重实现飞书 API。
- 通过可注入的 `CommandRunner` trait 调用，便于测试注入假输出，也为未来"摆脱 CLI"预留替换点。
- 第一版支持四类按需拉取，归一化为统一 `Document`：
  - 文档：`docs +fetch`（飞书文档/Wiki）
  - 消息：`im` 搜索/读取会话记录
  - 表格：`sheets` 读取 → 文本行
  - 邮件：`mail` 读取/搜索
- 前端「添加飞书源」：粘贴 URL 或搜索 → 后端调对应命令 → 拉取 → 进索引管线 → 进知识库列表，可重新同步。
- 前置依赖：lark-cli 已安装并 `auth login`；后端启动健康检查，未就绪在设置页引导。

## 8. 前端界面（React + TS）

三个主视图：

1. **Chat**：输入框、流式答案、答案下方可点击引用卡片（本地引用打开/定位文件，飞书引用打开 URL）。
2. **Library（知识库）**：来源列表（类型图标、标题、索引状态、最后更新）；重新同步飞书源、移除来源；顶部整体索引进度。
3. **Settings（设置）**：监听文件夹增删、Embedding/Chat provider 选择与参数、飞书连接状态、重建索引。

## 9. 错误处理

- 统一错误类型（`thiserror`），按模块分类：解析失败、嵌入失败、lark-cli 失败、provider 网络失败、存储失败。
- **降级而非崩溃**：单来源/单 chunk 失败不影响整体；失败记录到 `sources.status/error`，前端可见、可重试。
- provider 调用带超时与重试（限流/网络抖动）；lark-cli 失败回传 stderr 摘要。
- 检索零命中时明确告知"知识库中未找到相关内容"，不让模型编造。

## 10. 测试策略（测试金字塔：单元 → 集成 → E2E）

- **单元测试**：chunker（切分边界）、RRF 融合排序、增量哈希逻辑、各 parser。
- **集成测试**：临时 SQLite 跑完整「索引→检索」管线；用 `MockEmbedder`/`MockChatModel`（确定性假向量）避免依赖真实模型。
- **lark 模块**：注入假 `CommandRunner` 输出，不依赖真实飞书。
- **前端组件测试**：引用卡片、流式渲染等关键组件。
- **E2E 测试**：`tauri-driver` + WebDriver（WebdriverIO，Windows 走 Edge WebDriver）驱动打包后的真实应用，覆盖关键旅程：
  1. 添加监听文件夹 → 索引完成 → 提问 → 断言答案出现且带引用卡片，点击引用可定位来源。
  2. 添加飞书源（注入 fake runner）→ 同步成功 → Library 出现该来源 → 提问命中飞书内容。
  3. 切换 embedding provider → 提示重建 → 重建后检索仍可用。
  - E2E 在测试配置下注入 `MockEmbedder` / `MockChatModel` / fake `CommandRunner`，结果确定可断言；纳入 CI（Windows runner）。

## 11. 第一版明确不做（Non-Goals）

不做：自动总结、任务提取/执行、持续学习记忆、Cursor 会话索引、定时同步、飞书写操作、多用户/云同步、多 Agent、应用内 Agent 运行时。

这些均为后续独立子系统，各自走 spec → plan。

## 12. 演进路线（远期愿景，不在第一版实现）

- **v2** — Cursor 会话索引：把本地 `agent-transcripts` 纳入知识库。
- **v3** — 自动总结 + 任务提取。
- **v4** — 定时同步 + 持续学习记忆。
- **v5 起 · Agent 平台化**：
  - **多 Agent 接入**：可配置/编排多个 Agent（不同模型、不同角色协作）。
  - **应用内完整 Agent 交互**：在应用内实现完整 Agent 运行时与工具调用循环，**摆脱对终端 CLI 的依赖**（lark-cli 等逐步内化为应用内能力/插件）。
  - **记忆管理（Memory）**：长期/短期记忆的查看、编辑、检索、遗忘。
  - **技能管理（Skills）**：技能安装、启停、编写。
  - **Hooks 管理**：围绕 Agent 事件的钩子配置与脚本。
  - **插件管理（Plugins）**：第三方能力安装与权限管理。

### 为远期预留的扩展点（在第一版即满足）

- `provider` trait（embedder/chat）天然支持未来多模型/多 Agent。
- `lark` 模块用可注入的 `CommandRunner` 抽象，未来"摆脱 CLI"只换实现、不动上层。
- 统一 `Document` + 可扩展 `source kind`，便于接入更多来源与记忆体。
