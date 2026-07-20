import { useCallback, useState } from "react";

import {
  deleteTask as deleteTaskCmd,
  listTasks,
  updateTaskStatus,
} from "../lib/tauri";
import type { Task } from "../types/tasks";

export interface UseTasksOptions {
  onError?: (message: string) => void;
}

export function useTasks({ onError }: UseTasksOptions = {}) {
  const [tasks, setTasks] = useState<Task[]>([]);

  const reportError = useCallback(
    (error: unknown) => {
      onError?.(String(error));
    },
    [onError],
  );

  const refreshTasks = useCallback(async () => {
    const list = await listTasks();
    setTasks(list);
    return list;
  }, []);

  const toggleTask = useCallback(
    async (task: Task) => {
      try {
        const next = task.status === "done" ? "pending" : "done";
        await updateTaskStatus(task.id, next);
        await refreshTasks();
      } catch (error) {
        reportError(error);
      }
    },
    [refreshTasks, reportError],
  );

  const deleteTask = useCallback(
    async (id: string) => {
      try {
        await deleteTaskCmd(id);
        await refreshTasks();
      } catch (error) {
        reportError(error);
      }
    },
    [refreshTasks, reportError],
  );

  return {
    tasks,
    refreshTasks,
    toggleTask,
    deleteTask,
  };
}
