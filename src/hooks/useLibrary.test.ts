/**
 * @vitest-environment jsdom
 */
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { listSources } from "../lib/tauri";
import { useLibrary } from "./useLibrary";

vi.mock("../lib/tauri", () => ({
  listSources: vi.fn().mockResolvedValue([
    {
      id: "source-1",
      kind: "local_file",
      uri: "file:///tmp/doc.md",
      title: "doc.md",
      status: "indexed",
      indexed_at: 1,
      error: null,
      summary: null,
    },
  ]),
  listCursorTranscripts: vi.fn().mockResolvedValue([]),
  retrySource: vi.fn().mockResolvedValue({ indexed: 1, failed: 0, skipped: 0 }),
  removeSource: vi.fn().mockResolvedValue(undefined),
  summarizeSource: vi.fn().mockResolvedValue("summary"),
  extractTasks: vi.fn().mockResolvedValue([]),
  compileWiki: vi.fn().mockResolvedValue({
    wikiRoot: "/tmp/wiki",
    pagesWritten: 1,
    created: 1,
    updated: 0,
    skippedUserEdit: 0,
    cleaned: 0,
  }),
  runInsightsAll: vi.fn().mockResolvedValue({
    summarized: 1,
    tasksExtracted: 0,
    failed: 0,
  }),
  indexFile: vi.fn().mockResolvedValue("source-2"),
  syncCursorTranscripts: vi.fn().mockResolvedValue({ indexed: 0, failed: 0, skipped: 0 }),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn().mockResolvedValue(null),
}));

describe("useLibrary", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("refreshSources sets sources state", async () => {
    const { result } = renderHook(() => useLibrary());

    await act(async () => {
      await result.current.refreshSources();
    });

    await waitFor(() => {
      expect(result.current.sources).toHaveLength(1);
    });
    expect(result.current.sources[0]?.id).toBe("source-1");
    expect(listSources).toHaveBeenCalled();
  });

  it("returns library domain helpers", async () => {
    const { result } = renderHook(() => useLibrary());

    expect(typeof result.current.refreshSources).toBe("function");
    expect(typeof result.current.refreshCursorCandidates).toBe("function");
    expect(typeof result.current.retrySource).toBe("function");
    expect(typeof result.current.removeSource).toBe("function");
    expect(typeof result.current.summarizeSource).toBe("function");
    expect(typeof result.current.extractTasks).toBe("function");
    expect(typeof result.current.compileWiki).toBe("function");
    expect(typeof result.current.runInsightsAll).toBe("function");
    expect(typeof result.current.pickAndIndex).toBe("function");
    expect(typeof result.current.syncCursorTranscripts).toBe("function");
  });

  it("compileWiki calls tauri and refreshSources", async () => {
    const { compileWiki: compileWikiCmd } = await import("../lib/tauri");
    const { result } = renderHook(() => useLibrary());
    await act(async () => {
      await result.current.compileWiki("source-1");
    });
    expect(compileWikiCmd).toHaveBeenCalledWith("source-1");
    expect(listSources).toHaveBeenCalled();
  });

  it("compileWiki reports errors via onError", async () => {
    const { compileWiki: compileWikiCmd } = await import("../lib/tauri");
    vi.mocked(compileWikiCmd).mockRejectedValueOnce(new Error("wiki off"));
    const onError = vi.fn();
    const { result } = renderHook(() => useLibrary({ onError }));
    await act(async () => {
      await result.current.compileWiki("source-1");
    });
    expect(onError).toHaveBeenCalled();
  });

  it("does not expose indexProgress or busy", () => {
    const { result } = renderHook(() => useLibrary());

    const keys = Object.keys(result.current);
    expect(keys).not.toContain("indexProgress");
    expect(keys).not.toContain("busy");
    expect(keys).not.toContain("err");
  });
});
