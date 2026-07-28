import { Channel, invoke } from "@tauri-apps/api/core";

import type { ChatMessage, ChatSession } from "../types/chat";
import type {
  AppConfig,
  AskResponse,
  IndexStatusView,
  SyncStatusView,
  TaskItem,
  WikiCompileSummary,
  WikiExportPreflight,
} from "../types/ipc";
import type {
  CursorTranscriptSummary,
  InsightsReport,
  RebuildReport,
  RelatedSource,
  Source,
} from "../types/library";
import type { MemorySource } from "../types/memory";
import type { Task } from "../types/tasks";

export function getConfig() {
  return invoke<AppConfig>("get_config");
}

export function setConfig(config: AppConfig) {
  return invoke<void>("set_config", { config });
}

export function getIndexStatus() {
  return invoke<IndexStatusView>("get_index_status");
}

export function sourceCount() {
  return invoke<number>("source_count");
}

export function listSources() {
  return invoke<Source[]>("list_sources");
}

export function removeSource(id: string) {
  return invoke<void>("remove_source", { id });
}

export function getSyncStatus() {
  return invoke<SyncStatusView>("get_sync_status");
}

export function runScheduledSync() {
  return invoke<RebuildReport>("run_scheduled_sync_cmd");
}

export function listChatSessions() {
  return invoke<ChatSession[]>("list_chat_sessions");
}

export function createChatSession(title?: string | null) {
  return invoke<ChatSession>("create_chat_session", { title: title ?? null });
}

export function deleteChatSession(id: string) {
  return invoke<void>("delete_chat_session", { id });
}

export function listChatMessages(sessionId: string) {
  return invoke<ChatMessage[]>("list_chat_messages", { sessionId });
}

export function askInSession(sessionId: string, question: string) {
  return invoke<AskResponse>("ask_in_session", { sessionId, question });
}

export function askInSessionStream(
  sessionId: string,
  question: string,
  onToken: Channel<{ token: string }>,
) {
  return invoke<AskResponse>("ask_in_session_stream", {
    sessionId,
    question,
    onToken,
  });
}

export function rebuildIndex() {
  return invoke<RebuildReport>("rebuild_index");
}

export function reinitAndRebuildIndex() {
  return invoke<RebuildReport>("reinit_and_rebuild_index");
}

export function retrySource(id: string) {
  return invoke<RebuildReport>("retry_source", { id });
}

export function indexFile(path: string) {
  return invoke<string>("index_file", { path });
}

export function summarizeSource(sourceId: string) {
  return invoke<string>("summarize_source_cmd", { sourceId });
}

export function listRelatedSources(sourceId: string, topN?: number | null) {
  return invoke<RelatedSource[]>("list_related_sources", {
    sourceId,
    topN: topN ?? null,
  });
}

export function extractTasks(sourceId: string) {
  return invoke<TaskItem[]>("extract_tasks_cmd", { sourceId });
}

export function compileWiki(sourceId: string) {
  return invoke<WikiCompileSummary>("compile_wiki_cmd", { sourceId });
}

export function wikiExportPreflight() {
  return invoke<WikiExportPreflight>("wiki_export_preflight_cmd");
}

export function exportWikiZip(destPath: string) {
  return invoke<void>("export_wiki_zip_cmd", { destPath });
}

export function listTasks() {
  return invoke<Task[]>("list_tasks");
}

export function updateTaskStatus(id: string, status: "pending" | "done") {
  return invoke<void>("update_task_status", { id, status });
}

export function deleteTask(id: string) {
  return invoke<void>("delete_task", { id });
}

export function runInsightsAll(summarize: boolean, extractTasksFlag: boolean) {
  return invoke<InsightsReport>("run_insights_all_cmd", {
    summarize,
    extractTasks: extractTasksFlag,
  });
}

export function listCursorTranscripts() {
  return invoke<CursorTranscriptSummary[]>("list_cursor_transcripts");
}

export function syncCursorTranscripts() {
  return invoke<RebuildReport>("sync_cursor_transcripts_cmd");
}

export function addWatchFolder(path: string) {
  return invoke<void>("add_watch_folder", { path });
}

export function removeWatchFolder(path: string) {
  return invoke<void>("remove_watch_folder", { path });
}

export function syncLarkDoc(token: string) {
  return invoke<string>("sync_lark_doc", { token });
}

export function syncLarkSheet(token: string) {
  return invoke<string>("sync_lark_sheet", { token });
}

export function syncLarkMail(messageId: string) {
  return invoke<string>("sync_lark_mail", { messageId });
}

export function syncLarkIm(chatId: string) {
  return invoke<string>("sync_lark_im", { chatId });
}

export interface ApiKeyStatus {
  hasKey: boolean;
}

export function getApiKeyStatus() {
  return invoke<ApiKeyStatus>("get_api_key_status");
}

export function setApiKey(key: string) {
  return invoke<void>("set_api_key", { key });
}

export function clearApiKey() {
  return invoke<void>("clear_api_key");
}

export function listMemories() {
  return invoke<MemorySource[]>("list_memories_cmd");
}

export function getMemoryContent(sourceId: string) {
  return invoke<string>("get_memory_content_cmd", { sourceId });
}

export function addMemory(content: string, title?: string | null) {
  return invoke<string>("add_memory_cmd", { content, title: title ?? null });
}

export function updateMemory(
  id: string,
  content: string,
  title?: string | null,
) {
  return invoke<string>("update_memory_cmd", {
    id,
    content,
    title: title ?? null,
  });
}

export function forgetMemory(id: string) {
  return invoke<void>("forget_memory_cmd", { id });
}
