/**
 * @vitest-environment jsdom
 */
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { listTasks, updateTaskStatus } from "../lib/tauri";
import { useTasks } from "./useTasks";

const sampleTask = {
  id: "task-1",
  source_id: "source-1",
  source_title: "doc.md",
  title: "Follow up",
  description: "Check the report",
  status: "pending" as const,
  created_at: 1,
  updated_at: 1,
};

vi.mock("../lib/tauri", () => ({
  listTasks: vi.fn().mockResolvedValue([
    {
      id: "task-1",
      source_id: "source-1",
      source_title: "doc.md",
      title: "Follow up",
      description: "Check the report",
      status: "pending",
      created_at: 1,
      updated_at: 1,
    },
  ]),
  updateTaskStatus: vi.fn().mockResolvedValue(undefined),
  deleteTask: vi.fn().mockResolvedValue(undefined),
}));

describe("useTasks", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(listTasks).mockResolvedValue([sampleTask]);
  });

  it("refreshTasks sets tasks state", async () => {
    const { result } = renderHook(() => useTasks());

    await act(async () => {
      await result.current.refreshTasks();
    });

    await waitFor(() => {
      expect(result.current.tasks).toHaveLength(1);
    });
    expect(result.current.tasks[0]?.id).toBe("task-1");
    expect(listTasks).toHaveBeenCalled();
  });

  it("toggleTask flips status via updateTaskStatus", async () => {
    const { result } = renderHook(() => useTasks());

    await act(async () => {
      await result.current.refreshTasks();
    });

    await act(async () => {
      await result.current.toggleTask(sampleTask);
    });

    expect(updateTaskStatus).toHaveBeenCalledWith("task-1", "done");
    expect(listTasks).toHaveBeenCalledTimes(2);
  });

  it("returns task domain helpers", () => {
    const { result } = renderHook(() => useTasks());

    expect(typeof result.current.refreshTasks).toBe("function");
    expect(typeof result.current.toggleTask).toBe("function");
    expect(typeof result.current.deleteTask).toBe("function");
  });
});
