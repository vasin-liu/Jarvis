import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

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
} from "../lib/tauri";
import { flatToNested, nestedToFlat, type NestedAppConfig } from "../types/config";
import type {
  AppConfig,
  HookItem,
  IndexStatusView,
  LarkAuthStatus,
  PluginItem,
  SkillItem,
  SyncStatusView,
} from "../types/ipc";

export interface UseJarvisConfigOptions {
  onError?: (message: string) => void;
  busy: boolean;
  setBusy: (value: boolean) => void;
  settingsActive?: boolean;
  e2eMode?: boolean;
  onAfterLibraryChange?: () => void | Promise<void>;
  onAfterIndexChange?: () => void | Promise<void>;
}

export function useJarvisConfig({
  onError,
  busy,
  setBusy,
  settingsActive = false,
  e2eMode = false,
  onAfterLibraryChange,
  onAfterIndexChange,
}: UseJarvisConfigOptions) {
  const [nestedConfig, setNestedConfig] = useState<NestedAppConfig | null>(null);
  const [indexStatus, setIndexStatus] = useState<IndexStatusView | null>(null);
  const [syncStatus, setSyncStatus] = useState<SyncStatusView | null>(null);
  const [larkDocToken, setLarkDocToken] = useState("");
  const [larkSheetToken, setLarkSheetToken] = useState("");
  const [larkMailId, setLarkMailId] = useState("");
  const [larkChatId, setLarkChatId] = useState("");
  const [larkStatus, setLarkStatus] = useState<LarkAuthStatus | null>(null);
  const [larkSyncError, setLarkSyncError] = useState<string | null>(null);
  const [hasApiKey, setHasApiKey] = useState(false);
  const [apiKeyDraft, setApiKeyDraft] = useState("");
  const [newWatchFolder, setNewWatchFolder] = useState("");
  const [skills, setSkills] = useState<SkillItem[]>([]);
  const [hooks, setHooks] = useState<HookItem[]>([]);
  const [plugins, setPlugins] = useState<PluginItem[]>([]);
  const [newAgentId, setNewAgentId] = useState("");
  const [newAgentName, setNewAgentName] = useState("");
  const [newAgentPrompt, setNewAgentPrompt] = useState("");
  const [newAgentChatProvider, setNewAgentChatProvider] = useState("");
  const [newAgentEmbedderProvider, setNewAgentEmbedderProvider] = useState("");

  const nestedConfigRef = useRef(nestedConfig);
  nestedConfigRef.current = nestedConfig;
  const apiKeyDraftRef = useRef(apiKeyDraft);
  apiKeyDraftRef.current = apiKeyDraft;

  const reportError = useCallback(
    (error: unknown) => {
      onError?.(String(error));
    },
    [onError],
  );

  const config: AppConfig | null = nestedConfig
    ? nestedToFlat(nestedConfig)
    : null;

  const setConfig = useCallback((next: AppConfig) => {
    setNestedConfig(flatToNested(next));
  }, []);

  const refreshConfig = useCallback(async () => {
    const cfg = await invoke<AppConfig>("get_config");
    setNestedConfig(flatToNested(cfg));
    return cfg;
  }, []);

  const refreshIndexStatus = useCallback(async () => {
    const status = await invoke<IndexStatusView>("get_index_status");
    setIndexStatus(status);
    return status;
  }, []);

  const refreshSyncStatus = useCallback(async () => {
    const status = await invoke<SyncStatusView>("get_sync_status");
    setSyncStatus(status);
    return status;
  }, []);

  const refreshSkills = useCallback(async () => {
    const list = await invoke<SkillItem[]>("list_skills");
    setSkills(list);
    return list;
  }, []);

  const refreshHooks = useCallback(async () => {
    const list = await invoke<HookItem[]>("list_hooks");
    setHooks(list);
    return list;
  }, []);

  const refreshPlugins = useCallback(async () => {
    const list = await invoke<PluginItem[]>("list_plugins");
    setPlugins(list);
    return list;
  }, []);

  const refreshSettingsData = useCallback(async () => {
    await Promise.all([
      refreshConfig(),
      refreshIndexStatus(),
      refreshSyncStatus(),
      refreshSkills(),
      refreshHooks(),
      refreshPlugins(),
    ]);
  }, [
    refreshConfig,
    refreshHooks,
    refreshIndexStatus,
    refreshPlugins,
    refreshSkills,
    refreshSyncStatus,
  ]);

  const handleSaveConfig = useCallback(
    async (next: AppConfig) => {
      setBusy(true);
      try {
        const keyToSave = apiKeyDraftRef.current.trim();
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
        setNestedConfig(flatToNested({ ...payload, cloud_api_key: "" }));
        await onAfterIndexChange?.();
      } catch (error) {
        reportError(error);
      } finally {
        setBusy(false);
      }
    },
    [onAfterIndexChange, reportError, setBusy],
  );

  useEffect(() => {
    refreshSettingsData().catch(reportError);
  }, [refreshSettingsData, reportError]);

  useEffect(() => {
    if (
      !settingsActive ||
      !config ||
      (config.embedder !== "cloud" && config.chat !== "cloud")
    ) {
      return;
    }
    void getApiKeyStatus()
      .then((status) => setHasApiKey(status.hasKey))
      .catch(() => setHasApiKey(false));
  }, [settingsActive, config?.embedder, config?.chat]);

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
      const nested = nestedConfigRef.current;
      if (!nested) return;
      const next = nestedToFlat({
        ...nested,
        agent: { ...nested.agent, agent_orchestration_mode: mode },
      });
      void handleSaveConfig(next);
    };
    return () => {
      delete (window as E2eOrchWindow).__JARVIS_E2E_SET_ORCHESTRATION__;
    };
  }, [e2eMode, handleSaveConfig]);

  const patchConfig = useCallback(
    async (patch: Partial<AppConfig>) => {
      if (!config) return;
      await handleSaveConfig({ ...config, ...patch });
    },
    [config, handleSaveConfig],
  );

  const updateConfigLocal = useCallback(
    (patch: Partial<AppConfig>) => {
      if (!config) return;
      setConfig({ ...config, ...patch });
    },
    [config, setConfig],
  );

  const handleClearApiKey = useCallback(async () => {
    setBusy(true);
    try {
      await clearApiKey();
      setHasApiKey(false);
      setApiKeyDraft("");
      if (config) {
        setConfig({ ...config, cloud_api_key: "" });
      }
    } catch (error) {
      reportError(error);
    } finally {
      setBusy(false);
    }
  }, [config, reportError, setBusy, setConfig]);

  const handleRebuildIndex = useCallback(async () => {
    setBusy(true);
    try {
      await rebuildIndex();
      await onAfterLibraryChange?.();
      await refreshIndexStatus();
    } catch (error) {
      reportError(error);
    } finally {
      setBusy(false);
    }
  }, [onAfterLibraryChange, refreshIndexStatus, reportError, setBusy]);

  const handleReinitAndRebuild = useCallback(async () => {
    if (
      !window.confirm(
        "将清除所有向量索引并按当前 Embedder 配置重新嵌入，是否继续？",
      )
    ) {
      return;
    }
    setBusy(true);
    try {
      await reinitAndRebuildIndex();
      await onAfterLibraryChange?.();
      await refreshIndexStatus();
    } catch (error) {
      reportError(error);
    } finally {
      setBusy(false);
    }
  }, [onAfterLibraryChange, refreshIndexStatus, reportError, setBusy]);

  const handleCheckLark = useCallback(async () => {
    setBusy(true);
    try {
      const status = await invoke<LarkAuthStatus>("check_lark_connection");
      setLarkStatus(status);
    } catch (error) {
      setLarkStatus(null);
      reportError(error);
    } finally {
      setBusy(false);
    }
  }, [reportError, setBusy]);

  const handleDetectLarkCli = useCallback(async () => {
    if (!config) return;
    setBusy(true);
    try {
      const detected = await invoke<string | null>("detect_lark_cli");
      if (detected) {
        await patchConfig({ lark_cli_bin: detected });
      } else {
        reportError(
          "未在 PATH 中找到 lark-cli，请先运行 npx @larksuite/cli@latest install",
        );
      }
    } catch (error) {
      reportError(error);
    } finally {
      setBusy(false);
    }
  }, [config, patchConfig, reportError, setBusy]);

  const syncLark = useCallback(
    async (kind: "doc" | "sheet" | "mail" | "im", value: string) => {
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
        await onAfterLibraryChange?.();
      } catch (error) {
        const message = String(error);
        setLarkSyncError(message);
        reportError(error);
      } finally {
        setBusy(false);
      }
    },
    [onAfterLibraryChange, reportError, setBusy],
  );

  const handleAddWatchFolder = useCallback(async () => {
    const path = newWatchFolder.trim();
    if (!path) return;
    setBusy(true);
    try {
      await invoke("add_watch_folder", { path });
      setNewWatchFolder("");
      await refreshConfig();
      await onAfterLibraryChange?.();
    } catch (error) {
      reportError(error);
    } finally {
      setBusy(false);
    }
  }, [
    newWatchFolder,
    onAfterLibraryChange,
    refreshConfig,
    reportError,
    setBusy,
  ]);

  const handlePickWatchFolder = useCallback(async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected === null || typeof selected !== "string") return;
    setNewWatchFolder(selected);
  }, []);

  const handleRemoveWatchFolder = useCallback(
    async (path: string) => {
      setBusy(true);
      try {
        await invoke("remove_watch_folder", { path });
        await refreshConfig();
      } catch (error) {
        reportError(error);
      } finally {
        setBusy(false);
      }
    },
    [refreshConfig, reportError, setBusy],
  );

  const handlePickCursorProjectsRoot = useCallback(async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected === null || typeof selected !== "string" || !config) return;
    await patchConfig({ cursor_projects_root: selected });
  }, [config, patchConfig]);

  const handleRunScheduledSync = useCallback(async () => {
    setBusy(true);
    try {
      await invoke("run_scheduled_sync_cmd");
      await onAfterLibraryChange?.();
      await refreshSyncStatus();
    } catch (error) {
      reportError(error);
    } finally {
      setBusy(false);
    }
  }, [onAfterLibraryChange, refreshSyncStatus, reportError, setBusy]);

  const handleToggleSkill = useCallback(
    async (skillId: string) => {
      if (!config) return;
      const enabled = config.enabled_skill_ids.includes(skillId);
      const next = enabled
        ? config.enabled_skill_ids.filter((id) => id !== skillId)
        : [...config.enabled_skill_ids, skillId];
      await patchConfig({ enabled_skill_ids: next });
    },
    [config, patchConfig],
  );

  const handleToggleHook = useCallback(
    async (hookId: string) => {
      if (!config) return;
      const enabled = config.enabled_hook_ids.includes(hookId);
      const next = enabled
        ? config.enabled_hook_ids.filter((id) => id !== hookId)
        : [...config.enabled_hook_ids, hookId];
      await patchConfig({ enabled_hook_ids: next });
    },
    [config, patchConfig],
  );

  const handleTogglePlugin = useCallback(
    async (pluginId: string) => {
      if (!config) return;
      const enabled = config.enabled_plugin_ids.includes(pluginId);
      const next = enabled
        ? config.enabled_plugin_ids.filter((id) => id !== pluginId)
        : [...config.enabled_plugin_ids, pluginId];
      await patchConfig({ enabled_plugin_ids: next });
    },
    [config, patchConfig],
  );

  const handleTogglePluginPermission = useCallback(
    async (permission: string) => {
      if (!config) return;
      const granted = config.granted_plugin_permissions.includes(permission);
      const next = granted
        ? config.granted_plugin_permissions.filter((p) => p !== permission)
        : [...config.granted_plugin_permissions, permission];
      await patchConfig({ granted_plugin_permissions: next });
    },
    [config, patchConfig],
  );

  const handleUpsertAgent = useCallback(async () => {
    if (!config) return;
    const id = newAgentId.trim();
    const name = newAgentName.trim();
    const system_prompt = newAgentPrompt.trim();
    if (!id || !name || !system_prompt) {
      reportError("Agent id、名称与系统提示不能为空");
      return;
    }
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
    } catch (error) {
      reportError(error);
    } finally {
      setBusy(false);
    }
  }, [
    config,
    newAgentChatProvider,
    newAgentEmbedderProvider,
    newAgentId,
    newAgentName,
    newAgentPrompt,
    refreshConfig,
    reportError,
    setBusy,
  ]);

  const handleRemoveAgent = useCallback(
    async (id: string) => {
      setBusy(true);
      try {
        await invoke("remove_agent_profile", { id });
        await refreshConfig();
      } catch (error) {
        reportError(error);
      } finally {
        setBusy(false);
      }
    },
    [refreshConfig, reportError, setBusy],
  );

  const handleTogglePipelineAgent = useCallback(
    (agentId: string) => {
      if (!config) return;
      const ids = config.pipeline_agent_ids;
      const next = ids.includes(agentId)
        ? ids.filter((id) => id !== agentId)
        : [...ids, agentId];
      updateConfigLocal({ pipeline_agent_ids: next });
    },
    [config, updateConfigLocal],
  );

  const handleMovePipelineAgent = useCallback(
    (agentId: string, direction: -1 | 1) => {
      if (!config) return;
      const ids = [...config.pipeline_agent_ids];
      const idx = ids.indexOf(agentId);
      if (idx < 0) return;
      const target = idx + direction;
      if (target < 0 || target >= ids.length) return;
      [ids[idx], ids[target]] = [ids[target], ids[idx]];
      updateConfigLocal({ pipeline_agent_ids: ids });
    },
    [config, updateConfigLocal],
  );

  const savePipelineAgents = useCallback(async () => {
    if (!config) return;
    await patchConfig({ pipeline_agent_ids: config.pipeline_agent_ids });
  }, [config, patchConfig]);

  return {
    config,
    nestedConfig,
    setConfig,
    indexStatus,
    syncStatus,
    larkDocToken,
    setLarkDocToken,
    larkSheetToken,
    setLarkSheetToken,
    larkMailId,
    setLarkMailId,
    larkChatId,
    setLarkChatId,
    larkStatus,
    larkSyncError,
    hasApiKey,
    apiKeyDraft,
    setApiKeyDraft,
    newWatchFolder,
    setNewWatchFolder,
    skills,
    hooks,
    plugins,
    newAgentId,
    setNewAgentId,
    newAgentName,
    setNewAgentName,
    newAgentPrompt,
    setNewAgentPrompt,
    newAgentChatProvider,
    setNewAgentChatProvider,
    newAgentEmbedderProvider,
    setNewAgentEmbedderProvider,
    refreshConfig,
    refreshIndexStatus,
    refreshSyncStatus,
    refreshSkills,
    refreshHooks,
    refreshPlugins,
    refreshSettingsData,
    handleSaveConfig,
    patchConfig,
    updateConfigLocal,
    handleClearApiKey,
    handleRebuildIndex,
    handleReinitAndRebuild,
    handleCheckLark,
    handleDetectLarkCli,
    syncLark,
    handleAddWatchFolder,
    handlePickWatchFolder,
    handleRemoveWatchFolder,
    handlePickCursorProjectsRoot,
    handleRunScheduledSync,
    handleToggleSkill,
    handleToggleHook,
    handleTogglePlugin,
    handleTogglePluginPermission,
    handleUpsertAgent,
    handleRemoveAgent,
    handleTogglePipelineAgent,
    handleMovePipelineAgent,
    savePipelineAgents,
    busy,
  };
}
