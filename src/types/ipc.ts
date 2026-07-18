export interface ChatSession {
  id: string;
  title: string;
  created_at: number;
  updated_at: number;
}

export interface ChatMessage {
  id: number;
  session_id: string;
  role: "user" | "assistant" | "system";
  content: string;
  citations_json?: string | null;
  created_at: number;
}

export type { Source } from "./library";

export type { Task as TaskItem } from "./tasks";

export type { InsightsReport } from "./library";

export interface IndexProgressView {
  phase: string;
  current: number;
  total: number;
  sourceTitle: string;
  outcome?: string | null;
  message?: string | null;
}

export interface LarkAuthStatus {
  ok: boolean;
  identity: string;
  userName?: string | null;
  userOpenId?: string | null;
  tokenStatus?: string | null;
  userAvailable: boolean;
  botAvailable: boolean;
  note?: string | null;
  hint?: string | null;
  appId?: string | null;
  brand?: string | null;
}

export interface AgentProfile {
  id: string;
  name: string;
  system_prompt: string;
  enabled: boolean;
  chat_provider?: string | null;
  embedder_provider?: string | null;
}

export interface AppConfig {
  watch_folders: string[];
  embedder: "mock" | "ollama" | "fast_embed" | "cloud";
  chat: "mock" | "ollama" | "cloud";
  mock_embed_dim: number;
  ollama_base_url: string;
  ollama_embed_model: string;
  ollama_chat_model: string;
  ollama_embed_dim: number;
  lark_cli_bin: string;
  lark_identity: "user" | "bot" | "auto";
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
  scheduled_sync_lark: boolean;
  auto_learn_from_chat: boolean;
  agents: AgentProfile[];
  active_agent_id: string;
  enabled_skill_ids: string[];
  enabled_hook_ids: string[];
  enabled_plugin_ids: string[];
  agent_orchestration_mode: "single" | "pipeline" | "router";
  pipeline_agent_ids: string[];
  granted_plugin_permissions: string[];
  wiki?: { enabled: boolean; auto_on_insights: boolean };
}

export interface ToolCallInfo {
  name: string;
  arguments: Record<string, unknown>;
  result: string;
}

export interface OrchestrationStepInfo {
  agent_id: string;
  agent_name: string;
  answer: string;
  tool_calls: ToolCallInfo[];
}

export interface AskResponse {
  answer: string;
  citations: unknown[];
  tool_calls?: ToolCallInfo[];
  orchestration_steps?: OrchestrationStepInfo[];
  toolParseWarnings?: string[];
  tool_parse_warnings?: string[];
}

export interface PluginItem {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  tools: { name: string; description: string; command: string }[];
}

export interface SkillItem {
  id: string;
  name: string;
  description: string;
  content: string;
}

export interface HookItem {
  id: string;
  name: string;
  description: string;
  event: string;
  command: string;
}

export interface SyncStatusView {
  lastScheduledSyncAt?: number | null;
  scheduledSyncEnabled: boolean;
  scheduledSyncIntervalMinutes: number;
  lastScheduledSyncError?: string | null;
  lastScheduledSyncErrorAt?: number | null;
}

export type { CursorTranscriptSummary } from "./library";

export interface IndexStatusView {
  store_dim: number;
  config_embed_dim: number;
  stored_embedder_id: string | null;
  config_embedder_id: string;
  needs_rebuild: boolean;
  dim_mismatch: boolean;
  source_count: number;
  chunk_count: number;
}

export type { RebuildReport } from "./library";
