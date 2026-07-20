import { motion, useReducedMotion } from "motion/react";
import type { RefObject } from "react";

import type { MemorySource } from "../types/memory";

export interface MemoryViewProps {
  memories: MemorySource[];
  busy: boolean;
  newMemoryText: string;
  onNewMemoryTextChange: (value: string) => void;
  editingMemoryId: string | null;
  editingMemoryContent: string;
  onEditingMemoryContentChange: (value: string) => void;
  editingMemoryTitle: string;
  onEditingMemoryTitleChange: (value: string) => void;
  memoryEditContentRef: RefObject<HTMLTextAreaElement | null>;
  onAddMemory: () => void;
  onEditMemory: (id: string) => void;
  onSaveEdit: () => void;
  onCancelEdit: () => void;
  onForgetMemory: (id: string) => void;
}

export function MemoryView({
  memories,
  busy,
  newMemoryText,
  onNewMemoryTextChange,
  editingMemoryId,
  editingMemoryContent,
  onEditingMemoryContentChange,
  editingMemoryTitle,
  onEditingMemoryTitleChange,
  memoryEditContentRef,
  onAddMemory,
  onEditMemory,
  onSaveEdit,
  onCancelEdit,
  onForgetMemory,
}: MemoryViewProps) {
  const reduceMotion = useReducedMotion();

  return (
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
          onChange={(e) => onNewMemoryTextChange(e.target.value)}
          placeholder="手动添加一条记忆…"
        />
        <button
          type="button"
          className="btn-primary shrink-0"
          data-testid="add-memory-submit"
          disabled={busy || !newMemoryText.trim()}
          onClick={() => void onAddMemory()}
        >
          添加
        </button>
      </div>
      <ul
        className="divide-y divide-white/10 overflow-auto rounded-xl border border-white/10"
        data-testid="memory-list"
      >
        {memories.length === 0 ? (
          <li className="px-4 py-3 text-sm text-zinc-500">暂无记忆。</li>
        ) : (
          memories.map((m) => (
            <li key={m.id} className="flex flex-col gap-2 px-4 py-3 text-sm">
              <div className="flex items-start justify-between gap-4">
                <div className="min-w-0">
                  <div className="font-medium text-zinc-100">{m.title}</div>
                  {m.summary && (
                    <div className="mt-1 text-xs text-zinc-400">{m.summary}</div>
                  )}
                  <div className="mt-1 font-mono text-xs text-zinc-600">{m.uri}</div>
                </div>
                <div className="flex shrink-0 gap-2">
                  <button
                    type="button"
                    className="btn-ghost text-xs"
                    data-testid={`memory-edit-${m.id}`}
                    disabled={busy}
                    onClick={() => void onEditMemory(m.id)}
                  >
                    编辑
                  </button>
                  <button
                    type="button"
                    className="btn-ghost shrink-0 text-xs text-red-300"
                    data-testid={`memory-forget-${m.id}`}
                    disabled={busy}
                    onClick={() => void onForgetMemory(m.id)}
                  >
                    忘记
                  </button>
                </div>
              </div>
              {editingMemoryId === m.id && (
                <div
                  className="space-y-2 rounded-lg border border-white/10 bg-black/20 p-3"
                  data-testid="memory-edit-panel"
                >
                  <input
                    className="field text-sm"
                    data-testid="memory-edit-title"
                    placeholder="标题（可选）"
                    value={editingMemoryTitle}
                    onChange={(e) => onEditingMemoryTitleChange(e.target.value)}
                  />
                  <textarea
                    ref={memoryEditContentRef}
                    className="field min-h-24 text-sm"
                    data-testid="memory-edit-content"
                    value={editingMemoryContent}
                    onChange={(e) => onEditingMemoryContentChange(e.target.value)}
                  />
                  <div className="flex gap-2">
                    <button
                      type="button"
                      className="btn-primary text-xs"
                      data-testid="memory-save-edit"
                      disabled={busy}
                      onClick={() => void onSaveEdit()}
                    >
                      保存
                    </button>
                    <button
                      type="button"
                      className="btn-ghost text-xs"
                      disabled={busy}
                      onClick={onCancelEdit}
                    >
                      取消
                    </button>
                  </div>
                </div>
              )}
            </li>
          ))
        )}
      </ul>
    </motion.section>
  );
}
