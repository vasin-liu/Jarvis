import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AnimatePresence } from "motion/react";
import { AppShell } from "./AppShell";
import { useAppEvents } from "./hooks/useAppEvents";
import { useChat } from "./hooks/useChat";
import { useLibrary } from "./hooks/useLibrary";
import { useMemory } from "./hooks/useMemory";
import { useTasks } from "./hooks/useTasks";
import type { AppConfig } from "./types/ipc";
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
  const [notice, setNotice] = useState<string | null>(null);

  const {
    sources,
    cursorCandidates,
    refreshSources,
    refreshCursorCandidates,
    retrySource: libRetrySource,
    removeSource: libRemoveSource,
    summarizeSource: libSummarizeSource,
    extractTasks: libExtractTasks,
    compileWiki: libCompileWiki,
    exportWiki: libExportWiki,
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

  const [config, setConfig] = useState<AppConfig | null>(null);
  const configRef = useRef(config);
  configRef.current = config;

  const refreshLibrary = useCallback(async () => {
    await refreshSources();
    await refreshCursorCandidates();
    await refreshTasks();
    await refreshMemories();
  }, [
    refreshCursorCandidates,
    refreshMemories,
    refreshSources,
    refreshTasks,
  ]);

  const refreshConfig = useCallback(async () => {
    const cfg = await invoke<AppConfig>("get_config");
    setConfig(cfg);
  }, []);

  const refreshIndexStatus = useCallback(async () => {
    await invoke("get_index_status");
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

  useEffect(() => {
    Promise.all([refreshLibrary(), refreshConfig(), refreshIndexStatus()]).catch(
      (e) => setErr(String(e)),
    );
  }, [refreshConfig, refreshIndexStatus, refreshLibrary]);

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

  useEffect(() => {
    type OrchMode = AppConfig["agent_orchestration_mode"];
    type E2eOrchWindow = Window & {
      __JARVIS_E2E_SET_ORCHESTRATION__?: (mode: OrchMode) => void;
      __JARVIS_E2E_REFRESH_CONFIG__?: () => Promise<void>;
    };
    if (!e2eMode) {
      delete (window as E2eOrchWindow).__JARVIS_E2E_SET_ORCHESTRATION__;
      delete (window as E2eOrchWindow).__JARVIS_E2E_REFRESH_CONFIG__;
      return;
    }
    (window as E2eOrchWindow).__JARVIS_E2E_REFRESH_CONFIG__ = () => refreshConfig();
    (window as E2eOrchWindow).__JARVIS_E2E_SET_ORCHESTRATION__ = (mode) => {
      const cfg = configRef.current;
      if (!cfg) return;
      const payload: AppConfig = {
        ...cfg,
        agent_orchestration_mode: mode,
        cloud_api_key: "",
      };
      void invoke("set_config", { config: payload })
        .then(() => refreshConfig())
        .catch((e) => setErr(String(e)));
    };
    return () => {
      delete (window as E2eOrchWindow).__JARVIS_E2E_SET_ORCHESTRATION__;
      delete (window as E2eOrchWindow).__JARVIS_E2E_REFRESH_CONFIG__;
    };
  }, [e2eMode, refreshConfig]);

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

  async function handleCompileWiki(sourceId: string) {
    setErr(null);
    setBusy(true);
    try {
      await libCompileWiki(sourceId);
    } finally {
      setBusy(false);
    }
  }

  async function handleExportWiki() {
    setErr(null);
    setNotice(null);
    setBusy(true);
    try {
      await libExportWiki({
        onSuccess: (path) => {
          setNotice(`Wiki 已导出：${path}`);
        },
      });
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

  return (
    <AppShell
      view={view}
      setView={setView}
      busy={busy}
      err={err}
      setErr={setErr}
      notice={notice}
      setNotice={setNotice}
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
            onCompileWiki={(sourceId) => void handleCompileWiki(sourceId)}
            onExportWiki={() => void handleExportWiki()}
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

        {view === "settings" && (
          <SettingsView
            busy={busy}
            setBusy={setBusy}
            onError={setErr}
            settingsActive
            e2eMode={e2eMode}
            onAfterLibraryChange={refreshLibrary}
            onAfterIndexChange={async () => {
              // Settings owns its own useJarvisConfig; refresh App config so
              // Library gates (e.g. wiki.enabled) see the saved values.
              await refreshConfig();
              await refreshIndexStatus();
            }}
          />
        )}
      </AnimatePresence>
    </AppShell>
  );
}

export default App;
