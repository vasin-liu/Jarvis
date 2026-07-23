import {
  IconAlertCircle,
  IconBrain,
  IconCircleCheck,
  IconClock,
  IconFile,
  IconFileText,
  IconMail,
  IconMessages,
  IconRobot,
  IconTable,
} from "@tabler/icons-react";
import { motion, useReducedMotion } from "motion/react";

import {
  formatIndexedAt,
  sourceKindLabel,
  statusLabel,
  statusTone,
} from "../lib/sourceDisplay";
import type { AppConfig, IndexProgressView, TaskItem } from "../types/ipc";
import type { CursorTranscriptSummary, Source } from "../types/library";

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

export interface LibraryViewProps {
  sources: Source[];
  tasks: TaskItem[];
  busy: boolean;
  indexProgress: IndexProgressView | null;
  config: AppConfig | null;
  cursorCandidates: CursorTranscriptSummary[];
  onRunInsightsAll: () => void;
  onPickAndIndex: () => void;
  onSyncCursorTranscripts: () => void;
  onSummarizeSource: (sourceId: string) => void;
  onExtractTasks: (sourceId: string) => void;
  onCompileWiki: (sourceId: string) => void;
  onExportWiki: () => void;
  onRetrySource: (sourceId: string) => void;
  onRemoveSource: (sourceId: string) => void;
}

export function LibraryView({
  sources,
  tasks: _tasks,
  busy,
  indexProgress: _indexProgress,
  config,
  cursorCandidates,
  onRunInsightsAll,
  onPickAndIndex,
  onSyncCursorTranscripts,
  onSummarizeSource,
  onExtractTasks,
  onCompileWiki,
  onExportWiki,
  onRetrySource,
  onRemoveSource,
}: LibraryViewProps) {
  const reduceMotion = useReducedMotion();
  void _tasks;
  void _indexProgress;

  return (
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
        已索引 {sources.filter((s) => s.status === "indexed").length} · 失败{" "}
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
              busy || sources.filter((s) => s.status === "indexed").length === 0
            }
            onClick={onRunInsightsAll}
          >
            一键洞察
          </button>
          <button
            type="button"
            className="btn-ghost"
            disabled={busy}
            onClick={onPickAndIndex}
          >
            选择文件索引
          </button>
          {config?.wiki?.enabled === true && (
            <button
              type="button"
              className="btn-ghost"
              disabled={busy}
              data-testid="wiki-export"
              onClick={onExportWiki}
            >
              导出 Wiki
            </button>
          )}
        </div>
      </div>

      <div className="rounded-xl border border-white/10 bg-zinc-950/40 p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h3 className="text-sm font-medium text-zinc-200">Cursor 会话</h3>
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
            onClick={onSyncCursorTranscripts}
          >
            同步 Cursor 会话
          </button>
        </div>
      </div>

      {sources.length === 0 ? (
        <p className="text-sm text-zinc-500">暂无来源。</p>
      ) : (
        <ul
          className="divide-y divide-white/10 overflow-auto rounded-xl border border-white/10"
          data-testid="source-list"
        >
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
                    <div className="mt-1 text-xs text-red-300">{s.error}</div>
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
                      onClick={() => onSummarizeSource(s.id)}
                    >
                      生成摘要
                    </button>
                    <button
                      type="button"
                      className="btn-ghost text-xs"
                      disabled={busy}
                      data-testid={`extract-tasks-${s.id}`}
                      onClick={() => onExtractTasks(s.id)}
                    >
                      提取任务
                    </button>
                    {config?.wiki?.enabled === true && s.kind !== "wiki_page" && (
                      <button
                        type="button"
                        className="btn-ghost text-xs"
                        disabled={busy}
                        data-testid={`wiki-compile-${s.id}`}
                        onClick={() => onCompileWiki(s.id)}
                      >
                        生成笔记
                      </button>
                    )}
                  </>
                )}
                <button
                  type="button"
                  className="btn-ghost text-xs"
                  disabled={busy}
                  data-testid={`retry-source-${s.id}`}
                  onClick={() => onRetrySource(s.id)}
                >
                  {s.status === "failed" ? "重试" : "重新同步"}
                </button>
                <button
                  type="button"
                  className="btn-ghost text-xs"
                  disabled={busy}
                  onClick={() => onRemoveSource(s.id)}
                >
                  移除
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </motion.section>
  );
}
