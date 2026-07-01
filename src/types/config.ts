import type { AgentProfile, AppConfig } from "./ipc";

export type EmbedderProvider = AppConfig["embedder"];
export type ChatProvider = AppConfig["chat"];
export type LarkIdentity = AppConfig["lark_identity"];
export type AgentOrchestrationMode = AppConfig["agent_orchestration_mode"];

export interface EmbeddingConfig {
  embedder: EmbedderProvider;
  mock_embed_dim: number;
  ollama_base_url: string;
  ollama_embed_model: string;
  ollama_embed_dim: number;
  fastembed_model: string;
  fastembed_dim: number;
  cloud_base_url: string;
  cloud_api_key: string;
  cloud_embed_model: string;
  cloud_embed_dim: number;
}

export interface ChatConfig {
  chat: ChatProvider;
  ollama_chat_model: string;
  cloud_chat_model: string;
}

export interface LarkConfig {
  lark_cli_bin: string;
  lark_identity: LarkIdentity;
}

export interface SyncConfig {
  watch_folders: string[];
  cursor_projects_root: string;
  auto_summarize_on_index: boolean;
  auto_extract_tasks_on_index: boolean;
  scheduled_sync_enabled: boolean;
  scheduled_sync_interval_minutes: number;
  scheduled_sync_watch_folders: boolean;
  scheduled_sync_cursor: boolean;
  scheduled_sync_lark: boolean;
}

export interface AgentConfig {
  auto_learn_from_chat: boolean;
  agents: AgentProfile[];
  active_agent_id: string;
  enabled_skill_ids: string[];
  enabled_hook_ids: string[];
  enabled_plugin_ids: string[];
  agent_orchestration_mode: AgentOrchestrationMode;
  pipeline_agent_ids: string[];
  granted_plugin_permissions: string[];
}

export interface NestedAppConfig {
  embedding: EmbeddingConfig;
  chat: ChatConfig;
  lark: LarkConfig;
  sync: SyncConfig;
  agent: AgentConfig;
}

export function flatToNested(flat: AppConfig): NestedAppConfig {
  return {
    embedding: {
      embedder: flat.embedder,
      mock_embed_dim: flat.mock_embed_dim,
      ollama_base_url: flat.ollama_base_url,
      ollama_embed_model: flat.ollama_embed_model,
      ollama_embed_dim: flat.ollama_embed_dim,
      fastembed_model: flat.fastembed_model,
      fastembed_dim: flat.fastembed_dim,
      cloud_base_url: flat.cloud_base_url,
      cloud_api_key: flat.cloud_api_key,
      cloud_embed_model: flat.cloud_embed_model,
      cloud_embed_dim: flat.cloud_embed_dim,
    },
    chat: {
      chat: flat.chat,
      ollama_chat_model: flat.ollama_chat_model,
      cloud_chat_model: flat.cloud_chat_model,
    },
    lark: {
      lark_cli_bin: flat.lark_cli_bin,
      lark_identity: flat.lark_identity,
    },
    sync: {
      watch_folders: [...flat.watch_folders],
      cursor_projects_root: flat.cursor_projects_root,
      auto_summarize_on_index: flat.auto_summarize_on_index,
      auto_extract_tasks_on_index: flat.auto_extract_tasks_on_index,
      scheduled_sync_enabled: flat.scheduled_sync_enabled,
      scheduled_sync_interval_minutes: flat.scheduled_sync_interval_minutes,
      scheduled_sync_watch_folders: flat.scheduled_sync_watch_folders,
      scheduled_sync_cursor: flat.scheduled_sync_cursor,
      scheduled_sync_lark: flat.scheduled_sync_lark,
    },
    agent: {
      auto_learn_from_chat: flat.auto_learn_from_chat,
      agents: flat.agents.map((agent) => ({ ...agent })),
      active_agent_id: flat.active_agent_id,
      enabled_skill_ids: [...flat.enabled_skill_ids],
      enabled_hook_ids: [...flat.enabled_hook_ids],
      enabled_plugin_ids: [...flat.enabled_plugin_ids],
      agent_orchestration_mode: flat.agent_orchestration_mode,
      pipeline_agent_ids: [...flat.pipeline_agent_ids],
      granted_plugin_permissions: [...flat.granted_plugin_permissions],
    },
  };
}

export function nestedToFlat(nested: NestedAppConfig): AppConfig {
  return {
    ...nested.embedding,
    ...nested.chat,
    ...nested.lark,
    ...nested.sync,
    ...nested.agent,
    agents: nested.agent.agents.map((agent) => ({ ...agent })),
    watch_folders: [...nested.sync.watch_folders],
    enabled_skill_ids: [...nested.agent.enabled_skill_ids],
    enabled_hook_ids: [...nested.agent.enabled_hook_ids],
    enabled_plugin_ids: [...nested.agent.enabled_plugin_ids],
    pipeline_agent_ids: [...nested.agent.pipeline_agent_ids],
    granted_plugin_permissions: [...nested.agent.granted_plugin_permissions],
  };
}
