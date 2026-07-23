/**
 * @vitest-environment jsdom
 */
import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { AppConfig } from "../types/ipc";
import { SettingsView } from "./SettingsView";

vi.mock("motion/react", () => ({
  motion: {
    section: ({
      children,
      ...props
    }: React.PropsWithChildren<Record<string, unknown>>) => (
      <section {...props}>{children}</section>
    ),
    div: ({
      children,
      ...props
    }: React.PropsWithChildren<Record<string, unknown>>) => (
      <div {...props}>{children}</div>
    ),
  },
  AnimatePresence: ({ children }: React.PropsWithChildren) => <>{children}</>,
  useReducedMotion: () => true,
}));

const baseConfig: AppConfig = {
  watch_folders: [],
  embedder: "mock",
  chat: "mock",
  mock_embed_dim: 4,
  ollama_base_url: "http://127.0.0.1:11434",
  ollama_embed_model: "nomic-embed-text",
  ollama_chat_model: "llama3",
  ollama_embed_dim: 768,
  lark_cli_bin: "lark-cli",
  lark_identity: "user",
  fastembed_model: "bge-small-zh-v1.5",
  fastembed_dim: 512,
  cloud_base_url: "https://api.openai.com/v1",
  cloud_api_key: "",
  cloud_embed_model: "text-embedding-3-small",
  cloud_chat_model: "gpt-4o-mini",
  cloud_embed_dim: 1536,
  cursor_projects_root: "",
  auto_summarize_on_index: false,
  auto_extract_tasks_on_index: false,
  scheduled_sync_enabled: false,
  scheduled_sync_interval_minutes: 60,
  scheduled_sync_watch_folders: false,
  scheduled_sync_cursor: false,
  scheduled_sync_lark: false,
  auto_learn_from_chat: false,
  agents: [],
  active_agent_id: "",
  enabled_skill_ids: [],
  enabled_hook_ids: [],
  enabled_plugin_ids: [],
  agent_orchestration_mode: "single",
  pipeline_agent_ids: [],
  granted_plugin_permissions: [],
  wiki: { enabled: false, auto_on_insights: true },
};

const setConfig = vi.fn();
const handleSaveConfig = vi.fn();

function mockUseJarvisConfig(config: AppConfig = baseConfig) {
  return {
    config,
    setConfig,
    indexStatus: null,
    syncStatus: null,
    larkDocToken: "",
    setLarkDocToken: vi.fn(),
    larkSheetToken: "",
    setLarkSheetToken: vi.fn(),
    larkMailId: "",
    setLarkMailId: vi.fn(),
    larkChatId: "",
    setLarkChatId: vi.fn(),
    larkStatus: null,
    larkSyncError: null,
    hasApiKey: false,
    apiKeyDraft: "",
    setApiKeyDraft: vi.fn(),
    newWatchFolder: "",
    setNewWatchFolder: vi.fn(),
    skills: [],
    hooks: [],
    plugins: [],
    newAgentId: "",
    setNewAgentId: vi.fn(),
    newAgentName: "",
    setNewAgentName: vi.fn(),
    newAgentPrompt: "",
    setNewAgentPrompt: vi.fn(),
    newAgentChatProvider: "",
    setNewAgentChatProvider: vi.fn(),
    newAgentEmbedderProvider: "",
    setNewAgentEmbedderProvider: vi.fn(),
    refreshIndexStatus: vi.fn(),
    handleSaveConfig,
    handleClearApiKey: vi.fn(),
    handleRebuildIndex: vi.fn(),
    handleReinitAndRebuild: vi.fn(),
    handleCheckLark: vi.fn(),
    handleDetectLarkCli: vi.fn(),
    syncLark: vi.fn(),
    handleAddWatchFolder: vi.fn(),
    handlePickWatchFolder: vi.fn(),
    handleRemoveWatchFolder: vi.fn(),
    handlePickCursorProjectsRoot: vi.fn(),
    handleRunScheduledSync: vi.fn(),
    handleToggleSkill: vi.fn(),
    handleToggleHook: vi.fn(),
    handleTogglePlugin: vi.fn(),
    handleTogglePluginPermission: vi.fn(),
    handleUpsertAgent: vi.fn(),
    handleRemoveAgent: vi.fn(),
    handleTogglePipelineAgent: vi.fn(),
    handleMovePipelineAgent: vi.fn(),
    handleSetActiveAgent: vi.fn(),
  };
}

vi.mock("../hooks/useJarvisConfig", () => ({
  useJarvisConfig: vi.fn(),
}));

async function expandWikiSection() {
  const section = screen.getByTestId("settings-section-wiki");
  fireEvent.click(within(section).getByRole("button"));
  return section;
}

describe("SettingsView wiki section", () => {
  beforeEach(async () => {
    cleanup();
    setConfig.mockClear();
    handleSaveConfig.mockClear();
    const { useJarvisConfig } = await import("../hooks/useJarvisConfig");
    vi.mocked(useJarvisConfig).mockReturnValue(
      mockUseJarvisConfig() as ReturnType<typeof useJarvisConfig>,
    );
  });

  afterEach(() => {
    cleanup();
  });

  it("renders Wiki 笔记 section and 启用 Wiki 笔记层 toggle after expand", async () => {
    render(
      <SettingsView busy={false} setBusy={vi.fn()} settingsActive />,
    );

    const section = await expandWikiSection();
    expect(screen.getByTestId("settings-section-wiki")).toBeTruthy();
    expect(within(section).getByText("Wiki 笔记")).toBeTruthy();
    expect(within(section).getByText("启用 Wiki 笔记层")).toBeTruthy();
    expect(screen.getByTestId("wiki-enabled-toggle")).toBeTruthy();
  });

  it("checkbox unchecked when wiki.enabled is false", async () => {
    render(
      <SettingsView busy={false} setBusy={vi.fn()} settingsActive />,
    );
    await expandWikiSection();
    expect(
      (screen.getByTestId("wiki-enabled-toggle") as HTMLInputElement).checked,
    ).toBe(false);
  });

  it("checkbox checked when wiki.enabled is true", async () => {
    const { useJarvisConfig } = await import("../hooks/useJarvisConfig");
    vi.mocked(useJarvisConfig).mockReturnValue(
      mockUseJarvisConfig({
        ...baseConfig,
        wiki: { enabled: true, auto_on_insights: true },
      }) as ReturnType<typeof useJarvisConfig>,
    );
    render(
      <SettingsView busy={false} setBusy={vi.fn()} settingsActive />,
    );
    await expandWikiSection();
    expect(
      (screen.getByTestId("wiki-enabled-toggle") as HTMLInputElement).checked,
    ).toBe(true);
  });

  it("toggle calls setConfig with enabled flipped and auto_on_insights preserved; does not save", async () => {
    render(
      <SettingsView busy={false} setBusy={vi.fn()} settingsActive />,
    );
    await expandWikiSection();

    fireEvent.click(screen.getByTestId("wiki-enabled-toggle"));

    expect(setConfig).toHaveBeenCalledTimes(1);
    expect(setConfig).toHaveBeenCalledWith(
      expect.objectContaining({
        wiki: { enabled: true, auto_on_insights: true },
      }),
    );
    expect(handleSaveConfig).not.toHaveBeenCalled();
  });

  it("wiki section has exactly one checkbox (no auto_on_insights control)", async () => {
    render(
      <SettingsView busy={false} setBusy={vi.fn()} settingsActive />,
    );
    const section = await expandWikiSection();
    const checkboxes = within(section).getAllByRole("checkbox");
    expect(checkboxes).toHaveLength(1);
    expect(checkboxes[0]).toBe(screen.getByTestId("wiki-enabled-toggle"));
  });
});
