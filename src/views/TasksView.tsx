import { motion, useReducedMotion } from "motion/react";

import type { Task } from "../types/tasks";

export interface TasksViewProps {
  tasks: Task[];
  busy: boolean;
  onToggleTask: (task: Task) => void;
  onDeleteTask: (id: string) => void;
}

export function TasksView({
  tasks,
  busy,
  onToggleTask,
  onDeleteTask,
}: TasksViewProps) {
  const reduceMotion = useReducedMotion();

  return (
    <motion.section
      key="tasks"
      data-testid="task-list"
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
          data-testid="task-list-items"
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
                  onClick={() => onToggleTask(task)}
                >
                  {task.status === "done" ? "标为待办" : "完成"}
                </button>
                <button
                  type="button"
                  className="btn-ghost text-xs"
                  disabled={busy}
                  onClick={() => onDeleteTask(task.id)}
                >
                  删除
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </motion.section>
  );
}
