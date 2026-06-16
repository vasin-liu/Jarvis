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
  IconListCheck,
  IconBrain,
  IconMessages,
  IconRobot,
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

type View = "chat" | "library" | "tasks" | "memory" | "settings";

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
  summary?: string | null;
}

interface TaskItem {
  id: string;
  source_id?: string | null;
  source_title?: string | null;
  title: string;
  description?: string | null;
  status: "pending" | "done";
  created_at: number;
  updated_at: number;
}

interface InsightsReport {
  summarized: number;
  tasksExtracted: number;
  failed: number;
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
  cursor_projects_root: string;
  auto_summarize_on_index: boolean;
  auto_extract_tasks_on_index: boolean;
  scheduled_sync_enabled: boolean;
  scheduled_sync_interval_minutes: number;
  scheduled_sync_watch_folders: boolean;
  scheduled_sync_cursor: boolean;
  auto_learn_from_chat: boolean;
  agents: AgentProfile[];
  active_agent_id: string;
  enabled_skill_ids: string[];
  enabled_hook_ids: string[];
  enabled_plugin_ids: string[];
  agent_orchestration_mode: "single" | "pipeline" | "router";
  pipeline_agent_ids: string[];
  granted_plugin_permissions: string[];
}

interface OrchestrationStepInfo {
  agent_id: string;
  agent_name: string;
  answer: string;
  tool_calls: ToolCallInfo[];
}

interface ToolCallInfo {
  name: string;
  arguments: Record<string, unknown>;
  result: string;
}

interface AskResponse {
  answer: string;
  citations: unknown[];
  tool_calls?: ToolCallInfo[];
  orchestration_steps?: OrchestrationStepInfo[];
}

interface AgentProfile {
  id: string;
  name: string;
  system_prompt: string;
  enabled: boolean;
  chat_provider?: string | null;
}

interface PluginItem {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  tools: { name: string; description: string; command: string }[];
}

interface SkillItem {
  id: string;
  name: string;
  description: string;
  content: string;
}

interface HookItem {
  id: string;
  name: string;
  description: string;
  event: string;
  command: string;
}

interface SyncStatusView {
  lastScheduledSyncAt?: number | null;
  scheduledSyncEnabled: boolean;
  scheduledSyncIntervalMinutes: number;
}

interface CursorTranscriptSummary {
  session_id: string;
  project: string;
  path: string;
  uri: string;
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
    case "cursor_transcript":
      return <IconRobot className={cls} aria-hidden />;
    case "memory":
      return <IconBrain className={cls} aria-hidden />;
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
  if (uri.startsWith("lark://") || uri.startsWith("cursor://")) {
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
  const [cursorCandidates, setCursorCandidates] = useState<
    CursorTranscriptSummary[]
  >([]);
  const [newWatchFolder, setNewWatchFolder] = useState("");
  const [tasks, setTasks] = useState<TaskItem[]>([]);
  const [memories, setMemories] = useState<Source[]>([]);
  const [syncStatus, setSyncStatus] = useState<SyncStatusView | null>(null);
  const [newMemoryText, setNewMemoryText] = useState("");
  const [skills, setSkills] = useState<SkillItem[]>([]);
  const [hooks, setHooks] = useState<HookItem[]>([]);
  const [plugins, setPlugins] = useState<PluginItem[]>([]);
  const [lastToolCalls, setLastToolCalls] = useState<ToolCallInfo[]>([]);
  const [lastOrchestrationSteps, setLastOrchestrationSteps] = useState<
    OrchestrationStepInfo[]
  >([]);
  const [agentMode, setAgentMode] = useState(true);
  const [newAgentId, setNewAgentId] = useState("");
  const [newAgentName, setNewAgentName] = useState("");
  const [newAgentPrompt, setNewAgentPrompt] = useState("");
  const [newAgentChatProvider, setNewAgentChatProvider] = useState("");

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

  const refreshSyncStatus = useCallback(async () => {
    const status = await invoke<SyncStatusView>("get_sync_status");
    setSyncStatus(status);
  }, []);

  const refreshMemories = useCallback(async () => {
    const list = await invoke<Source[]>("list_memories_cmd");
    setMemories(list);
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

  const refreshTasks = useCallback(async () => {
    const list = await invoke<TaskItem[]>("list_tasks");
    setTasks(list);
  }, []);

  const refreshLibrary = useCallback(async () => {
    const list = await invoke<Source[]>("list_sources");
    setSources(list);
    const transcripts = await invoke<CursorTranscriptSummary[]>(
      "list_cursor_transcripts",
    );
    setCursorCandidates(transcripts);
    await refreshTasks();
    await refreshMemories();
    await refreshSkills();
    await refreshHooks();
    await refreshPlugins();
  }, [refreshHooks, refreshMemories, refreshPlugins, refreshSkills, refreshTasks]);

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
      refreshSyncStatus(),
      ensureSession(),
    ]).catch((e) => setErr(String(e)));
  }, [ensureSession, refreshConfig, refreshIndexStatus, refreshLibrary, refreshSyncStatus]);

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
      const resp = await invoke<AskResponse>(
        agentMode ? "ask_agent_in_session_stream" : "ask_in_session_stream",
        {
          sessionId: activeSessionId,
          question: q,
          onToken: channel,
        },
      );
      setLastToolCalls(agentMode ? (resp.tool_calls ?? []) : []);
      setLastOrchestrationSteps(
        agentMode ? (resp.orchestration_steps ?? []) : [],
      );
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
        },
      });
      setNewAgentId("");
      setNewAgentName("");
      setNewAgentPrompt("");
      setNewAgentChatProvider("");
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

  async function handlePickCursorProjectsRoot() {
    const selected = await open({ directory: true, multiple: false });
    if (selected === null || typeof selected !== "string" || !config) return;
    setConfig({ ...config, cursor_projects_root: selected });
  }

  async function handleSummarizeSource(sourceId: string) {
    setErr(null);
    setBusy(true);
    try {
      await invoke<string>("summarize_source_cmd", { sourceId });
      await refreshLibrary();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleExtractTasks(sourceId: string) {
    setErr(null);
    setBusy(true);
    try {
      await invoke<TaskItem[]>("extract_tasks_cmd", { sourceId });
      await refreshLibrary();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleRunInsightsAll() {
    setErr(null);
    setBusy(true);
    try {
      const report = await invoke<InsightsReport>("run_insights_all_cmd", {
        summarize: true,
        extractTasks: true,
      });
      await refreshLibrary();
      setLarkStatus(
        `洞察完成：摘要 ${report.summarized}，任务 ${report.tasksExtracted}，失败 ${report.failed}`,
      );
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleToggleTask(task: TaskItem) {
    setErr(null);
    setBusy(true);
    try {
      const next = task.status === "done" ? "pending" : "done";
      await invoke("update_task_status", { id: task.id, status: next });
      await refreshTasks();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleDeleteTask(id: string) {
    setErr(null);
    setBusy(true);
    try {
      await invoke("delete_task", { id });
      await refreshTasks();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleRunScheduledSync() {
    setErr(null);
    setBusy(true);
    try {
      const report = await invoke<RebuildReport>("run_scheduled_sync_cmd");
      await refreshLibrary();
      await refreshSyncStatus();
      setLarkStatus(
        `定时同步完成：成功 ${report.indexed}，失败 ${report.failed}`,
      );
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleAddMemory() {
    const text = newMemoryText.trim();
    if (!text) return;
    setErr(null);
    setBusy(true);
    try {
      await invoke("add_memory_cmd", { content: text, title: null });
      setNewMemoryText("");
      await refreshLibrary();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleSyncCursorTranscripts() {
    setErr(null);
    setBusy(true);
    try {
      const report = await invoke<RebuildReport>("sync_cursor_transcripts_cmd");
      await refreshLibrary();
      await refreshIndexStatus();
      setLarkStatus(
        `Cursor 会话同步完成：成功 ${report.indexed}，失败 ${report.failed}`,
      );
    } catch (e) {
      setErr(String(e));
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
            ["tasks", "任务", IconListCheck],
            ["memory", "记忆", IconBrain],
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
          来源 {sources.length} · v8
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
                      onChange={(e) => void handleSetActiveAgent(e.target.value)}
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
                      <span className="text-xs text-violet-300">
                        流水线模式
                      </span>
                    )}
                  {agentMode &&
                    config &&
                    config.agent_orchestration_mode === "router" && (
                      <span className="text-xs text-emerald-300">路由模式</span>
                    )}
                </div>
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
              <div className="flex flex-wrap gap-2">
                <button
                  type="button"
                  className="btn-ghost"
                  data-testid="run-insights-all"
                  disabled={
                    busy ||
                    sources.filter((s) => s.status === "indexed").length === 0
                  }
                  onClick={() => void handleRunInsightsAll()}
                >
                  一键洞察
                </button>
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

            <div className="rounded-xl border border-white/10 bg-zinc-950/40 p-4">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <h3 className="text-sm font-medium text-zinc-200">
                    Cursor 会话
                  </h3>
                  <p className="mt-1 text-xs text-zinc-500">
                    {config?.cursor_projects_root
                      ? `发现 ${cursorCandidates.length} 个本地会话（${config.cursor_projects_root}）`
                      : "请先在设置中配置 Cursor projects 根目录"}
                  </p>
                </div>
                <button
                  type="button"
                  className="btn-primary shrink-0"
                  data-testid="sync-cursor-transcripts"
                  disabled={
                    busy ||
                    !config?.cursor_projects_root?.trim() ||
                    cursorCandidates.length === 0
                  }
                  onClick={() => void handleSyncCursorTranscripts()}
                >
                  同步 Cursor 会话
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
                        {s.summary && (
                          <div className="mt-1 line-clamp-2 text-xs text-zinc-400">
                            {s.summary}
                          </div>
                        )}
                        {s.error && (
                          <div className="mt-1 text-xs text-red-300">
                            {s.error}
                          </div>
                        )}
                      </div>
                    </div>
                    <div className="flex shrink-0 flex-col gap-2">
                      {s.status === "indexed" && (
                        <>
                          <button
                            type="button"
                            className="btn-ghost text-xs"
                            disabled={busy}
                            data-testid={`summarize-${s.id}`}
                            onClick={() => void handleSummarizeSource(s.id)}
                          >
                            生成摘要
                          </button>
                          <button
                            type="button"
                            className="btn-ghost text-xs"
                            disabled={busy}
                            data-testid={`extract-tasks-${s.id}`}
                            onClick={() => void handleExtractTasks(s.id)}
                          >
                            提取任务
                          </button>
                        </>
                      )}
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

          {view === "tasks" && (
            <motion.section
              key="tasks"
              className="glass-panel flex flex-1 flex-col gap-4 p-5"
              initial={reduceMotion ? false : { opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={reduceMotion ? undefined : { opacity: 0, y: -6 }}
              transition={{ duration: 0.18 }}
            >
              <div className="flex flex-wrap items-center justify-between gap-3">
                <h2 className="text-base font-medium">任务</h2>
                <div className="text-sm text-zinc-400">
                  待办 {tasks.filter((t) => t.status === "pending").length} · 已完成{" "}
                  {tasks.filter((t) => t.status === "done").length}
                </div>
              </div>
              {tasks.length === 0 ? (
                <p className="text-sm text-zinc-500">
                  暂无任务。在资料库中对已索引来源点击「提取任务」，或开启索引后自动提取。
                </p>
              ) : (
                <ul
                  className="divide-y divide-white/10 overflow-auto rounded-xl border border-white/10"
                  data-testid="task-list"
                >
                  {tasks.map((task) => (
                    <li
                      key={task.id}
                      className="flex items-start justify-between gap-4 px-4 py-3 text-sm"
                    >
                      <div className="min-w-0">
                        <div
                          className={`font-medium ${task.status === "done" ? "text-zinc-500 line-through" : "text-zinc-100"}`}
                        >
                          {task.title}
                        </div>
                        {task.description && (
                          <div className="mt-1 text-xs text-zinc-400">
                            {task.description}
                          </div>
                        )}
                        {task.source_title && (
                          <div className="mt-1 truncate text-xs text-zinc-500">
                            来源：{task.source_title}
                          </div>
                        )}
                      </div>
                      <div className="flex shrink-0 flex-col gap-2">
                        <button
                          type="button"
                          className="btn-ghost text-xs"
                          disabled={busy}
                          data-testid={`toggle-task-${task.id}`}
                          onClick={() => void handleToggleTask(task)}
                        >
                          {task.status === "done" ? "标为待办" : "完成"}
                        </button>
                        <button
                          type="button"
                          className="btn-ghost text-xs"
                          disabled={busy}
                          onClick={() => void handleDeleteTask(task.id)}
                        >
                          删除
                        </button>
                      </div>
                    </li>
                  ))}
                </ul>
              )}
            </motion.section>
          )}

          {view === "memory" && (
            <motion.section
              key="memory"
              className="glass-panel flex flex-1 flex-col gap-4 p-5"
              initial={reduceMotion ? false : { opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={reduceMotion ? undefined : { opacity: 0, y: -6 }}
              transition={{ duration: 0.18 }}
            >
              <div className="flex flex-wrap items-center justify-between gap-3">
                <h2 className="text-base font-medium">长期记忆</h2>
                <span className="text-sm text-zinc-400">共 {memories.length} 条</span>
              </div>
              <p className="text-sm text-zinc-500">
                从对话中自动提取或手动添加的记忆会参与 RAG 检索。
              </p>
              <div className="flex gap-2">
                <input
                  className="field flex-1"
                  data-testid="new-memory-input"
                  value={newMemoryText}
                  onChange={(e) => setNewMemoryText(e.target.value)}
                  placeholder="手动添加一条记忆…"
                />
                <button
                  type="button"
                  className="btn-primary shrink-0"
                  disabled={busy || !newMemoryText.trim()}
                  onClick={() => void handleAddMemory()}
                >
                  添加
                </button>
              </div>
              {memories.length === 0 ? (
                <p className="text-sm text-zinc-500">暂无记忆。</p>
              ) : (
                <ul
                  className="divide-y divide-white/10 overflow-auto rounded-xl border border-white/10"
                  data-testid="memory-list"
                >
                  {memories.map((m) => (
                    <li
                      key={m.id}
                      className="flex items-start justify-between gap-4 px-4 py-3 text-sm"
                    >
                      <div className="min-w-0">
                        <div className="font-medium text-zinc-100">{m.title}</div>
                        {m.summary && (
                          <div className="mt-1 text-xs text-zinc-400">{m.summary}</div>
                        )}
                      </div>
                      <button
                        type="button"
                        className="btn-ghost shrink-0 text-xs"
                        disabled={busy}
                        onClick={() => void handleRemoveSource(m.id)}
                      >
                        删除
                      </button>
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
                      setConfig({
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
                      setConfig({
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
                      setConfig({
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
          )}
        </AnimatePresence>
      </main>
    </div>
  );
}

export default App;
