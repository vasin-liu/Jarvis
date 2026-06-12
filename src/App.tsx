import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";

type View = "chat" | "library" | "settings";

interface Citation {
  chunk_id: number;
  source_id: string;
  source_title: string;
  source_uri: string;
  loc: string;
  excerpt: string;
}

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
  error?: string | null;
}

interface AppConfig {
  watch_folders: string[];
  embedder: "mock" | "ollama";
  chat: "mock" | "ollama";
  mock_embed_dim: number;
  ollama_base_url: string;
  ollama_embed_model: string;
  ollama_chat_model: string;
  ollama_embed_dim: number;
  lark_cli_bin: string;
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

function parseCitations(raw?: string | null): Citation[] {
  if (!raw) return [];
  try {
    return JSON.parse(raw) as Citation[];
  } catch {
    return [];
  }
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

  const [sources, setSources] = useState<Source[]>([]);
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
    Promise.all([refreshLibrary(), refreshConfig(), ensureSession()]).catch(
      (e) => setErr(String(e)),
    );
  }, [ensureSession, refreshConfig, refreshLibrary]);

  useEffect(() => {
    if (activeSessionId) {
      refreshMessages(activeSessionId).catch((e) => setErr(String(e)));
    }
  }, [activeSessionId, refreshMessages]);

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
    try {
      await invoke("ask_in_session", {
        sessionId: activeSessionId,
        question: q,
      });
      setQuestion("");
      await refreshMessages(activeSessionId);
      await refreshSessions();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
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
            ["chat", "对话"],
            ["library", "资料库"],
            ["settings", "设置"],
          ] as const
        ).map(([id, label]) => (
          <button
            key={id}
            type="button"
            className={`nav-btn ${view === id ? "nav-btn-active" : "nav-btn-idle"}`}
            onClick={() => setView(id)}
          >
            {label}
          </button>
        ))}
        <div className="mt-auto px-2 pt-4 text-xs text-zinc-500">
          来源 {sources.length} · M5
        </div>
      </aside>

      <main className="flex flex-1 flex-col gap-4 p-4 pl-0">
        {err && (
          <div className="glass-panel border-red-400/30 bg-red-950/40 px-4 py-3 text-sm text-red-200">
            {err}
          </div>
        )}

        {view === "chat" && (
          <section className="glass-panel flex min-h-[70vh] flex-1 overflow-hidden">
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
              <div className="flex-1 space-y-4 overflow-auto">
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
                  className="btn-primary mt-2"
                  disabled={busy || !question.trim() || !activeSessionId}
                  onClick={handleAsk}
                >
                  {busy ? "处理中…" : "发送 (Ctrl+Enter)"}
                </button>
              </div>
            </div>
          </section>
        )}

        {view === "library" && (
          <section className="glass-panel flex flex-1 flex-col gap-4 p-5">
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
                    <div className="min-w-0">
                      <div className="truncate font-medium text-zinc-100">
                        {s.title}
                      </div>
                      <div className="mt-1 truncate text-xs text-zinc-500">
                        {s.kind} · {s.status} · {s.uri}
                      </div>
                      {s.error && (
                        <div className="mt-1 text-xs text-red-300">{s.error}</div>
                      )}
                    </div>
                    <button
                      type="button"
                      className="btn-ghost shrink-0 text-xs"
                      disabled={busy}
                      onClick={() => handleRemoveSource(s.id)}
                    >
                      移除
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </section>
        )}

        {view === "settings" && config && (
          <section className="glass-panel flex flex-col gap-5 p-5">
            <h2 className="text-base font-medium">设置</h2>

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
                </select>
              </label>
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
              onClick={() => handleSaveConfig(config)}
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
          </section>
        )}
      </main>
    </div>
  );
}

export default App;
