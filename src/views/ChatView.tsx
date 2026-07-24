import { useLayoutEffect, useRef } from "react";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { motion, useReducedMotion } from "motion/react";

import { parseCitations } from "../lib/citations";
import type { ChatMessage, ChatSession } from "../types/chat";
import type {
  AppConfig,
  IndexProgressView,
  OrchestrationStepInfo,
  ToolCallInfo,
} from "../types/ipc";

async function openCitation(uri: string) {
  if (uri.startsWith("http://") || uri.startsWith("https://")) {
    await openUrl(uri);
    return;
  }
  if (uri.startsWith("lark://") || uri.startsWith("cursor://")) {
    await navigator.clipboard.writeText(uri);
    return;
  }
  await openPath(uri);
}

function IndexProgressBanner({ progress }: { progress: IndexProgressView }) {
  const pct =
    progress.total > 0
      ? Math.round((progress.current / progress.total) * 100)
      : 0;
  return (
    <div
      data-testid="index-progress"
      className="glass-panel border-cyan-400/20 px-4 py-3 text-sm text-zinc-200"
    >
      <div className="mb-2 flex flex-wrap items-center justify-between gap-2 text-xs text-zinc-400">
        <span>
          索引中 · {progress.phase} · {progress.sourceTitle}
          {progress.outcome ? ` · ${progress.outcome}` : ""}
        </span>
        <span>
          {progress.current}/{progress.total}
        </span>
      </div>
      <div className="index-progress-track">
        <div className="index-progress-fill" style={{ width: `${pct}%` }} />
      </div>
    </div>
  );
}

export interface ChatViewProps {
  sessions: ChatSession[];
  activeSessionId: string | null;
  messages: ChatMessage[];
  question: string;
  streamingDraft: string;
  onSetQuestion: (value: string) => void;
  onCreateSession: () => void;
  onSwitchSession: (sessionId: string) => void;
  onDeleteSession: (sessionId: string) => void;
  agentMode: boolean;
  setAgentMode: (enabled: boolean) => void;
  lastToolCalls: ToolCallInfo[];
  lastToolParseWarnings?: string[];
  lastOrchestrationSteps: OrchestrationStepInfo[];
  onRagAsk: () => void;
  onAgentAsk: () => void;
  onManageAgent: () => void;
  onSetActiveAgent: (agentId: string) => void;
  busy: boolean;
  err: string | null;
  setErr: (err: string | null) => void;
  indexProgress: IndexProgressView | null;
  e2eMode: boolean;
  config: AppConfig | null;
}

export function ChatView({
  sessions,
  activeSessionId,
  messages,
  question,
  streamingDraft,
  onSetQuestion,
  onCreateSession,
  onSwitchSession,
  onDeleteSession,
  agentMode,
  setAgentMode,
  lastToolCalls,
  lastToolParseWarnings = [],
  lastOrchestrationSteps,
  onRagAsk,
  onAgentAsk,
  onManageAgent,
  onSetActiveAgent,
  busy,
  err,
  setErr,
  indexProgress,
  e2eMode,
  config,
}: ChatViewProps) {
  const chatScrollRef = useRef<HTMLDivElement>(null);
  const reduceMotion = useReducedMotion();

  useLayoutEffect(() => {
    const el = chatScrollRef.current;
    if (el) {
      el.scrollTop = el.scrollHeight;
    }
  }, [messages, streamingDraft]);

  return (
    <motion.section
      key="chat"
      className="glass-panel flex min-h-[70vh] flex-1 overflow-hidden"
      initial={reduceMotion ? false : { opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={reduceMotion ? undefined : { opacity: 0, y: -6 }}
      transition={{ duration: 0.18 }}
    >
      <div className="flex w-52 shrink-0 flex-col border-r border-white/10 p-3">
        <button
          type="button"
          className="btn-primary mb-3 text-xs"
          disabled={busy}
          onClick={() => void onCreateSession()}
        >
          新对话
        </button>
        <ul className="flex-1 space-y-1 overflow-auto">
          {sessions.map((s) => (
            <li key={s.id} className="group flex items-center gap-1">
              <button
                type="button"
                className={`nav-btn flex-1 truncate ${activeSessionId === s.id ? "nav-btn-active" : "nav-btn-idle"}`}
                onClick={() => onSwitchSession(s.id)}
              >
                {s.title}
              </button>
              <button
                type="button"
                className="btn-ghost hidden px-2 py-1 text-xs group-hover:inline"
                onClick={() => void onDeleteSession(s.id)}
              >
                ×
              </button>
            </li>
          ))}
        </ul>
      </div>

      <div className="flex flex-1 flex-col gap-4 p-5">
        {err && (
          <div
            data-testid="chat-error"
            className="glass-panel border-red-400/30 bg-red-950/40 px-4 py-3 text-sm text-red-200"
          >
            {err}
          </div>
        )}

        {indexProgress && <IndexProgressBanner progress={indexProgress} />}

        <div
          ref={chatScrollRef}
          className="flex-1 space-y-4 overflow-auto"
        >
          {messages.length === 0 && (
            <p className="text-sm text-zinc-500">
              开始提问，答案会保存在当前会话中。
            </p>
          )}
          {messages.map((m) => (
            <div
              key={m.id}
              data-testid={
                m.role === "assistant"
                  ? "chat-message-assistant"
                  : m.role === "user"
                    ? "chat-message-user"
                    : undefined
              }
              className={`rounded-xl border px-4 py-3 text-sm ${
                m.role === "user"
                  ? "ml-8 border-cyan-400/20 bg-cyan-950/30"
                  : "mr-8 border-white/10 bg-zinc-950/40"
              }`}
            >
              <div className="mb-1 text-xs uppercase text-zinc-500">
                {m.role === "user" ? "你" : "助理"}
              </div>
              <p className="whitespace-pre-wrap text-zinc-100">{m.content}</p>
              {m.role === "assistant" &&
                parseCitations(m.citations_json).length > 0 && (
                  <ul className="mt-3 space-y-2 border-t border-white/10 pt-3">
                    {parseCitations(m.citations_json).map((c) => (
                      <li key={c.chunk_id}>
                        <button
                          type="button"
                          className="text-left text-xs text-cyan-300 hover:underline"
                          data-source-uri={c.source_uri}
                          onClick={() =>
                            openCitation(c.source_uri).catch((e) =>
                              setErr(String(e)),
                            )
                          }
                        >
                          {c.source_title} · {c.loc}
                        </button>
                        <div
                          data-testid="citation-excerpt"
                          className="mt-1 text-zinc-400"
                        >
                          {c.excerpt}
                        </div>
                      </li>
                    ))}
                  </ul>
                )}
            </div>
          ))}
          {!busy && agentMode && lastOrchestrationSteps.length > 0 && (
            <div
              data-testid="orchestration-steps"
              className="mr-8 rounded-xl border border-violet-400/20 bg-violet-950/20 px-4 py-3 text-sm"
            >
              <div className="mb-2 text-xs uppercase text-violet-200/80">
                多 Agent 编排 ({lastOrchestrationSteps.length} 步)
              </div>
              <ul className="space-y-3">
                {lastOrchestrationSteps.map((step, i) => (
                  <li key={`${step.agent_id}-${i}`} className="text-xs">
                    <span className="font-medium text-violet-200">
                      {i + 1}. {step.agent_name}
                    </span>
                    <p className="mt-1 whitespace-pre-wrap text-zinc-400">
                      {step.answer.length > 300
                        ? `${step.answer.slice(0, 300)}…`
                        : step.answer}
                    </p>
                  </li>
                ))}
              </ul>
            </div>
          )}
          {!busy && agentMode && lastToolParseWarnings.length > 0 && (
            <div
              data-testid="agent-tool-parse-warnings"
              className="mr-8 rounded-xl border border-amber-400/25 bg-amber-950/25 px-4 py-3 text-sm backdrop-blur-sm"
            >
              <div className="mb-2 text-xs uppercase tracking-wide text-amber-200/90">
                工具解析警告
              </div>
              <ul className="space-y-1.5">
                {lastToolParseWarnings.map((warning, i) => (
                  <li key={`warn-${i}`} className="text-xs text-amber-100/90">
                    {warning}
                  </li>
                ))}
              </ul>
            </div>
          )}
          {!busy && agentMode && lastToolCalls.length > 0 && (
            <div
              data-testid="agent-tool-calls"
              className="mr-8 rounded-xl border border-amber-400/20 bg-amber-950/20 px-4 py-3 text-sm"
            >
              <div className="mb-2 text-xs uppercase text-amber-200/80">
                工具调用
              </div>
              <ul className="space-y-2">
                {lastToolCalls.map((tc, i) => (
                  <li key={`${tc.name}-${i}`} className="text-xs">
                    <span className="font-mono text-amber-200">{tc.name}</span>
                    <p className="mt-1 whitespace-pre-wrap text-zinc-400">
                      {tc.result.length > 400
                        ? `${tc.result.slice(0, 400)}…`
                        : tc.result}
                    </p>
                  </li>
                ))}
              </ul>
            </div>
          )}
          {busy && <div data-testid="ask-busy" className="hidden" />}
          {busy && (
            <div
              data-testid="streaming-answer"
              className="mr-8 rounded-xl border border-white/10 bg-zinc-950/40 px-4 py-3 text-sm"
            >
              <div className="mb-1 text-xs uppercase text-zinc-500">
                助理 · 生成中
              </div>
              {streamingDraft ? (
                <p className="whitespace-pre-wrap text-zinc-100">
                  {streamingDraft}
                  <span className="stream-cursor" aria-hidden="true" />
                </p>
              ) : (
                <p className="text-zinc-400">
                  思考中…
                  <span className="stream-cursor" aria-hidden="true" />
                </p>
              )}
            </div>
          )}
        </div>

        <div className="border-t border-white/10 pt-4">
          <div className="mb-3 flex flex-wrap items-center gap-3 text-sm">
            <label className="flex items-center gap-2 text-zinc-300">
              <input
                type="checkbox"
                data-testid="agent-mode-toggle"
                checked={agentMode}
                onChange={(e) => setAgentMode(e.target.checked)}
              />
              Agent 模式（工具调用）
            </label>
            {agentMode && config && config.agent_orchestration_mode === "single" && (
              <select
                className="field max-w-xs text-sm"
                data-testid="agent-profile-select"
                value={config.active_agent_id}
                onChange={(e) => onSetActiveAgent(e.target.value)}
                disabled={busy}
              >
                {config.agents
                  .filter((a) => a.enabled)
                  .map((a) => (
                    <option key={a.id} value={a.id}>
                      {a.name}
                    </option>
                  ))}
              </select>
            )}
            {agentMode &&
              config &&
              config.agent_orchestration_mode === "pipeline" && (
                <span className="text-xs text-violet-300">流水线模式</span>
              )}
            {agentMode &&
              config &&
              config.agent_orchestration_mode === "router" && (
                <span className="text-xs text-emerald-300">路由模式</span>
              )}
            {e2eMode && agentMode && config && (
              <div
                data-testid={`orchestration-mode-${config.agent_orchestration_mode}`}
                className="hidden"
              />
            )}
            <button
              type="button"
              className="btn-ghost text-xs text-cyan-300"
              onClick={onManageAgent}
            >
              管理 Agent
            </button>
          </div>
          <textarea
            data-testid="chat-input"
            className="field min-h-24 resize-y"
            value={question}
            onChange={(e) => onSetQuestion(e.target.value)}
            placeholder="输入问题…"
            onKeyDown={(e) => {
              if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
                e.preventDefault();
                if (agentMode) {
                  void onAgentAsk();
                } else {
                  document
                    .querySelector<HTMLFormElement>(
                      '[data-testid="chat-form"]',
                    )
                    ?.requestSubmit();
                }
              }
            }}
          />
          <div className="mt-2 flex flex-wrap gap-2">
            <form
              data-testid="chat-form"
              onSubmit={(e) => {
                e.preventDefault();
                void onRagAsk();
              }}
            >
              <button
                type="submit"
                data-testid="ask-submit"
                className="btn-primary"
                disabled={busy || !question.trim() || !activeSessionId}
              >
                {busy ? "处理中…" : "发送 (Ctrl+Enter)"}
              </button>
            </form>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                void onAgentAsk();
              }}
            >
              <button
                type="submit"
                data-testid="agent-submit"
                className="btn-primary border-violet-400/30 bg-violet-950/50"
                disabled={busy || !question.trim() || !activeSessionId}
              >
                {busy ? "处理中…" : "发送 (Agent)"}
              </button>
            </form>
          </div>
        </div>
      </div>
    </motion.section>
  );
}
