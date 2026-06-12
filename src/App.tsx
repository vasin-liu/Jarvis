import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import {
  IconAlertCircle,
  IconBooks,
  IconCircleCheck,
  IconClock,
  IconFile,
  IconFileText,
  IconMail,
  IconMessageChatbot,
  IconMessages,
  IconSettings,
  IconTable,
} from "@tabler/icons-react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { parseCitations } from "./lib/citations";
import {
  formatIndexedAt,
  sourceKindLabel,
  statusLabel,
  statusTone,
} from "./lib/sourceDisplay";

type View = "chat" | "library" | "settings";

interface ChatSession {
  id: string;
  title: string;
  created_at: number;
  updated_at: number;
}

interface ChatMessage {
  id: number;
  session_id: string;
  role: "user" | "assistant" | "system";
  content: string;
  citations_json?: string | null;
  created_at: number;
}

interface Source {
  id: string;
  kind: string;
  uri: string;
  title: string;
  status: string;
  indexed_at?: number | null;
  error?: string | null;
}

interface IndexProgressView {
  phase: string;
  current: number;
  total: number;
  sourceTitle: string;
  outcome?: string | null;
  message?: string | null;
}

interface AppConfig {
  watch_folders: string[];
  embedder: "mock" | "ollama" | "fast_embed" | "cloud";
  chat: "mock" | "ollama" | "cloud";
  mock_embed_dim: number;
  ollama_base_url: string;
  ollama_embed_model: string;
  ollama_chat_model: string;
  ollama_embed_dim: number;
  lark_cli_bin: string;
  fastembed_model: string;
  fastembed_dim: number;
  cloud_base_url: string;
  cloud_api_key: string;
  cloud_embed_model: string;
  cloud_chat_model: string;
  cloud_embed_dim: number;
}

interface IndexStatusView {
  store_dim: number;
  config_embed_dim: number;
  stored_embedder_id: string | null;
  config_embedder_id: string;
  needs_rebuild: boolean;
  dim_mismatch: boolean;
  source_count: number;
  chunk_count: number;
}

interface RebuildReport {
  indexed: number;
  failed: number;
  skipped: number;
}

function SourceKindIcon({ kind }: { kind: string }) {
  const cls = "size-4 shrink-0 text-cyan-300/80";
  switch (kind) {
    case "local_file":
      return <IconFile className={cls} aria-hidden />;
    case "lark_doc":
      return <IconFileText className={cls} aria-hidden />;
    case "lark_sheet":
      return <IconTable className={cls} aria-hidden />;
    case "lark_mail":
      return <IconMail className={cls} aria-hidden />;
    case "lark_msg":
      return <IconMessages className={cls} aria-hidden />;
    default:
      return <IconFile className={cls} aria-hidden />;
  }
}

function StatusBadge({ status }: { status: string }) {
  const tone = statusTone(status);
  const Icon =
    status === "indexed"
      ? IconCircleCheck
      : status === "failed"
        ? IconAlertCircle
        : IconClock;
  return (
    <span className={`status-badge status-badge-${tone}`}>
      <Icon className="size-3" aria-hidden />
      {statusLabel(status)}
    </span>
  );
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

async function openCitation(uri: string) {
  if (uri.startsWith("http://") || uri.startsWith("https://")) {
    await openUrl(uri);
    return;
  }
  if (uri.startsWith("lark://")) {
    await navigator.clipboard.writeText(uri);
    return;
  }
  await openPath(uri);
}

function App() {
  const [view, setView] = useState<View>("chat");
  const [busy, setBusy] = useState(false);
  const [err, setErrState] = useState<string | null>(null);
  const setErr = setErrState;

  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [question, setQuestion] = useState("");
  const [streamingDraft, setStreamingDraft] = useState("");
  const [indexProgress, setIndexProgress] = useState<IndexProgressView | null>(
    null,
  );
  const chatScrollRef = useRef<HTMLDivElement>(null);
  const reduceMotion = useReducedMotion();

  const [sources, setSources] = useState<Source[]>([]);
  const [indexStatus, setIndexStatus] = useState<IndexStatusView | null>(null);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [larkDocToken, setLarkDocToken] = useState("");
  const [larkSheetToken, setLarkSheetToken] = useState("");
  const [larkMailId, setLarkMailId] = useState("");
  const [larkChatId, setLarkChatId] = useState("");
  const [larkStatus, setLarkStatus] = useState<string | null>(null);
  const [newWatchFolder, setNewWatchFolder] = useState("");

  const refreshSessions = useCallback(async () => {
    const list = await invoke<ChatSession[]>("list_chat_sessions");
    setSessions(list);
    return list;
  }, []);

  const refreshMessages = useCallback(async (sessionId: string) => {
    const list = await invoke<ChatMessage[]>("list_chat_messages", {
      sessionId,
    });
    setMessages(list);
  }, []);

  const refreshLibrary = useCallback(async () => {
    const list = await invoke<Source[]>("list_sources");
    setSources(list);
  }, []);

  const refreshConfig = useCallback(async () => {
    const cfg = await invoke<AppConfig>("get_config");
    setConfig(cfg);
  }, []);

  const refreshIndexStatus = useCallback(async () => {
    const status = await invoke<IndexStatusView>("get_index_status");
    setIndexStatus(status);
  }, []);

  const ensureSession = useCallback(async () => {
    let list = await refreshSessions();
    if (list.length === 0) {
      const created = await invoke<ChatSession>("create_chat_session", {
        title: null,
      });
      list = [created];
      setSessions(list);
    }
    const id = list[0].id;
    setActiveSessionId(id);
    await refreshMessages(id);
    return id;
  }, [refreshMessages, refreshSessions]);

  useEffect(() => {
    Promise.all([
      refreshLibrary(),
      refreshConfig(),
      refreshIndexStatus(),
      ensureSession(),
    ]).catch((e) => setErr(String(e)));
  }, [ensureSession, refreshConfig, refreshIndexStatus, refreshLibrary]);

  useEffect(() => {
    if (activeSessionId) {
      refreshMessages(activeSessionId).catch((e) => setErr(String(e)));
    }
  }, [activeSessionId, refreshMessages]);

  useLayoutEffect(() => {
    const el = chatScrollRef.current;
    if (el) {
      el.scrollTop = el.scrollHeight;
    }
  }, [messages, streamingDraft]);

  useEffect(() => {
    const unsubs: Array<() => void> = [];
    void listen<IndexProgressView>("index-progress", (event) => {
      setIndexProgress(event.payload);
    }).then((unlisten) => unsubs.push(unlisten));
    void listen<RebuildReport>("index-complete", () => {
      setIndexProgress(null);
      void refreshLibrary();
      void refreshIndexStatus();
    }).then((unlisten) => unsubs.push(unlisten));
    return () => {
      for (const unlisten of unsubs) unlisten();
    };
  }, [refreshIndexStatus, refreshLibrary]);

  async function handleNewSession() {
    setErr(null);
    setBusy(true);
    try {
      const s = await invoke<ChatSession>("create_chat_session", {
        title: null,
      });
      await refreshSessions();
      setActiveSessionId(s.id);
      setMessages([]);
      setQuestion("");
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleDeleteSession(id: string) {
    setErr(null);
    setBusy(true);
    try {
      await invoke("delete_chat_session", { id });
      const list = await refreshSessions();
      if (activeSessionId === id) {
        if (list.length > 0) {
          setActiveSessionId(list[0].id);
        } else {
          const s = await invoke<ChatSession>("create_chat_session", {
            title: null,
          });
          setSessions([s]);
          setActiveSessionId(s.id);
          setMessages([]);
        }
      }
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleAsk() {
    const q = question.trim();
    if (!q || !activeSessionId) return;
    setErr(null);
    setBusy(true);
    setStreamingDraft("");
    try {
      const channel = new Channel<{ token: string }>();
      channel.onmessage = (msg) => {
        setStreamingDraft((prev) => prev + msg.token);
      };
      await invoke("ask_in_session_stream", {
        sessionId: activeSessionId,
        question: q,
        onToken: channel,
      });
      setQuestion("");
      setStreamingDraft("");
      await refreshMessages(activeSessionId);
      await refreshSessions();
      await refreshIndexStatus();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
      setStreamingDraft("");
    }
  }

  async function handlePickAndIndex() {
    setErr(null);
    setBusy(true);
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Documents", extensions: ["txt", "md", "markdown"] }],
      });
      if (selected === null) return;
      await invoke<string>("index_file", { path: selected });
      await refreshLibrary();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleRetrySource(id: string) {
    setErr(null);
    setBusy(true);
    try {
      await invoke<RebuildReport>("retry_source", { id });
      await refreshLibrary();
      await refreshIndexStatus();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleRemoveSource(id: string) {
    setErr(null);
    setBusy(true);
    try {
      await invoke("remove_source", { id });
      await refreshLibrary();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleSaveConfig(next: AppConfig) {
    setErr(null);
    setBusy(true);
    try {
      await invoke("set_config", { config: next });
      setConfig(next);
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleCheckLark() {
    setErr(null);
    setBusy(true);
    try {
      const who = await invoke<string>("check_lark_connection");
      setLarkStatus(who);
    } catch (e) {
      setLarkStatus(null);
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function syncLark(
    command: string,
    payload: Record<string, string>,
  ) {
    setErr(null);
    setBusy(true);
    try {
      await invoke<string>(command, payload);
      await refreshLibrary();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleAddWatchFolder() {
    const path = newWatchFolder.trim();
    if (!path) return;
    setErr(null);
    setBusy(true);
    try {
      await invoke("add_watch_folder", { path });
      setNewWatchFolder("");
      await refreshConfig();
      await refreshLibrary();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handlePickWatchFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (selected === null || typeof selected !== "string") return;
    setNewWatchFolder(selected);
  }

  async function handleRemoveWatchFolder(path: string) {
    setErr(null);
    setBusy(true);
    try {
      await invoke("remove_watch_folder", { path });
      await refreshConfig();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleRebuildIndex() {
    setErr(null);
    setBusy(true);
    try {
      const report = await invoke<RebuildReport>("rebuild_index");
      await refreshLibrary();
      await refreshIndexStatus();
      setLarkStatus(
        `重建完成：成功 ${report.indexed}，失败 ${report.failed}，跳过 ${report.skipped}`,
      );
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleReinitAndRebuild() {
    if (
      !window.confirm(
        "将清除所有向量索引并按当前 Embedder 配置重新嵌入，是否继续？",
      )
    ) {
      return;
    }
    setErr(null);
    setBusy(true);
    try {
      const report = await invoke<RebuildReport>("reinit_and_rebuild_index");
      await refreshLibrary();
      await refreshIndexStatus();
      setLarkStatus(
        `重置并重建完成：成功 ${report.indexed}，失败 ${report.failed}`,
      );
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex min-h-screen">
      <aside className="glass-panel m-4 flex w-56 shrink-0 flex-col gap-2 p-4">
        <div className="mb-2 px-2">
          <div className="text-xs uppercase tracking-widest text-cyan-300/80">
            Jarvis
          </div>
          <h1 className="text-lg font-semibold text-zinc-50">知识中枢</h1>
        </div>
        {(
          [
            ["chat", "对话", IconMessageChatbot],
            ["library", "资料库", IconBooks],
            ["settings", "设置", IconSettings],
          ] as const
        ).map(([id, label, Icon]) => (
          <button
            key={id}
            type="button"
            data-testid={`nav-${id}`}
            className={`nav-btn flex items-center gap-2 ${view === id ? "nav-btn-active" : "nav-btn-idle"}`}
            onClick={() => setView(id)}
          >
            <Icon className="size-4 shrink-0 opacity-80" aria-hidden />
            {label}
          </button>
        ))}
        <div className="mt-auto px-2 pt-4 text-xs text-zinc-500">
          来源 {sources.length} · M9
        </div>
      </aside>

      <main className="flex flex-1 flex-col gap-4 p-4 pl-0">
        {err && (
          <div className="glass-panel border-red-400/30 bg-red-950/40 px-4 py-3 text-sm text-red-200">
            {err}
          </div>
        )}

        {indexProgress && <IndexProgressBanner progress={indexProgress} />}

        <AnimatePresence mode="wait">
          {view === "chat" && (
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
                onClick={handleNewSession}
              >
                新对话
              </button>
              <ul className="flex-1 space-y-1 overflow-auto">
                {sessions.map((s) => (
                  <li key={s.id} className="group flex items-center gap-1">
                    <button
                      type="button"
                      className={`nav-btn flex-1 truncate ${activeSessionId === s.id ? "nav-btn-active" : "nav-btn-idle"}`}
                      onClick={() => setActiveSessionId(s.id)}
                    >
                      {s.title}
                    </button>
                    <button
                      type="button"
                      className="btn-ghost hidden px-2 py-1 text-xs group-hover:inline"
                      onClick={() => handleDeleteSession(s.id)}
                    >
                      ×
                    </button>
                  </li>
                ))}
              </ul>
            </div>

            <div className="flex flex-1 flex-col gap-4 p-5">
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
                    className={`rounded-xl border px-4 py-3 text-sm ${
                      m.role === "user"
                        ? "ml-8 border-cyan-400/20 bg-cyan-950/30"
                        : "mr-8 border-white/10 bg-zinc-950/40"
                    }`}
                  >
                    <div className="mb-1 text-xs uppercase text-zinc-500">
                      {m.role === "user" ? "你" : "助理"}
                    </div>
                    <p className="whitespace-pre-wrap text-zinc-100">
                      {m.content}
                    </p>
                    {m.role === "assistant" &&
                      parseCitations(m.citations_json).length > 0 && (
                        <ul className="mt-3 space-y-2 border-t border-white/10 pt-3">
                          {parseCitations(m.citations_json).map((c) => (
                            <li key={c.chunk_id}>
                              <button
                                type="button"
                                className="text-left text-xs text-cyan-300 hover:underline"
                                onClick={() =>
                                  openCitation(c.source_uri).catch((e) =>
                                    setErr(String(e)),
                                  )
                                }
                              >
                                {c.source_title} · {c.loc}
                              </button>
                              <div className="mt-1 text-zinc-400">
                                {c.excerpt}
                              </div>
                            </li>
                          ))}
                        </ul>
                      )}
                  </div>
                ))}
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
                <textarea
                  className="field min-h-24 resize-y"
                  value={question}
                  onChange={(e) => setQuestion(e.target.value)}
                  placeholder="输入问题…"
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
                      void handleAsk();
                    }
                  }}
                />
                <button
                  type="button"
                  data-testid="ask-submit"
                  className="btn-primary mt-2"
                  disabled={busy || !question.trim() || !activeSessionId}
                  onClick={handleAsk}
                >
                  {busy ? "处理中…" : "发送 (Ctrl+Enter)"}
                </button>
              </div>
            </div>
            </motion.section>
          )}

          {view === "library" && (
            <motion.section
              key="library"
              className="glass-panel flex flex-1 flex-col gap-4 p-5"
              initial={reduceMotion ? false : { opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={reduceMotion ? undefined : { opacity: 0, y: -6 }}
              transition={{ duration: 0.18 }}
            >
            <div
              data-testid="library-stats"
              className="rounded-xl border border-white/10 bg-zinc-950/40 px-4 py-3 text-sm text-zinc-300"
            >
              <span className="text-zinc-400">索引概览：</span>
              已索引{" "}
              {sources.filter((s) => s.status === "indexed").length} · 失败{" "}
              {sources.filter((s) => s.status === "failed").length} · 待处理{" "}
              {sources.filter((s) => s.status === "pending").length} · 共{" "}
              {sources.length} 个来源
            </div>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <h2 className="text-base font-medium">资料库</h2>
              <button
                type="button"
                className="btn-ghost"
                disabled={busy}
                onClick={handlePickAndIndex}
              >
                选择文件索引
              </button>
            </div>

            <div className="grid gap-3 md:grid-cols-2">
              <div className="flex gap-2">
                <input
                  className="field flex-1"
                  value={larkDocToken}
                  onChange={(e) => setLarkDocToken(e.target.value)}
                  placeholder="飞书文档 token"
                />
                <button
                  type="button"
                  className="btn-primary shrink-0"
                  disabled={busy || !larkDocToken.trim()}
                  onClick={() =>
                    syncLark("sync_lark_doc", { token: larkDocToken.trim() })
                  }
                >
                  同步文档
                </button>
              </div>
              <div className="flex gap-2">
                <input
                  className="field flex-1"
                  value={larkSheetToken}
                  onChange={(e) => setLarkSheetToken(e.target.value)}
                  placeholder="电子表格 token"
                />
                <button
                  type="button"
                  className="btn-primary shrink-0"
                  disabled={busy || !larkSheetToken.trim()}
                  onClick={() =>
                    syncLark("sync_lark_sheet", {
                      token: larkSheetToken.trim(),
                    })
                  }
                >
                  同步表格
                </button>
              </div>
              <div className="flex gap-2">
                <input
                  className="field flex-1"
                  value={larkMailId}
                  onChange={(e) => setLarkMailId(e.target.value)}
                  placeholder="邮件 message id"
                />
                <button
                  type="button"
                  className="btn-primary shrink-0"
                  disabled={busy || !larkMailId.trim()}
                  onClick={() =>
                    syncLark("sync_lark_mail", {
                      messageId: larkMailId.trim(),
                    })
                  }
                >
                  同步邮件
                </button>
              </div>
              <div className="flex gap-2">
                <input
                  className="field flex-1"
                  value={larkChatId}
                  onChange={(e) => setLarkChatId(e.target.value)}
                  placeholder="IM chat id"
                />
                <button
                  type="button"
                  className="btn-primary shrink-0"
                  disabled={busy || !larkChatId.trim()}
                  onClick={() =>
                    syncLark("sync_lark_im", { chatId: larkChatId.trim() })
                  }
                >
                  同步会话
                </button>
              </div>
            </div>

            {sources.length === 0 ? (
              <p className="text-sm text-zinc-500">暂无来源。</p>
            ) : (
              <ul className="divide-y divide-white/10 overflow-auto rounded-xl border border-white/10">
                {sources.map((s) => (
                  <li
                    key={s.id}
                    className="flex items-start justify-between gap-4 px-4 py-3 text-sm"
                  >
                    <div className="flex min-w-0 gap-3">
                      <SourceKindIcon kind={s.kind} />
                      <div className="min-w-0">
                        <div className="flex flex-wrap items-center gap-2">
                          <div className="truncate font-medium text-zinc-100">
                            {s.title}
                          </div>
                          <StatusBadge status={s.status} />
                        </div>
                        <div className="mt-1 truncate text-xs text-zinc-500">
                          {sourceKindLabel(s.kind)} · 更新{" "}
                          {formatIndexedAt(s.indexed_at)} · {s.uri}
                        </div>
                        {s.error && (
                          <div className="mt-1 text-xs text-red-300">
                            {s.error}
                          </div>
                        )}
                      </div>
                    </div>
                    <div className="flex shrink-0 flex-col gap-2">
                      <button
                        type="button"
                        className="btn-ghost text-xs"
                        disabled={busy}
                        data-testid={`retry-source-${s.id}`}
                        onClick={() => handleRetrySource(s.id)}
                      >
                        {s.status === "failed" ? "重试" : "重新同步"}
                      </button>
                      <button
                        type="button"
                        className="btn-ghost text-xs"
                        disabled={busy}
                        onClick={() => handleRemoveSource(s.id)}
                      >
                        移除
                      </button>
                    </div>
                  </li>
                ))}
              </ul>
            )}
            </motion.section>
          )}

          {view === "settings" && config && (
            <motion.section
              key="settings"
              data-testid="settings-panel"
              className="glass-panel flex flex-col gap-5 p-5"
              initial={reduceMotion ? false : { opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={reduceMotion ? undefined : { opacity: 0, y: -6 }}
              transition={{ duration: 0.18 }}
            >
            <h2 className="text-base font-medium">设置</h2>

            {indexStatus && (
              <div
                data-testid="index-status"
                className={`rounded-xl border p-4 text-sm ${
                  indexStatus.needs_rebuild
                    ? "border-amber-400/30 bg-amber-950/30 text-amber-100"
                    : "border-white/10 bg-zinc-950/40 text-zinc-300"
                }`}
              >
                <div className="mb-2 font-medium">索引状态</div>
                <div className="space-y-1 text-xs text-zinc-400">
                  <div>
                    向量维度：库 {indexStatus.store_dim} / 配置{" "}
                    {indexStatus.config_embed_dim}
                    {indexStatus.dim_mismatch && " · 维度不一致"}
                  </div>
                  <div>
                    Embedder：{indexStatus.stored_embedder_id ?? "未记录"} →{" "}
                    {indexStatus.config_embedder_id}
                  </div>
                  <div>
                    来源 {indexStatus.source_count} · 分块{" "}
                    {indexStatus.chunk_count}
                  </div>
                </div>
                <div className="mt-3 flex flex-wrap gap-2">
                  <button
                    type="button"
                    data-testid="rebuild-index"
                    className="btn-ghost text-xs"
                    disabled={busy || indexStatus.source_count === 0}
                    onClick={handleRebuildIndex}
                  >
                    重建索引
                  </button>
                  {indexStatus.dim_mismatch && (
                    <button
                      type="button"
                      data-testid="reinit-rebuild-index"
                      className="btn-primary text-xs"
                      disabled={busy}
                      onClick={handleReinitAndRebuild}
                    >
                      重置向量表并重建
                    </button>
                  )}
                </div>
              </div>
            )}

            <div className="rounded-xl border border-white/10 p-4">
              <div className="mb-2 text-sm text-zinc-300">飞书连接</div>
              <div className="flex flex-wrap items-center gap-3">
                <button
                  type="button"
                  className="btn-ghost"
                  disabled={busy}
                  onClick={handleCheckLark}
                >
                  检测 lark-cli
                </button>
                {larkStatus && (
                  <span className="text-sm text-emerald-300">已登录：{larkStatus}</span>
                )}
              </div>
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <label className="space-y-1 text-sm">
                <span className="text-zinc-400">Embedder</span>
                <select
                  className="field"
                  value={config.embedder}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      embedder: e.target.value as AppConfig["embedder"],
                    })
                  }
                >
                  <option value="mock">Mock</option>
                  <option value="ollama">Ollama</option>
                  <option value="fast_embed">FastEmbed（本地 ONNX）</option>
                  <option value="cloud">Cloud（OpenAI 兼容）</option>
                </select>
              </label>
              <label className="space-y-1 text-sm">
                <span className="text-zinc-400">Chat</span>
                <select
                  className="field"
                  value={config.chat}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      chat: e.target.value as AppConfig["chat"],
                    })
                  }
                >
                  <option value="mock">Mock</option>
                  <option value="ollama">Ollama</option>
                  <option value="cloud">Cloud（OpenAI 兼容）</option>
                </select>
              </label>
              {(config.embedder === "cloud" || config.chat === "cloud") && (
                <>
                  <label className="space-y-1 text-sm md:col-span-2">
                    <span className="text-zinc-400">Cloud Base URL</span>
                    <input
                      className="field"
                      value={config.cloud_base_url}
                      onChange={(e) =>
                        setConfig({
                          ...config,
                          cloud_base_url: e.target.value,
                        })
                      }
                      placeholder="https://api.openai.com/v1"
                    />
                  </label>
                  <label className="space-y-1 text-sm md:col-span-2">
                    <span className="text-zinc-400">Cloud API Key</span>
                    <input
                      className="field"
                      type="password"
                      autoComplete="off"
                      value={config.cloud_api_key}
                      onChange={(e) =>
                        setConfig({
                          ...config,
                          cloud_api_key: e.target.value,
                        })
                      }
                      placeholder="sk-…"
                    />
                  </label>
                </>
              )}
              {config.embedder === "cloud" && (
                <>
                  <label className="space-y-1 text-sm">
                    <span className="text-zinc-400">Embed Model</span>
                    <input
                      className="field"
                      value={config.cloud_embed_model}
                      onChange={(e) =>
                        setConfig({
                          ...config,
                          cloud_embed_model: e.target.value,
                        })
                      }
                    />
                  </label>
                  <label className="space-y-1 text-sm">
                    <span className="text-zinc-400">Embed Dim</span>
                    <input
                      className="field"
                      type="number"
                      min={1}
                      value={config.cloud_embed_dim}
                      onChange={(e) =>
                        setConfig({
                          ...config,
                          cloud_embed_dim: Number(e.target.value) || 1536,
                        })
                      }
                    />
                  </label>
                </>
              )}
              {config.chat === "cloud" && (
                <label className="space-y-1 text-sm md:col-span-2">
                  <span className="text-zinc-400">Chat Model</span>
                  <input
                    className="field"
                    value={config.cloud_chat_model}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        cloud_chat_model: e.target.value,
                      })
                    }
                  />
                </label>
              )}
              {config.embedder === "fast_embed" && (
                <>
                  <label className="space-y-1 text-sm md:col-span-2">
                    <span className="text-zinc-400">FastEmbed 模型</span>
                    <select
                      className="field"
                      value={config.fastembed_model}
                      onChange={(e) =>
                        setConfig({
                          ...config,
                          fastembed_model: e.target.value,
                        })
                      }
                    >
                      <option value="bge-small-zh-v1.5">
                        bge-small-zh-v1.5（512 维，中文）
                      </option>
                      <option value="bge-small-en-v1.5">
                        bge-small-en-v1.5（384 维）
                      </option>
                      <option value="bge-base-en-v1.5">
                        bge-base-en-v1.5（768 维）
                      </option>
                      <option value="multilingual-e5-small">
                        multilingual-e5-small（384 维）
                      </option>
                    </select>
                  </label>
                  <p className="text-xs text-zinc-500 md:col-span-2">
                    首次使用会从网络下载 ONNX 模型；切换模型后请在上方「重置向量表并重建」。
                  </p>
                </>
              )}
              <label className="space-y-1 text-sm md:col-span-2">
                <span className="text-zinc-400">Ollama Base URL</span>
                <input
                  className="field"
                  value={config.ollama_base_url}
                  onChange={(e) =>
                    setConfig({ ...config, ollama_base_url: e.target.value })
                  }
                />
              </label>
              <label className="space-y-1 text-sm">
                <span className="text-zinc-400">Embed Model</span>
                <input
                  className="field"
                  value={config.ollama_embed_model}
                  onChange={(e) =>
                    setConfig({ ...config, ollama_embed_model: e.target.value })
                  }
                />
              </label>
              <label className="space-y-1 text-sm">
                <span className="text-zinc-400">Chat Model</span>
                <input
                  className="field"
                  value={config.ollama_chat_model}
                  onChange={(e) =>
                    setConfig({ ...config, ollama_chat_model: e.target.value })
                  }
                />
              </label>
              <label className="space-y-1 text-sm">
                <span className="text-zinc-400">lark-cli 路径</span>
                <input
                  className="field"
                  value={config.lark_cli_bin}
                  onChange={(e) =>
                    setConfig({ ...config, lark_cli_bin: e.target.value })
                  }
                />
              </label>
            </div>

            <button
              type="button"
              className="btn-primary w-fit"
              disabled={busy}
              onClick={() => {
                void handleSaveConfig(config).then(() => refreshIndexStatus());
              }}
            >
              保存配置
            </button>

            <div className="border-t border-white/10 pt-4">
              <h3 className="mb-3 text-sm font-medium text-zinc-200">
                监听文件夹
              </h3>
              <div className="mb-3 flex flex-wrap gap-2">
                <input
                  className="field max-w-xl flex-1"
                  value={newWatchFolder}
                  onChange={(e) => setNewWatchFolder(e.target.value)}
                  placeholder="文件夹路径"
                />
                <button
                  type="button"
                  className="btn-ghost"
                  onClick={handlePickWatchFolder}
                >
                  浏览
                </button>
                <button
                  type="button"
                  className="btn-primary"
                  disabled={busy || !newWatchFolder.trim()}
                  onClick={handleAddWatchFolder}
                >
                  添加并扫描
                </button>
              </div>
              <ul className="space-y-2">
                {config.watch_folders.map((folder) => (
                  <li
                    key={folder}
                    className="flex items-center justify-between rounded-xl border border-white/10 px-3 py-2 text-sm"
                  >
                    <span className="truncate text-zinc-300">{folder}</span>
                    <button
                      type="button"
                      className="btn-ghost text-xs"
                      disabled={busy}
                      onClick={() => handleRemoveWatchFolder(folder)}
                    >
                      移除
                    </button>
                  </li>
                ))}
                {config.watch_folders.length === 0 && (
                  <li className="text-sm text-zinc-500">未配置监听文件夹</li>
                )}
              </ul>
            </div>
            </motion.section>
          )}
        </AnimatePresence>
      </main>
    </div>
  );
}

export default App;
