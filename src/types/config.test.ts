import { describe, expect, it } from "vitest";

import type { AppConfig } from "./ipc";
import {
  flatToNested,
  nestedToFlat,
  type AgentConfig,
  type EmbeddingConfig,
  type SyncConfig,
} from "./config";

const sampleFlat: AppConfig = {
  watch_folders: ["/data/docs"],
  embedder: "fast_embed",
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
  cursor_projects_root: "C:\\Users\\me\\.cursor\\projects",
  auto_summarize_on_index: true,
  auto_extract_tasks_on_index: false,
  scheduled_sync_enabled: true,
  scheduled_sync_interval_minutes: 60,
  scheduled_sync_watch_folders: true,
  scheduled_sync_cursor: true,
  scheduled_sync_lark: false,
  auto_learn_from_chat: true,
  agents: [
    {
      id: "default",
      name: "默认",
      system_prompt: "You are helpful.",
      enabled: true,
    },
  ],
  active_agent_id: "default",
  enabled_skill_ids: ["skill-a"],
  enabled_hook_ids: [],
  enabled_plugin_ids: [],
  agent_orchestration_mode: "single",
  pipeline_agent_ids: [],
  granted_plugin_permissions: ["shell_exec"],
};

describe("config types", () => {
  it("EmbeddingConfig embedder accepts valid provider strings", () => {
    const embedding: EmbeddingConfig = {
      embedder: "cloud",
      mock_embed_dim: 4,
      ollama_base_url: "http://127.0.0.1:11434",
      ollama_embed_model: "nomic-embed-text",
      ollama_embed_dim: 768,
      fastembed_model: "bge-small-zh-v1.5",
      fastembed_dim: 512,
      cloud_base_url: "https://api.openai.com/v1",
      cloud_api_key: "",
      cloud_embed_model: "text-embedding-3-small",
      cloud_embed_dim: 1536,
    };
    expect(embedding.embedder).toBe("cloud");
    expect(["mock", "ollama", "fast_embed", "cloud"]).toContain(
      embedding.embedder,
    );
  });

  it("SyncConfig has watchFolders and scheduledSyncEnabled", () => {
    const sync: SyncConfig = {
      watch_folders: ["/watch"],
      cursor_projects_root: "",
      auto_summarize_on_index: false,
      auto_extract_tasks_on_index: false,
      scheduled_sync_enabled: true,
      scheduled_sync_interval_minutes: 30,
      scheduled_sync_watch_folders: true,
      scheduled_sync_cursor: true,
      scheduled_sync_lark: false,
    };
    expect(sync.watch_folders).toEqual(["/watch"]);
    expect(sync.scheduled_sync_enabled).toBe(true);
  });

  it("AgentConfig has agent profile fields", () => {
    const agent: AgentConfig = {
      auto_learn_from_chat: false,
      agents: sampleFlat.agents,
      active_agent_id: "default",
      enabled_skill_ids: ["x"],
      enabled_hook_ids: [],
      enabled_plugin_ids: [],
      agent_orchestration_mode: "pipeline",
      pipeline_agent_ids: ["default"],
      granted_plugin_permissions: ["shell_exec"],
    };
    expect(agent.agents).toHaveLength(1);
    expect(agent.active_agent_id).toBe("default");
    expect(agent.agent_orchestration_mode).toBe("pipeline");
  });

  it("roundtrips flat IPC payload through nested shape without data loss", () => {
    const nested = flatToNested(sampleFlat);
    const roundtrip = nestedToFlat(nested);

    expect(roundtrip).toEqual(sampleFlat);
    expect(nested.embedding.embedder).toBe("fast_embed");
    expect(nested.sync.watch_folders).toEqual(["/data/docs"]);
    expect(nested.agent.active_agent_id).toBe("default");
    expect(nested.lark.lark_identity).toBe("user");
    expect(nested.chat.chat).toBe("mock");
  });
});
