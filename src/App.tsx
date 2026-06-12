import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

type View = "chat" | "library" | "settings";

interface Citation {
  chunk_id: number;
  source_id: string;
  source_title: string;
  source_uri: string;
  loc: string;
  excerpt: string;
}

interface AskResponse {
  answer: string;
  citations: Citation[];
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

function App() {
  const [view, setView] = useState<View>("chat");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  const [question, setQuestion] = useState("");
  const [answer, setAnswer] = useState<AskResponse | null>(null);

  const [sources, setSources] = useState<Source[]>([]);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [larkToken, setLarkToken] = useState("");
  const [newWatchFolder, setNewWatchFolder] = useState("");

  const refreshLibrary = useCallback(async () => {
    const list = await invoke<Source[]>("list_sources");
    setSources(list);
  }, []);

  const refreshConfig = useCallback(async () => {
    const cfg = await invoke<AppConfig>("get_config");
    setConfig(cfg);
  }, []);

  useEffect(() => {
    Promise.all([refreshLibrary(), refreshConfig()]).catch((e) =>
      setErr(String(e)),
    );
  }, [refreshLibrary, refreshConfig]);

  async function handleAsk() {
    const q = question.trim();
    if (!q) return;
    setErr(null);
    setBusy(true);
    try {
      const resp = await invoke<AskResponse>("ask_question", { question: q });
      setAnswer(resp);
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

  async function handleSyncLark() {
    const token = larkToken.trim();
    if (!token) return;
    setErr(null);
    setBusy(true);
    try {
      await invoke<string>("sync_lark_doc", { token });
      setLarkToken("");
      await refreshLibrary();
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
          来源 {sources.length} · M4
        </div>
      </aside>

      <main className="flex flex-1 flex-col gap-4 p-4 pl-0">
        {err && (
          <div className="glass-panel border-red-400/30 bg-red-950/40 px-4 py-3 text-sm text-red-200">
            {err}
          </div>
        )}

        {view === "chat" && (
          <section className="glass-panel flex flex-1 flex-col gap-4 p-5">
            <h2 className="text-base font-medium text-zinc-100">问答</h2>
            <textarea
              className="field min-h-28 resize-y"
              value={question}
              onChange={(e) => setQuestion(e.target.value)}
              placeholder="输入问题，基于已索引资料检索并回答…"
            />
            <div className="flex gap-2">
              <button
                type="button"
                className="btn-primary"
                disabled={busy || !question.trim()}
                onClick={handleAsk}
              >
                {busy ? "处理中…" : "提问"}
              </button>
            </div>
            {answer && (
              <div className="space-y-4 border-t border-white/10 pt-4">
                <div>
                  <h3 className="mb-2 text-sm text-cyan-300/90">回答</h3>
                  <p className="whitespace-pre-wrap text-sm leading-relaxed text-zinc-200">
                    {answer.answer}
                  </p>
                </div>
                {answer.citations.length > 0 && (
                  <div>
                    <h3 className="mb-2 text-sm text-cyan-300/90">引用</h3>
                    <ul className="space-y-3">
                      {answer.citations.map((c) => (
                        <li
                          key={c.chunk_id}
                          className="rounded-xl border border-white/10 bg-zinc-950/40 p-3 text-sm"
                        >
                          <div className="font-medium text-zinc-100">
                            {c.source_title} · {c.loc}
                          </div>
                          <div className="mt-1 text-xs text-zinc-500">
                            {c.source_uri}
                          </div>
                          <div className="mt-2 text-zinc-300">{c.excerpt}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            )}
          </section>
        )}

        {view === "library" && (
          <section className="glass-panel flex flex-1 flex-col gap-4 p-5">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <h2 className="text-base font-medium">资料库</h2>
              <div className="flex flex-wrap gap-2">
                <button
                  type="button"
                  className="btn-ghost"
                  disabled={busy}
                  onClick={handlePickAndIndex}
                >
                  选择文件索引
                </button>
              </div>
            </div>

            <div className="flex flex-wrap gap-2">
              <input
                className="field max-w-md flex-1"
                value={larkToken}
                onChange={(e) => setLarkToken(e.target.value)}
                placeholder="飞书文档 token"
              />
              <button
                type="button"
                className="btn-primary"
                disabled={busy || !larkToken.trim()}
                onClick={handleSyncLark}
              >
                同步飞书文档
              </button>
            </div>

            {sources.length === 0 ? (
              <p className="text-sm text-zinc-500">
                暂无来源。添加监听文件夹、索引本地文件，或同步飞书文档。
              </p>
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
