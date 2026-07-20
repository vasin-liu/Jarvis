/**
 * @vitest-environment jsdom
 */
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { listMemories } from "../lib/tauri";
import { useMemory } from "./useMemory";

vi.mock("../lib/tauri", () => ({
  listMemories: vi.fn().mockResolvedValue([
    {
      id: "memory://550e8400-e29b-41d4-a716-446655440000",
      kind: "memory",
      uri: "memory://550e8400-e29b-41d4-a716-446655440000",
      title: "记忆: test",
      status: "indexed",
      indexed_at: 1,
      error: null,
      summary: null,
    },
  ]),
  getMemoryContent: vi.fn().mockResolvedValue("body"),
  addMemory: vi.fn().mockResolvedValue("memory://new"),
  updateMemory: vi.fn().mockResolvedValue("memory://new"),
  forgetMemory: vi.fn().mockResolvedValue(undefined),
}));

describe("useMemory", () => {
  beforeEach(() => {
    vi.mocked(listMemories).mockClear();
  });

  it("refreshMemories loads list into state", async () => {
    const { result } = renderHook(() => useMemory());

    await act(async () => {
      await result.current.refreshMemories();
    });

    await waitFor(() => {
      expect(result.current.memories).toHaveLength(1);
    });
    expect(result.current.memories[0]?.title).toBe("记忆: test");
  });
});
