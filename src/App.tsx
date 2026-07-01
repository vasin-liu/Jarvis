import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { AppShell } from "./AppShell";
import { useAppEvents } from "./hooks/useAppEvents";
import { useChat } from "./hooks/useChat";
import { useLibrary } from "./hooks/useLibrary";
import { useMemory } from "./hooks/useMemory";
import { useTasks } from "./hooks/useTasks";
import {
  clearApiKey,
  getApiKeyStatus,
  rebuildIndex,
  reinitAndRebuildIndex,
  setApiKey,
  syncLarkDoc,
  syncLarkIm,
  syncLarkMail,
  syncLarkSheet,
} from "./lib/tauri";
import { formatIndexedAt } from "./lib/sourceDisplay";
import type { RebuildReport } from "./types/library";
import type {
  AppConfig,
  HookItem,
  IndexStatusView,
  LarkAuthStatus,
  PluginItem,
  SkillItem,
  SyncStatusView,
} from "./types/ipc";
import type { Task } from "./types/tasks";
import type { View } from "./types/view";
import { ChatView } from "./views/ChatView";
import { LibraryView } from "./views/LibraryView";
import { MemoryView } from "./views/MemoryView";
import { SettingsView } from "./views/SettingsView";
import { TasksView } from "./views/TasksView";

function App() {
  const [view, setView] = useState<View>("chat");
  const [busy, setBusy] = useState(false);
  const [err, setErrState] = useState<string | null>(null);
  const setErr = setErrState;

  const {
    sources,
    cursorCandidates,
    refreshSources,
    refreshCursorCandidates,
    retrySource: libRetrySource,
    removeSource: libRemoveSource,
    summarizeSource: libSummarizeSource,
    extractTasks: libExtractTasks,
    runInsightsAll: libRunInsightsAll,
    pickAndIndex: libPickAndIndex,
    syncCursorTranscripts: libSyncCursorTranscripts,
  } = useLibrary({ onError: setErr });
  const {
    tasks,
    refreshTasks,
    toggleTask: tasksToggleTask,
    deleteTask: tasksDeleteTask,
  } = useTasks({ onError: setErr });
  const {
    memories,
    newMemoryText,
    setNewMemoryText,
    editingMemoryId,
    editingMemoryContent,
    setEditingMemoryContent,
    editingMemoryTitle,
    setEditingMemoryTitle,
    memoryEditContentRef,
    refreshMemories,
    addMemory: memoryAdd,
    startEdit: memoryStartEdit,
    saveEdit: memorySaveEdit,
    cancelEdit: memoryCancelEdit,
    forgetMemory: memoryForget,
  } = useMemory({ onError: setErr });

  const [indexStatus, setIndexStatus] = useState<IndexStatusView | null>(null);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [larkDocToken, setLarkDocToken] = useState("");
  const [larkSheetToken, setLarkSheetToken] = useState("");
  const [larkMailId, setLarkMailId] = useState("");
  const [larkChatId, setLarkChatId] = useState("");
  const [larkStatus, setLarkStatus] = useState<LarkAuthStatus | null>(null);
  const [larkSyncError, setLarkSyncError] = useState<string | null>(null);
  const [hasApiKey, setHasApiKey] = useState(false);
  const [apiKeyDraft, setApiKeyDraft] = useState("");
  const [newWatchFolder, setNewWatchFolder] = useState("");
  const [syncStatus, setSyncStatus] = useState<SyncStatusView | null>(null);
  const [skills, setSkills] = useState<SkillItem[]>([]);
  const [hooks, setHooks] = useState<HookItem[]>([]);
  const [plugins, setPlugins] = useState<PluginItem[]>([]);
  const [newAgentId, setNewAgentId] = useState("");
  const [newAgentName, setNewAgentName] = useState("");
  const [newAgentPrompt, setNewAgentPrompt] = useState("");
  const [newAgentChatProvider, setNewAgentChatProvider] = useState("");
  const [newAgentEmbedderProvider, setNewAgentEmbedderProvider] = useState("");
  const configRef = useRef(config);
  configRef.current = config;

  const refreshSyncStatus = useCallback(async () => {
    const status = await invoke<SyncStatusView>("get_sync_status");
    setSyncStatus(status);
  }, []);

  const refreshSkills = useCallback(async () => {
    const list = await invoke<SkillItem[]>("list_skills");
    setSkills(list);
  }, []);

  const refreshHooks = useCallback(async () => {
    const list = await invoke<HookItem[]>("list_hooks");
    setHooks(list);
  }, []);

  const refreshPlugins = useCallback(async () => {
    const list = await invoke<PluginItem[]>("list_plugins");
    setPlugins(list);
  }, []);

  const refreshLibrary = useCallback(async () => {
    await refreshSources();
    await refreshCursorCandidates();
    await refreshTasks();
    await refreshMemories();
    await refreshSkills();
    await refreshHooks();
    await refreshPlugins();
  }, [
    refreshCursorCandidates,
    refreshHooks,
    refreshMemories,
    refreshPlugins,
    refreshSkills,
    refreshSources,
    refreshTasks,
  ]);

  const refreshConfig = useCallback(async () => {
    const cfg = await invoke<AppConfig>("get_config");
    setConfig(cfg);
  }, []);

  const refreshIndexStatus = useCallback(async () => {
    const status = await invoke<IndexStatusView>("get_index_status");
    setIndexStatus(status);
  }, []);

  const { indexProgress, listenJarvisAskDone } = useAppEvents({
    onIndexComplete: () => {
      void refreshLibrary();
      void refreshIndexStatus();
    },
  });

  const {
    sessions,
    activeSessionId,
    messages,
    question,
    streamingDraft,
    setQuestion,
    createSession,
    switchSession,
    deleteSession,
    agentMode,
    setAgentMode,
    lastToolCalls,
    lastToolParseWarnings,
    lastOrchestrationSteps,
    handleRagAsk,
    handleAgentAsk,
    e2eMode,
    askHandleCount,
  } = useChat({
    onError: setErr,
    setBusy,
    listenJarvisAskDone,
    onAfterAsk: refreshIndexStatus,
  });

  const reduceMotion = useReducedMotion();

  useEffect(() => {
    Promise.all([
      refreshLibrary(),
      refreshConfig(),
      refreshIndexStatus(),
      refreshSyncStatus(),
    ]).catch((e) => setErr(String(e)));
  }, [refreshConfig, refreshIndexStatus, refreshLibrary, refreshSyncStatus]);

  useEffect(() => {
    if (
      view !== "settings" ||
      !config ||
      (config.embedder !== "cloud" && config.chat !== "cloud")
    ) {
      return;
    }
    void getApiKeyStatus()
      .then((status) => setHasApiKey(status.hasKey))
      .catch(() => setHasApiKey(false));
  }, [view, config?.embedder, config?.chat]);

  async function handleSetActiveAgent(id: string) {
    setErr(null);
    setBusy(true);
    try {
      await invoke("set_active_agent", { id });
      await refreshConfig();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleToggleSkill(skillId: string) {
    if (!config) return;
    const enabled = config.enabled_skill_ids.includes(skillId);
    const next = enabled
      ? config.enabled_skill_ids.filter((id) => id !== skillId)
      : [...config.enabled_skill_ids, skillId];
    await handleSaveConfig({ ...config, enabled_skill_ids: next });
  }

  async function handleToggleHook(hookId: string) {
    if (!config) return;
    const enabled = config.enabled_hook_ids.includes(hookId);
    const next = enabled
      ? config.enabled_hook_ids.filter((id) => id !== hookId)
      : [...config.enabled_hook_ids, hookId];
    await handleSaveConfig({ ...config, enabled_hook_ids: next });
  }

  async function handleTogglePlugin(pluginId: string) {
    if (!config) return;
    const enabled = config.enabled_plugin_ids.includes(pluginId);
    const next = enabled
      ? config.enabled_plugin_ids.filter((id) => id !== pluginId)
      : [...config.enabled_plugin_ids, pluginId];
    await handleSaveConfig({ ...config, enabled_plugin_ids: next });
  }

  async function handleTogglePluginPermission(permission: string) {
    if (!config) return;
    const granted = config.granted_plugin_permissions.includes(permission);
    const next = granted
      ? config.granted_plugin_permissions.filter((p) => p !== permission)
      : [...config.granted_plugin_permissions, permission];
    await handleSaveConfig({ ...config, granted_plugin_permissions: next });
  }

  async function handleUpsertAgent() {
    if (!config) return;
    const id = newAgentId.trim();
    const name = newAgentName.trim();
    const system_prompt = newAgentPrompt.trim();
    if (!id || !name || !system_prompt) {
      setErr("Agent id、名称与系统提示不能为空");
      return;
    }
    setErr(null);
    setBusy(true);
    try {
      await invoke("upsert_agent_profile", {
        profile: {
          id,
          name,
          system_prompt,
          enabled: true,
          chat_provider: newAgentChatProvider.trim() || null,
          embedder_provider: newAgentEmbedderProvider.trim() || null,
        },
      });
      setNewAgentId("");
      setNewAgentName("");
      setNewAgentPrompt("");
      setNewAgentChatProvider("");
      setNewAgentEmbedderProvider("");
      await refreshConfig();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleRemoveAgent(id: string) {
    setErr(null);
    setBusy(true);
    try {
      await invoke("remove_agent_profile", { id });
      await refreshConfig();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  function handleTogglePipelineAgent(agentId: string) {
    if (!config) return;
    const ids = config.pipeline_agent_ids;
    const next = ids.includes(agentId)
      ? ids.filter((id) => id !== agentId)
      : [...ids, agentId];
    setConfig({ ...config, pipeline_agent_ids: next });
  }

  function handleMovePipelineAgent(agentId: string, direction: -1 | 1) {
    if (!config) return;
    const ids = [...config.pipeline_agent_ids];
    const idx = ids.indexOf(agentId);
    if (idx < 0) return;
    const target = idx + direction;
    if (target < 0 || target >= ids.length) return;
    [ids[idx], ids[target]] = [ids[target], ids[idx]];
    setConfig({ ...config, pipeline_agent_ids: ids });
  }

  async function handlePickAndIndex() {
    setErr(null);
    setBusy(true);
    try {
      await libPickAndIndex();
    } finally {
      setBusy(false);
    }
  }

  async function handleRetrySource(id: string) {
    setErr(null);
    setBusy(true);
    try {
      await libRetrySource(id);
      await refreshIndexStatus();
    } finally {
      setBusy(false);
    }
  }

  async function handleForgetMemory(id: string) {
    setErr(null);
    setBusy(true);
    try {
      await memoryForget(id);
      await refreshLibrary();
    } finally {
      setBusy(false);
    }
  }

  async function handleEditMemory(id: string) {
    setErr(null);
    setBusy(true);
    try {
      await memoryStartEdit(id);
    } finally {
      setBusy(false);
    }
  }

  async function handleSaveMemoryEdit() {
    setErr(null);
    setBusy(true);
    try {
      await memorySaveEdit();
      await refreshLibrary();
    } finally {
      setBusy(false);
    }
  }

  async function handleRemoveSource(id: string) {
    setErr(null);
    setBusy(true);
    try {
      await libRemoveSource(id);
    } finally {
      setBusy(false);
    }
  }

  async function handleSaveConfig(next: AppConfig) {
    setErr(null);
    setBusy(true);
    try {
      const keyToSave = apiKeyDraft.trim();
      if (keyToSave) {
        await setApiKey(keyToSave);
        setHasApiKey(true);
        setApiKeyDraft("");
      }
      const payload: AppConfig = {
        ...next,
        cloud_api_key: "",
      };
      await invoke("set_config", { config: payload });
      setConfig({ ...payload, cloud_api_key: "" });
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleClearApiKey() {
    setErr(null);
    setBusy(true);
    try {
      await clearApiKey();
      setHasApiKey(false);
      setApiKeyDraft("");
      if (config) {
        setConfig({ ...config, cloud_api_key: "" });
      }
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    type OrchMode = AppConfig["agent_orchestration_mode"];
    type E2eOrchWindow = Window & {
      __JARVIS_E2E_SET_ORCHESTRATION__?: (mode: OrchMode) => void;
    };
    if (!e2eMode) {
      delete (window as E2eOrchWindow).__JARVIS_E2E_SET_ORCHESTRATION__;
      return;
    }
    (window as E2eOrchWindow).__JARVIS_E2E_SET_ORCHESTRATION__ = (mode) => {
      const cfg = configRef.current;
      if (!cfg) return;
      void handleSaveConfig({ ...cfg, agent_orchestration_mode: mode });
    };
    return () => {
      delete (window as E2eOrchWindow).__JARVIS_E2E_SET_ORCHESTRATION__;
    };
  }, [e2eMode]);

  async function handleCheckLark() {
    setErr(null);
    setBusy(true);
    try {
      const status = await invoke<LarkAuthStatus>("check_lark_connection");
      setLarkStatus(status);
    } catch (e) {
      setLarkStatus(null);
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleDetectLarkCli() {
    if (!config) return;
    setErr(null);
    setBusy(true);
    try {
      const detected = await invoke<string | null>("detect_lark_cli");
      if (detected) {
        setConfig({ ...config, lark_cli_bin: detected });
      } else {
        setErr("未在 PATH 中找到 lark-cli，请先运行 npx @larksuite/cli@latest install");
      }
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function syncLark(
    kind: "doc" | "sheet" | "mail" | "im",
    value: string,
  ) {
    setErr(null);
    setLarkSyncError(null);
    setBusy(true);
    try {
      if (kind === "doc") {
        await syncLarkDoc(value);
      } else if (kind === "sheet") {
        await syncLarkSheet(value);
      } else if (kind === "mail") {
        await syncLarkMail(value);
      } else {
        await syncLarkIm(value);
      }
      await refreshSources();
      await refreshTasks();
    } catch (e) {
      const message = String(e);
      setLarkSyncError(message);
      setErr(message);
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

  async function handlePickCursorProjectsRoot() {
    const selected = await open({ directory: true, multiple: false });
    if (selected === null || typeof selected !== "string" || !config) return;
    setConfig({ ...config, cursor_projects_root: selected });
  }

  async function handleSummarizeSource(sourceId: string) {
    setErr(null);
    setBusy(true);
    try {
      await libSummarizeSource(sourceId);
    } finally {
      setBusy(false);
    }
  }

  async function handleExtractTasks(sourceId: string) {
    setErr(null);
    setBusy(true);
    try {
      await libExtractTasks(sourceId);
      await refreshTasks();
    } finally {
      setBusy(false);
    }
  }

  async function handleRunInsightsAll() {
    setErr(null);
    setBusy(true);
    try {
      await libRunInsightsAll();
      await refreshTasks();
    } finally {
      setBusy(false);
    }
  }

  async function handleToggleTask(task: Task) {
    setErr(null);
    setBusy(true);
    try {
      await tasksToggleTask(task);
    } finally {
      setBusy(false);
    }
  }

  async function handleDeleteTask(id: string) {
    setErr(null);
    setBusy(true);
    try {
      await tasksDeleteTask(id);
    } finally {
      setBusy(false);
    }
  }

  async function handleRunScheduledSync() {
    setErr(null);
    setBusy(true);
    try {
      await invoke<RebuildReport>("run_scheduled_sync_cmd");
      await refreshLibrary();
      await refreshSyncStatus();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleAddMemory() {
    setErr(null);
    setBusy(true);
    try {
      await memoryAdd();
      await refreshLibrary();
    } finally {
      setBusy(false);
    }
  }

  async function handleSyncCursorTranscripts() {
    setErr(null);
    setBusy(true);
    try {
      await libSyncCursorTranscripts();
      await refreshIndexStatus();
    } finally {
      setBusy(false);
    }
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
      await rebuildIndex();
      await refreshLibrary();
      await refreshIndexStatus();
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
      await reinitAndRebuildIndex();
      await refreshLibrary();
      await refreshIndexStatus();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <AppShell
      view={view}
      setView={setView}
      busy={busy}
      err={err}
      setErr={setErr}
      sourceCount={sources.length}
      e2eMode={e2eMode}
      askHandleCount={askHandleCount}
    >
      {view === "chat" && activeSessionId && (
        <div data-testid="chat-session-ready" className="hidden" />
      )}

      <AnimatePresence mode="wait">
          {view === "chat" && (
            <ChatView
              sessions={sessions}
              activeSessionId={activeSessionId}
              messages={messages}
              question={question}
              streamingDraft={streamingDraft}
              onSetQuestion={setQuestion}
              onCreateSession={createSession}
              onSwitchSession={switchSession}
              onDeleteSession={deleteSession}
              agentMode={agentMode}
              setAgentMode={setAgentMode}
              lastToolCalls={lastToolCalls}
              lastToolParseWarnings={lastToolParseWarnings}
              lastOrchestrationSteps={lastOrchestrationSteps}
              onRagAsk={handleRagAsk}
              onAgentAsk={handleAgentAsk}
              onManageAgent={() => setView("settings")}
              onSetActiveAgent={(id) => void handleSetActiveAgent(id)}
              busy={busy}
              err={err}
              setErr={setErr}
              indexProgress={indexProgress}
              e2eMode={e2eMode}
              config={config}
            />
          )}

          {view === "library" && (
            <LibraryView
              sources={sources}
              tasks={tasks}
              busy={busy}
              indexProgress={indexProgress}
              config={config}
              cursorCandidates={cursorCandidates}
              onRunInsightsAll={() => void handleRunInsightsAll()}
              onPickAndIndex={() => void handlePickAndIndex()}
              onSyncCursorTranscripts={() => void handleSyncCursorTranscripts()}
              onSummarizeSource={(sourceId) =>
                void handleSummarizeSource(sourceId)
              }
              onExtractTasks={(sourceId) => void handleExtractTasks(sourceId)}
              onRetrySource={(id) => void handleRetrySource(id)}
              onRemoveSource={(id) => void handleRemoveSource(id)}
            />
          )}

          {view === "tasks" && (
            <TasksView
              tasks={tasks}
              busy={busy}
              onToggleTask={(task) => void handleToggleTask(task)}
              onDeleteTask={(id) => void handleDeleteTask(id)}
            />
          )}

          {view === "memory" && (
            <MemoryView
              memories={memories}
              busy={busy}
              newMemoryText={newMemoryText}
              onNewMemoryTextChange={setNewMemoryText}
              editingMemoryId={editingMemoryId}
              editingMemoryContent={editingMemoryContent}
              onEditingMemoryContentChange={setEditingMemoryContent}
              editingMemoryTitle={editingMemoryTitle}
              onEditingMemoryTitleChange={setEditingMemoryTitle}
              memoryEditContentRef={memoryEditContentRef}
              onAddMemory={() => void handleAddMemory()}
              onEditMemory={(id) => void handleEditMemory(id)}
              onSaveEdit={() => void handleSaveMemoryEdit()}
              onCancelEdit={memoryCancelEdit}
              onForgetMemory={(id) => void handleForgetMemory(id)}
            />
          )}

          {view === "settings" && config && (
            <SettingsView>
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
              <div className="mb-2 text-sm text-zinc-300">飞书 / lark-cli</div>
              <p className="mb-3 text-xs text-zinc-500">
                官方接入：安装 <code className="text-zinc-400">npx @larksuite/cli@latest install</code>
                ，运行 <code className="text-zinc-400">lark-cli config init</code> 与{" "}
                <code className="text-zinc-400">lark-cli auth login</code>。个人文档/邮箱/日历需用户身份（--as user）。
              </p>
              <div className="flex flex-wrap items-center gap-3">
                <button
                  type="button"
                  className="btn-ghost"
                  data-testid="lark-check-connection"
                  disabled={busy}
                  onClick={handleCheckLark}
                >
                  检测连接
                </button>
                <button
                  type="button"
                  className="btn-ghost"
                  data-testid="lark-detect-cli"
                  disabled={busy || !config}
                  onClick={() => void handleDetectLarkCli()}
                >
                  自动检测 CLI 路径
                </button>
              </div>
              {larkStatus && (
                <div
                  className="mt-3 space-y-1 text-sm"
                  data-testid="lark-status-panel"
                >
                  <div className={larkStatus.ok ? "text-emerald-300" : "text-amber-300"}>
                    {larkStatus.ok ? "lark-cli 可用" : "lark-cli 未就绪"}
                    {larkStatus.userName ? ` · ${larkStatus.userName}` : ""}
                    {larkStatus.identity ? ` · 当前身份 ${larkStatus.identity}` : ""}
                  </div>
                  <div className="text-xs text-zinc-500">
                    用户身份：{larkStatus.userAvailable ? "就绪" : "缺失"}
                    {" · "}
                    应用身份：{larkStatus.botAvailable ? "就绪" : "缺失"}
                    {larkStatus.tokenStatus ? ` · token ${larkStatus.tokenStatus}` : ""}
                  </div>
                  {larkStatus.note && (
                    <div className="text-xs text-zinc-500">{larkStatus.note}</div>
                  )}
                  {larkStatus.hint && (
                    <div className="text-xs text-amber-200/90">{larkStatus.hint}</div>
                  )}
                </div>
              )}
              <div className="mt-4 grid gap-3 md:grid-cols-2">
                <div className="flex gap-2 md:col-span-2">
                  <input
                    className="field flex-1"
                    data-testid="lark-sync-url-input"
                    value={larkDocToken}
                    onChange={(e) => setLarkDocToken(e.target.value)}
                    placeholder="飞书链接（文档/云文件/表格）或 token"
                  />
                  <button
                    type="button"
                    className="btn-primary shrink-0"
                    data-testid="lark-sync-submit"
                    disabled={busy || !larkDocToken.trim()}
                    onClick={() =>
                      void syncLark("doc", larkDocToken.trim())
                    }
                  >
                    同步飞书
                  </button>
                </div>
                <div className="flex gap-2">
                  <input
                    className="field flex-1"
                    value={larkSheetToken}
                    onChange={(e) => setLarkSheetToken(e.target.value)}
                    placeholder="电子表格 URL 或 token"
                  />
                  <button
                    type="button"
                    className="btn-primary shrink-0"
                    disabled={busy || !larkSheetToken.trim()}
                    onClick={() =>
                      void syncLark("sheet", larkSheetToken.trim())
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
                    onClick={() => void syncLark("mail", larkMailId.trim())}
                  >
                    同步邮件
                  </button>
                </div>
                <div className="flex gap-2 md:col-span-2">
                  <input
                    className="field flex-1"
                    value={larkChatId}
                    onChange={(e) => setLarkChatId(e.target.value)}
                    placeholder="IM chat id（oc_xxx）或用户 open_id（ou_xxx）"
                  />
                  <button
                    type="button"
                    className="btn-primary shrink-0"
                    disabled={busy || !larkChatId.trim()}
                    onClick={() => void syncLark("im", larkChatId.trim())}
                  >
                    同步会话
                  </button>
                </div>
              </div>
              {larkSyncError && (
                <div className="mt-3 text-xs text-red-300">{larkSyncError}</div>
              )}
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
                    <div className="flex flex-wrap gap-2">
                      <input
                        className="field min-w-0 flex-1"
                        type="password"
                        autoComplete="off"
                        data-testid="cloud-api-key-input"
                        value={apiKeyDraft}
                        onChange={(e) => setApiKeyDraft(e.target.value)}
                        placeholder={
                          hasApiKey && !apiKeyDraft
                            ? "••••••••（已保存）"
                            : "sk-…"
                        }
                      />
                      <button
                        type="button"
                        className="btn-ghost shrink-0"
                        data-testid="clear-api-key"
                        disabled={busy}
                        onClick={() => void handleClearApiKey()}
                      >
                        清除密钥
                      </button>
                    </div>
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
              <label className="space-y-1 text-sm">
                <span className="text-zinc-400">飞书 API 身份</span>
                <select
                  className="field"
                  value={config.lark_identity ?? "user"}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      lark_identity: e.target.value as AppConfig["lark_identity"],
                    })
                  }
                >
                  <option value="user">user（用户资源：文档/邮箱/IM）</option>
                  <option value="bot">bot（应用身份）</option>
                  <option value="auto">auto（CLI 自动选择）</option>
                </select>
              </label>
              <label className="space-y-1 text-sm md:col-span-2">
                <span className="text-zinc-400">Cursor projects 根目录</span>
                <div className="flex gap-2">
                  <input
                    className="field flex-1"
                    data-testid="cursor-projects-root"
                    value={config.cursor_projects_root}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        cursor_projects_root: e.target.value,
                      })
                    }
                    placeholder="%USERPROFILE%\\.cursor\\projects"
                  />
                  <button
                    type="button"
                    className="btn-ghost shrink-0"
                    onClick={() => void handlePickCursorProjectsRoot()}
                  >
                    浏览
                  </button>
                </div>
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
                Agent 编排
              </h3>
              <div className="mb-4 flex flex-col gap-3 text-sm">
                <label className="flex items-center gap-2 text-zinc-300">
                  <input
                    type="radio"
                    name="orchestration-mode"
                    checked={config.agent_orchestration_mode === "single"}
                    onChange={() =>
                      void handleSaveConfig({
                        ...config,
                        agent_orchestration_mode: "single",
                      })
                    }
                  />
                  单 Agent（使用当前选中的配置）
                </label>
                <label className="flex items-center gap-2 text-zinc-300">
                  <input
                    type="radio"
                    name="orchestration-mode"
                    data-testid="pipeline-mode-toggle"
                    checked={config.agent_orchestration_mode === "pipeline"}
                    onChange={() =>
                      void handleSaveConfig({
                        ...config,
                        agent_orchestration_mode: "pipeline",
                      })
                    }
                  />
                  多 Agent 流水线（按顺序协作）
                </label>
                <label className="flex items-center gap-2 text-zinc-300">
                  <input
                    type="radio"
                    name="orchestration-mode"
                    data-testid="router-mode-toggle"
                    checked={config.agent_orchestration_mode === "router"}
                    onChange={() =>
                      void handleSaveConfig({
                        ...config,
                        agent_orchestration_mode: "router",
                      })
                    }
                  />
                  智能路由（自动选择最合适的 Agent）
                </label>
                {config.agent_orchestration_mode === "pipeline" && (
                  <div className="ml-6 space-y-2 rounded-xl border border-violet-400/20 p-3">
                    <p className="text-xs text-zinc-500">
                      勾选并排序参与流水线的 Agent；留空则使用全部已启用 Agent。
                    </p>
                    {config.agents
                      .filter((a) => a.enabled)
                      .map((agent) => {
                        const inPipeline = config.pipeline_agent_ids.includes(
                          agent.id,
                        );
                        const order = config.pipeline_agent_ids.indexOf(
                          agent.id,
                        );
                        return (
                          <div
                            key={agent.id}
                            className="flex flex-wrap items-center gap-2"
                          >
                            <label className="flex items-center gap-2 text-zinc-300">
                              <input
                                type="checkbox"
                                checked={inPipeline}
                                onChange={() =>
                                  handleTogglePipelineAgent(agent.id)
                                }
                              />
                              {agent.name}
                              {inPipeline && order >= 0 && (
                                <span className="text-xs text-violet-300">
                                  #{order + 1}
                                </span>
                              )}
                            </label>
                            {inPipeline && (
                              <>
                                <button
                                  type="button"
                                  className="btn-ghost px-2 py-0.5 text-xs"
                                  onClick={() =>
                                    handleMovePipelineAgent(agent.id, -1)
                                  }
                                >
                                  ↑
                                </button>
                                <button
                                  type="button"
                                  className="btn-ghost px-2 py-0.5 text-xs"
                                  onClick={() =>
                                    handleMovePipelineAgent(agent.id, 1)
                                  }
                                >
                                  ↓
                                </button>
                              </>
                            )}
                          </div>
                        );
                      })}
                  </div>
                )}
              </div>
            </div>

            <div className="border-t border-white/10 pt-4">
              <h3 className="mb-3 text-sm font-medium text-zinc-200">
                Agent 配置
              </h3>
              <p className="mb-3 text-xs text-zinc-500">
                自定义 Agent 配置会保存到 config.json。
              </p>
              {config.agents.length > 0 && (
                <div className="mb-4 space-y-2">
                  {config.agents.map((agent) => (
                    <div
                      key={agent.id}
                      className="flex items-start justify-between gap-3 rounded-xl border border-white/10 px-3 py-2 text-sm"
                    >
                      <div className="min-w-0">
                        <div className="font-medium text-zinc-200">
                          {agent.name}{" "}
                          <span className="font-mono text-xs text-zinc-500">
                            ({agent.id})
                          </span>
                        </div>
                        <div className="mt-1 text-xs text-zinc-500 line-clamp-2">
                          {agent.system_prompt}
                        </div>
                        {agent.chat_provider && (
                          <div className="mt-1 text-xs text-cyan-400/80">
                            Chat: {agent.chat_provider}
                          </div>
                        )}
                        {agent.embedder_provider && (
                          <div className="mt-1 text-xs text-emerald-400/80">
                            Embedder: {agent.embedder_provider}
                          </div>
                        )}
                      </div>
                      {agent.id !== "default" && (
                        <button
                          type="button"
                          className="btn-ghost shrink-0 text-xs text-red-300"
                          disabled={busy}
                          onClick={() => void handleRemoveAgent(agent.id)}
                        >
                          删除
                        </button>
                      )}
                    </div>
                  ))}
                </div>
              )}
              <div className="space-y-2 rounded-xl border border-white/10 p-3">
                <p className="text-xs text-zinc-500">添加或覆盖 Agent</p>
                <input
                  className="field text-sm"
                  placeholder="id（如 researcher）"
                  value={newAgentId}
                  onChange={(e) => setNewAgentId(e.target.value)}
                />
                <input
                  className="field text-sm"
                  placeholder="显示名称"
                  value={newAgentName}
                  onChange={(e) => setNewAgentName(e.target.value)}
                />
                <textarea
                  className="field min-h-20 text-sm"
                  placeholder="系统提示词"
                  value={newAgentPrompt}
                  onChange={(e) => setNewAgentPrompt(e.target.value)}
                />
                <select
                  className="field text-sm"
                  value={newAgentChatProvider}
                  onChange={(e) => setNewAgentChatProvider(e.target.value)}
                >
                  <option value="">Chat Provider：继承全局</option>
                  <option value="mock">Mock</option>
                  <option value="ollama">Ollama</option>
                  <option value="cloud">Cloud</option>
                </select>
                <select
                  className="field text-sm"
                  value={newAgentEmbedderProvider}
                  onChange={(e) => setNewAgentEmbedderProvider(e.target.value)}
                >
                  <option value="">Embedder：继承全局</option>
                  <option value="mock">Mock</option>
                  <option value="ollama">Ollama</option>
                  <option value="fastembed">FastEmbed</option>
                  <option value="cloud">Cloud</option>
                </select>
                <button
                  type="button"
                  className="btn-primary w-fit text-sm"
                  disabled={busy}
                  onClick={() => void handleUpsertAgent()}
                >
                  保存 Agent
                </button>
              </div>
            </div>

            <div className="border-t border-white/10 pt-4">
              <h3 className="mb-3 text-sm font-medium text-zinc-200">
                Skills
              </h3>
              <p className="mb-3 text-xs text-zinc-500">
                Skills 位于应用数据目录下的 <code className="text-zinc-400">skills/</code>，Markdown 格式。
              </p>
              <div className="flex flex-col gap-2 text-sm">
                {skills.length === 0 ? (
                  <p className="text-zinc-500">暂无 Skills。</p>
                ) : (
                  skills.map((skill) => (
                    <label
                      key={skill.id}
                      className="flex items-start gap-2 rounded-xl border border-white/10 px-3 py-2 text-zinc-300"
                    >
                      <input
                        type="checkbox"
                        checked={config.enabled_skill_ids.includes(skill.id)}
                        onChange={() => void handleToggleSkill(skill.id)}
                      />
                      <span>
                        <span className="font-medium text-zinc-100">{skill.name}</span>
                        <span className="mt-0.5 block text-xs text-zinc-500">
                          {skill.description}
                        </span>
                      </span>
                    </label>
                  ))
                )}
              </div>
            </div>

            <div className="border-t border-white/10 pt-4">
              <h3 className="mb-3 text-sm font-medium text-zinc-200">
                Hooks
              </h3>
              <p className="mb-3 text-xs text-zinc-500">
                Hooks 位于 <code className="text-zinc-400">hooks/*.json</code>
                ，在 Agent 事件时执行 shell 命令（环境变量：JARVIS_HOOK_EVENT、JARVIS_TOOL_NAME 等）。
              </p>
              <div className="flex flex-col gap-2 text-sm">
                {hooks.length === 0 ? (
                  <p className="text-zinc-500">暂无 Hooks。</p>
                ) : (
                  hooks.map((hook) => (
                    <label
                      key={hook.id}
                      className="flex items-start gap-2 rounded-xl border border-white/10 px-3 py-2"
                    >
                      <input
                        type="checkbox"
                        className="mt-1"
                        checked={config.enabled_hook_ids.includes(hook.id)}
                        onChange={() => void handleToggleHook(hook.id)}
                      />
                      <span>
                        <span className="font-medium text-zinc-100">{hook.name}</span>
                        <span className="mt-0.5 block text-xs text-zinc-500">
                          {hook.event} · {hook.description || hook.command}
                        </span>
                      </span>
                    </label>
                  ))
                )}
              </div>
            </div>

            <div className="border-t border-white/10 pt-4">
              <h3 className="mb-3 text-sm font-medium text-zinc-200">
                Plugins
              </h3>
              <p className="mb-3 text-xs text-zinc-500">
                Plugins 位于 <code className="text-zinc-400">plugins/*/plugin.json</code>
                ，可向 Agent 注册额外工具（shell 命令，环境变量 JARVIS_TOOL_ARGS）。
              </p>
              <div className="mb-3 flex flex-wrap gap-3 text-sm">
                <label className="flex items-center gap-2 text-zinc-300">
                  <input
                    type="checkbox"
                    checked={config.granted_plugin_permissions.includes(
                      "shell_exec",
                    )}
                    onChange={() => void handleTogglePluginPermission("shell_exec")}
                  />
                  授权 shell_exec
                </label>
              </div>
              <div className="flex flex-col gap-2 text-sm">
                {plugins.length === 0 ? (
                  <p className="text-zinc-500">暂无 Plugins。</p>
                ) : (
                  plugins.map((plugin) => (
                    <label
                      key={plugin.id}
                      className="flex items-start gap-2 rounded-xl border border-white/10 px-3 py-2"
                    >
                      <input
                        type="checkbox"
                        className="mt-1"
                        checked={config.enabled_plugin_ids.includes(plugin.id)}
                        onChange={() => void handleTogglePlugin(plugin.id)}
                      />
                      <span>
                        <span className="font-medium text-zinc-100">{plugin.name}</span>
                        <span className="mt-0.5 block text-xs text-zinc-500">
                          {plugin.description || plugin.id}
                        </span>
                        <span className="mt-1 block text-xs text-zinc-600">
                          工具：{plugin.tools.map((t) => t.name).join(", ")}
                        </span>
                        {plugin.permissions?.length > 0 && (
                          <span className="mt-1 block text-xs text-amber-400/80">
                            需要权限：{plugin.permissions.join(", ")}
                          </span>
                        )}
                      </span>
                    </label>
                  ))
                )}
              </div>
            </div>

            <div className="border-t border-white/10 pt-4">
              <h3 className="mb-3 text-sm font-medium text-zinc-200">
                定时同步
              </h3>
              <div className="mb-4 flex flex-col gap-3 text-sm">
                <label className="flex items-center gap-2 text-zinc-300">
                  <input
                    type="checkbox"
                    checked={config.scheduled_sync_enabled}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        scheduled_sync_enabled: e.target.checked,
                      })
                    }
                  />
                  启用定时同步
                </label>
                <label className="space-y-1">
                  <span className="text-zinc-400">间隔（分钟，最少 5）</span>
                  <input
                    type="number"
                    min={5}
                    className="field max-w-xs"
                    value={config.scheduled_sync_interval_minutes}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        scheduled_sync_interval_minutes: Math.max(
                          5,
                          Number(e.target.value) || 5,
                        ),
                      })
                    }
                  />
                </label>
                <label className="flex items-center gap-2 text-zinc-300">
                  <input
                    type="checkbox"
                    checked={config.scheduled_sync_watch_folders}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        scheduled_sync_watch_folders: e.target.checked,
                      })
                    }
                  />
                  同步监听文件夹
                </label>
                <label className="flex items-center gap-2 text-zinc-300">
                  <input
                    type="checkbox"
                    checked={config.scheduled_sync_cursor}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        scheduled_sync_cursor: e.target.checked,
                      })
                    }
                  />
                  同步 Cursor 会话
                </label>
                <label className="flex items-center gap-2 text-zinc-300">
                  <input
                    type="checkbox"
                    checked={config.scheduled_sync_lark ?? false}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        scheduled_sync_lark: e.target.checked,
                      })
                    }
                  />
                  刷新已索引的飞书来源
                </label>
                <label className="flex items-center gap-2 text-zinc-300">
                  <input
                    type="checkbox"
                    checked={config.auto_learn_from_chat}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        auto_learn_from_chat: e.target.checked,
                      })
                    }
                  />
                  对话后自动提取记忆
                </label>
              </div>
              {syncStatus && (
                <p className="mb-3 text-xs text-zinc-500">
                  上次定时同步：
                  {syncStatus.lastScheduledSyncAt
                    ? formatIndexedAt(syncStatus.lastScheduledSyncAt)
                    : "尚未执行"}
                </p>
              )}
              <button
                type="button"
                className="btn-ghost text-sm"
                data-testid="run-scheduled-sync"
                disabled={busy}
                onClick={() => void handleRunScheduledSync()}
              >
                立即同步
              </button>
            </div>

            <div className="border-t border-white/10 pt-4">
              <h3 className="mb-3 text-sm font-medium text-zinc-200">
                自动洞察
              </h3>
              <div className="mb-6 flex flex-col gap-3 text-sm">
                <label className="flex items-center gap-2 text-zinc-300">
                  <input
                    type="checkbox"
                    checked={config.auto_summarize_on_index}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        auto_summarize_on_index: e.target.checked,
                      })
                    }
                  />
                  索引完成后自动生成摘要
                </label>
                <label className="flex items-center gap-2 text-zinc-300">
                  <input
                    type="checkbox"
                    checked={config.auto_extract_tasks_on_index}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        auto_extract_tasks_on_index: e.target.checked,
                      })
                    }
                  />
                  索引完成后自动提取任务
                </label>
              </div>
            </div>

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
            </SettingsView>
          )}
        </AnimatePresence>
    </AppShell>
  );
}

export default App;
