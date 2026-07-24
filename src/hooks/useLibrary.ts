import { useCallback, useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";

import {
  compileWiki as compileWikiCmd,
  exportWikiZip as exportWikiZipCmd,
  extractTasks as extractTasksCmd,
  indexFile,
  listCursorTranscripts,
  listSources,
  removeSource as removeSourceCmd,
  retrySource as retrySourceCmd,
  runInsightsAll as runInsightsAllCmd,
  summarizeSource as summarizeSourceCmd,
  syncCursorTranscripts as syncCursorTranscriptsCmd,
  wikiExportPreflight,
} from "../lib/tauri";
import type { CursorTranscriptSummary, Source } from "../types/library";

export interface UseLibraryOptions {
  onError?: (message: string) => void;
}

export interface ExportWikiOptions {
  onSuccess?: (path: string) => void;
}

function localYmd(): string {
  const d = new Date();
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
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

  const compileWiki = useCallback(
    async (sourceId: string) => {
      try {
        await compileWikiCmd(sourceId);
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

  const exportWiki = useCallback(
    async ({ onSuccess }: ExportWikiOptions = {}) => {
      try {
        const preflight = await wikiExportPreflight();
        if (!preflight.hasNotes) {
          reportError("还没有可导出的笔记，请先生成笔记");
          return;
        }
        const e2ePath =
          typeof window !== "undefined"
            ? (window as Window & { __JARVIS_E2E_WIKI_EXPORT_PATH__?: string })
                .__JARVIS_E2E_WIKI_EXPORT_PATH__
            : undefined;
        const destPath =
          e2ePath ??
          (await save({
            defaultPath: `jarvis-wiki-${localYmd()}.zip`,
            filters: [{ name: "Zip", extensions: ["zip"] }],
          }));
        if (destPath === null || destPath === undefined) return;
        await exportWikiZipCmd(destPath);
        onSuccess?.(destPath);
        return destPath;
      } catch (error) {
        reportError(error);
      }
    },
    [reportError],
  );

  return {
    sources,
    cursorCandidates,
    refreshSources,
    refreshCursorCandidates,
    retrySource,
    removeSource,
    summarizeSource,
    extractTasks,
    compileWiki,
    runInsightsAll,
    pickAndIndex,
    syncCursorTranscripts,
    exportWiki,
  };
}
