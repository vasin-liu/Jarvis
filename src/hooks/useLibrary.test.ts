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
  wikiExportPreflight: vi.fn().mockResolvedValue({ hasNotes: true }),
  exportWikiZip: vi.fn().mockResolvedValue(undefined),
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
  save: vi.fn().mockResolvedValue(null),
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

  it("exportWiki reports empty wiki and skips save dialog", async () => {
    const { wikiExportPreflight } = await import("../lib/tauri");
    const { save } = await import("@tauri-apps/plugin-dialog");
    vi.mocked(wikiExportPreflight).mockResolvedValueOnce({ hasNotes: false });
    const onError = vi.fn();
    const { result } = renderHook(() => useLibrary({ onError }));

    await act(async () => {
      await result.current.exportWiki();
    });

    expect(onError).toHaveBeenCalledWith("还没有可导出的笔记，请先生成笔记");
    expect(save).not.toHaveBeenCalled();
  });

  it("exportWiki cancels when save returns null", async () => {
    const { wikiExportPreflight, exportWikiZip } = await import("../lib/tauri");
    const { save } = await import("@tauri-apps/plugin-dialog");
    vi.mocked(wikiExportPreflight).mockResolvedValueOnce({ hasNotes: true });
    vi.mocked(save).mockResolvedValueOnce(null);
    const { result } = renderHook(() => useLibrary());

    await act(async () => {
      await result.current.exportWiki();
    });

    expect(exportWikiZip).not.toHaveBeenCalled();
  });

  it("exportWiki saves then exports and notifies onSuccess", async () => {
    const { wikiExportPreflight, exportWikiZip } = await import("../lib/tauri");
    const { save } = await import("@tauri-apps/plugin-dialog");
    const dest = "C:\\exports\\jarvis-wiki-2026-07-23.zip";
    vi.mocked(wikiExportPreflight).mockResolvedValueOnce({ hasNotes: true });
    vi.mocked(save).mockResolvedValueOnce(dest);
    const onSuccess = vi.fn();
    const { result } = renderHook(() => useLibrary());

    await act(async () => {
      await result.current.exportWiki({ onSuccess });
    });

    expect(exportWikiZip).toHaveBeenCalledWith(dest);
    expect(onSuccess).toHaveBeenCalledWith(dest);
  });

  it("exportWiki uses __JARVIS_E2E_WIKI_EXPORT_PATH__ and skips save", async () => {
    const { wikiExportPreflight, exportWikiZip } = await import("../lib/tauri");
    const { save } = await import("@tauri-apps/plugin-dialog");
    const forcedPath = "C:\\Temp\\jarvis-e2e-wiki.zip";
    vi.mocked(wikiExportPreflight).mockResolvedValueOnce({ hasNotes: true });
    (
      window as Window & { __JARVIS_E2E_WIKI_EXPORT_PATH__?: string }
    ).__JARVIS_E2E_WIKI_EXPORT_PATH__ = forcedPath;
    const onSuccess = vi.fn();
    const { result } = renderHook(() => useLibrary());

    try {
      await act(async () => {
        await result.current.exportWiki({ onSuccess });
      });

      expect(save).not.toHaveBeenCalled();
      expect(exportWikiZip).toHaveBeenCalledWith(forcedPath);
      expect(onSuccess).toHaveBeenCalledWith(forcedPath);
    } finally {
      delete (
        window as Window & { __JARVIS_E2E_WIKI_EXPORT_PATH__?: string }
      ).__JARVIS_E2E_WIKI_EXPORT_PATH__;
    }
  });

  it("exportWiki reports export errors and still excludes busy", async () => {
    const { wikiExportPreflight, exportWikiZip } = await import("../lib/tauri");
    const { save } = await import("@tauri-apps/plugin-dialog");
    vi.mocked(wikiExportPreflight).mockResolvedValueOnce({ hasNotes: true });
    vi.mocked(save).mockResolvedValueOnce("/tmp/out.zip");
    vi.mocked(exportWikiZip).mockRejectedValueOnce(new Error("zip failed"));
    const onError = vi.fn();
    const { result } = renderHook(() => useLibrary({ onError }));

    await act(async () => {
      await result.current.exportWiki();
    });

    expect(onError).toHaveBeenCalled();
    const keys = Object.keys(result.current);
    expect(keys).not.toContain("busy");
  });
});
