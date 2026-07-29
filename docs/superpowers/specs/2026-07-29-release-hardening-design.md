# Jarvis v1.12.0 — Release Hardening 设计文档

- 日期：2026-07-29
- 状态：已确认（待实现计划）
- 范围：发布硬化里程碑 — 启动可靠性闸门 + 本机 Windows 安装包（版本 **1.12.0**）
- 方法：可靠性优先（Approach A）

---

## 1. 背景与动机

GSD 功能里程碑已推进到 **v1.11**（related-docs + 只读 MCP），但产品包版本仍停在 **1.8.0**（`package.json` / `tauri.conf.json`）。用户需要一份可安装的正式 Windows 包，而不是继续堆功能。

v1.9 UAT 期间已引入 `DeferredEmbedder` + deferred `initial_scan`，避免 FastEmbed/ONNX 阻塞 Tauri setup 导致「窗口无响应」。该路径需在 **release 配置**下验证并补齐产品闭环（失败可观测），再允许版本 bump 与打包。

**核心价值不变：** 用户可在本地对自有知识提问并获得带引用的答案。本里程碑不扩展检索/Wiki/MCP 能力，只让已有能力以可发布形态交付。

---

## 2. 目标与成功标准

**一句话：** 在 Windows release 配置下冷启动可响应；FastEmbed 后台加载失败可感知；通过后才 bump 到 1.12.0 并产出本机安装包 + changelog。

| # | 成功标准 | 验证方式 |
|---|----------|----------|
| 1 | Release 包启动后主窗口可交互，不因 FastEmbed 初始化或首次 `initial_scan` 长时间无响应 | 本机 release 冷启动 smoke |
| 2 | FastEmbed init 失败时应用不静默卡死；用户可见 Failed + 原因；索引/问答不无限无反馈挂起 | unit + 壳层 IPC + smoke（坏缓存/失败注入） |
| 3 | 可靠性证据齐备后，才允许 `package.json` / `tauri.conf.json` → **1.12.0** | 门禁顺序（见 §6） |
| 4 | `tauri build` 产出 Windows 安装包（或项目既有 bundle）+ changelog 覆盖 1.9–1.11 与本里程碑修复 | 本机安装/打开 smoke |
| 5 | 既有 Mock/E2E 旅程不回归 | `cargo test` 相关 crate + 既有 E2E/Vitest |

### 非目标

- GitHub Release / 强制 CI 发版资产
- Linux / macOS 安装包
- Wiki 批量/自动编译、MCP 新工具、related-docs 新能力
- v2.0.0 大版本 framing
- 重做 embedder 架构或改 RAG / `RetrieverConfig` 默认
- 强制自动降级到 Mock/Ollama（失败时仅提示用户手动改配置）

---

## 3. 关键决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 里程碑焦点 | 发布向（非新功能） | 包版本落后功能账本；用户要可装包 |
| 产品版本号 | **1.12.0**（与 GSD v1.11 解耦） | 「发布硬化」单独编号，避免与功能里程碑混淆 |
| 可靠性 vs 打包 | 可靠性闸门后再 bump/打包（Approach A） | 避免「版本新、启动仍抖」 |
| Ship 完成定义 | 本机 Windows 安装包 + changelog + smoke | 不强制 GitHub Release |
| 架构策略 | hardening 现有 deferred 路径 | YAGNI；v1.9 已有基线 |

---

## 4. 架构

### 4.1 现有基线（保留）

```
Tauri setup 线程
  ├─ FastEmbed 路径：DeferredEmbedder 占位
  │    └─ 后台 thread：FastEmbedder::try_new → fulfill / fail
  ├─ watcher + scheduler 先挂上（窗口可先 paint）
  └─ 后台 thread：initial_scan_with_progress

E2E / 非 FastEmbed：同步 build_embedder（不走 deferred）
```

实现位置：`crates/embedder/src/deferred.rs`、`src-tauri/src/state.rs`、`src-tauri/src/lib.rs` setup。

### 4.2 本里程碑组件

| 单元 | 职责 | 依赖 |
|------|------|------|
| `DeferredEmbedder` | 已有 Pending/Ready/Failed + wait；补齐可测契约（超时/失败）；必要时允许测试注入更短 wait | `Embedder` trait |
| Embedder readiness 观测 | 将 Pending / Ready / Failed（+ 可选 message）暴露给壳层 | `AppState` 持有可查询状态（deferred 句柄或等价原子状态） |
| IPC / status | 新命令或扩展现有 index/status，供前端轮询或按需读取 | Tauri commands |
| UI 表面 | Settings（首选）显示「模型加载中 / 就绪 / 失败」；Failed 给可操作提示（检查模型/缓存或改 provider） | readiness IPC；`data-testid` |
| Release 闸门工件 | 版本 bump、changelog、`tauri build`、smoke 清单 | 仅在 §6 第 1 步通过后执行 |

### 4.3 数据流

**成功：**

```
try_new Ok → fulfill(inner)
  → readiness = Ready
  → embed/index 走真实 FastEmbed
```

**失败：**

```
try_new Err → fail(msg)
  → readiness = Failed(msg)
  → embed() 立即 EmbedError（含原因）
  → UI 显示 Failed + msg
```

**进行中：**

```
Pending → UI「加载中」
  → embed/index 可阻塞等待就绪（保留现有 wait），但不得表现为「死窗口」
```

### 4.4 已知边界（可记为后续债）

`AppState::reload_providers`（Settings 保存配置）当前走同步 `build_embedder`，与冷启动 deferred 路径不一致。本里程碑要求：**不因改动破坏 release 冷启动**；是否把 reload 也改为 deferred 标为可选后续，不阻塞 1.12.0。

---

## 5. 错误处理

| 场景 | 期望 |
|------|------|
| FastEmbed 后台 init 失败 | `fail(msg)`；后续 embed 立即失败；UI Failed + 消息 |
| Init 超时（生产默认约 300s wait） | 返回超时错误；UI Failed；测试可用更短超时 |
| Init 进行中用户触发索引/问答 | 可等待就绪；UI 标明加载中 |
| `initial_scan` 失败 | 保持现有：日志 + 不拖垮窗口；不强制新 UI，除非与 embedder Failed 必须合并提示 |
| E2E / Mock | 不走 Deferred；行为不变 |

**不做：** 静默自动切换 embedder provider。

---

## 6. 测试与发布门禁

### 6.1 测试金字塔

| 层 | 内容 |
|----|------|
| Unit | `DeferredEmbedder`：fulfill / fail / timeout（可注入短 wait） |
| 壳层 | readiness 可读；失败后 index/ask 返回可解析错误 |
| 前端 | readiness UI 的 Vitest（若新增可见状态）；稳定 `data-testid` |
| E2E | Mock 路径既有旅程绿；**不**在 CI 下载真实 FastEmbed 模型 |
| Release smoke | 本机 **release** 冷启动：窗口可交互；可选坏缓存触发 Failed 可见 |

### 6.2 门禁顺序（固定）

1. 可靠性自动化绿 + 本机 release 冷启动 smoke  
2. bump `package.json` + `tauri.conf.json` → **1.12.0**  
3. 写 changelog（摘要 1.9–1.11 + 本里程碑启动硬化）  
4. `tauri build` → Windows 安装包 → 本机安装/打开 smoke  

未过步骤 1 → **禁止** 将发布物标为 1.12.0。

---

## 7. 建议实现相位（供 writing-plans / GSD 拆分）

| 相位 | 内容 |
|------|------|
| P1 | DeferredEmbedder 契约测试 + 必要 API（状态查询 / 可测超时） |
| P2 | AppState + IPC readiness；失败路径联通 |
| P3 | Settings（或等价）UI + Vitest；可选轻量 E2E |
| P4 | Release smoke 清单执行 → version bump → changelog → `tauri build` |

具体计划在实现计划文档中展开；本文档不绑定精确 GSD phase 编号。

---

## 8. 与 Core Value 的关系

本里程碑不改变检索或引用语义，只保证：**用户拿到的正式包能启动、能感知模型就绪/失败，从而可信地使用已有 RAG/Agent 能力。** 信任链条从「功能存在」延伸到「可安装、可启动」。

---

## 9. 修订记录

| 日期 | 变更 |
|------|------|
| 2026-07-29 | 初稿：brainstorming 确认（发布向 / 1.12.0 / Approach A / 本机包） |
