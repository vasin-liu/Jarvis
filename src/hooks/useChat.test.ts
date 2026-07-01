/**
 * @vitest-environment jsdom
 */
import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useChat } from "./useChat";

vi.mock("../lib/tauri", () => ({
  listChatSessions: vi.fn().mockResolvedValue([]),
  createChatSession: vi.fn().mockResolvedValue({
    id: "session-1",
    title: "新对话",
    created_at: 0,
    updated_at: 0,
  }),
  deleteChatSession: vi.fn().mockResolvedValue(undefined),
  listChatMessages: vi.fn().mockResolvedValue([]),
}));

describe("useChat", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("returns RAG session state and CRUD helpers", async () => {
    const { result } = renderHook(() => useChat());

    await waitFor(() => {
      expect(result.current.activeSessionId).toBe("session-1");
    });

    expect(result.current.sessions).toHaveLength(1);
    expect(result.current.messages).toEqual([]);
    expect(result.current.question).toBe("");
    expect(result.current.streamingDraft).toBe("");
    expect(typeof result.current.setQuestion).toBe("function");
    expect(typeof result.current.createSession).toBe("function");
    expect(typeof result.current.switchSession).toBe("function");
    expect(typeof result.current.deleteSession).toBe("function");
    expect(typeof result.current.refreshMessages).toBe("function");
  });

  it("exposes agent ask handlers and E2E state from useChat", async () => {
    const { result } = renderHook(() => useChat());

    await waitFor(() => {
      expect(result.current.activeSessionId).toBe("session-1");
    });

    expect(result.current.agentMode).toBe(true);
    expect(typeof result.current.handleRagAsk).toBe("function");
    expect(typeof result.current.handleAgentAsk).toBe("function");
    expect(result.current.e2eMode).toBe(false);
    expect(result.current.askHandleCount).toBe(0);
  });
});
