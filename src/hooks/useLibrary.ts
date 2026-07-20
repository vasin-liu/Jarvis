import { useCallback, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import {
  extractTasks as extractTasksCmd,
  indexFile,
  listCursorTranscripts,
  listSources,
  removeSource as removeSourceCmd,
  retrySource as retrySourceCmd,
  runInsightsAll as runInsightsAllCmd,
  summarizeSource as summarizeSourceCmd,
  syncCursorTranscripts as syncCursorTranscriptsCmd,
} from "../lib/tauri";
import type { CursorTranscriptSummary, Source } from "../types/library";

export interface UseLibraryOptions {
  onError?: (message: string) => void;
}

export function useLibrary({ onError }: UseLibraryOptions = {}) {
  const [sources, setSources] = useState<Source[]>([]);
  const [cursorCandidates, setCursorCandidates] = useState<
    CursorTranscriptSummary[]
  >([]);

  const reportError = useCallback(
    (error: unknown) => {
      onError?.(String(error));
    },
    [onError],
  );

  const refreshSources = useCallback(async () => {
    const list = await listSources();
    setSources(list);
    return list;
  }, []);

  const refreshCursorCandidates = useCallback(async () => {
    const list = await listCursorTranscripts();
    setCursorCandidates(list);
    return list;
  }, []);

  const retrySource = useCallback(
    async (id: string) => {
      try {
        await retrySourceCmd(id);
        await refreshSources();
      } catch (error) {
        reportError(error);
      }
    },
    [refreshSources, reportError],
  );

  const removeSource = useCallback(
    async (id: string) => {
      try {
        await removeSourceCmd(id);
        await refreshSources();
      } catch (error) {
        reportError(error);
      }
    },
    [refreshSources, reportError],
  );

  const summarizeSource = useCallback(
    async (sourceId: string) => {
      try {
        await summarizeSourceCmd(sourceId);
        await refreshSources();
      } catch (error) {
        reportError(error);
      }
    },
    [refreshSources, reportError],
  );

  const extractTasks = useCallback(
    async (sourceId: string) => {
      try {
        await extractTasksCmd(sourceId);
        await refreshSources();
      } catch (error) {
        reportError(error);
      }
    },
    [refreshSources, reportError],
  );

  const runInsightsAll = useCallback(async () => {
    try {
      await runInsightsAllCmd(true, true);
      await refreshSources();
    } catch (error) {
      reportError(error);
    }
  }, [refreshSources, reportError]);

  const pickAndIndex = useCallback(async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Documents", extensions: ["txt", "md", "markdown"] }],
      });
      if (selected === null) return;
      await indexFile(selected);
      await refreshSources();
    } catch (error) {
      reportError(error);
    }
  }, [refreshSources, reportError]);

  const syncCursorTranscripts = useCallback(async () => {
    try {
      await syncCursorTranscriptsCmd();
      await refreshSources();
      await refreshCursorCandidates();
    } catch (error) {
      reportError(error);
    }
  }, [refreshCursorCandidates, refreshSources, reportError]);

  return {
    sources,
    cursorCandidates,
    refreshSources,
    refreshCursorCandidates,
    retrySource,
    removeSource,
    summarizeSource,
    extractTasks,
    runInsightsAll,
    pickAndIndex,
    syncCursorTranscripts,
  };
}
