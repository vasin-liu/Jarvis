/**
 * @vitest-environment jsdom
 */
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { sourceKindLabel } from "../lib/sourceDisplay";
import type { AppConfig } from "../types/ipc";
import { LibraryView } from "./LibraryView";

vi.mock("motion/react", () => ({
  motion: {
    section: ({
      children,
      ...props
    }: React.PropsWithChildren<Record<string, unknown>>) => (
      <section {...props}>{children}</section>
    ),
    div: ({
      children,
      ...props
    }: React.PropsWithChildren<Record<string, unknown>>) => (
      <div {...props}>{children}</div>
    ),
  },
  useReducedMotion: () => true,
}));

const listRelatedSources = vi.fn();

vi.mock("../lib/tauri", () => ({
  listRelatedSources: (...args: unknown[]) => listRelatedSources(...args),
}));

const indexedLocal = {
  id: "src-1",
  kind: "local_file",
  uri: "file:///tmp/a.md",
  title: "a.md",
  status: "indexed",
  indexed_at: 1,
  error: null,
  summary: null,
};

const indexedWikiPage = {
  id: "wiki-1",
  kind: "wiki_page",
  uri: "wiki://page/a",
  title: "笔记 a",
  status: "indexed",
  indexed_at: 1,
  error: null,
  summary: null,
};

const baseProps = {
  sources: [indexedLocal],
  tasks: [],
  busy: false,
  indexProgress: null,
  config: { wiki: { enabled: true, auto_on_insights: false } } as AppConfig,
  cursorCandidates: [],
  onRunInsightsAll: vi.fn(),
  onPickAndIndex: vi.fn(),
  onSyncCursorTranscripts: vi.fn(),
  onSummarizeSource: vi.fn(),
  onExtractTasks: vi.fn(),
  onCompileWiki: vi.fn(),
  onExportWiki: vi.fn(),
  onRetrySource: vi.fn(),
  onRemoveSource: vi.fn(),
};

describe("LibraryView wiki compile", () => {
  beforeEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  it("shows 生成笔记 when wiki.enabled and indexed local_file", () => {
    render(<LibraryView {...baseProps} />);
    const btn = screen.getByTestId("wiki-compile-src-1");
    expect(btn).toBeTruthy();
    expect(btn.textContent).toContain("生成笔记");
  });

  it("hides compile when wiki.enabled is false", () => {
    render(
      <LibraryView
        {...baseProps}
        config={{ wiki: { enabled: false, auto_on_insights: false } } as AppConfig}
      />,
    );
    expect(screen.queryByTestId("wiki-compile-src-1")).toBeNull();
  });

  it("hides compile when wiki is missing", () => {
    render(<LibraryView {...baseProps} config={{} as AppConfig} />);
    expect(screen.queryByTestId("wiki-compile-src-1")).toBeNull();
  });

  it("hides compile for indexed wiki_page even when enabled", () => {
    render(
      <LibraryView
        {...baseProps}
        sources={[indexedWikiPage]}
        config={{ wiki: { enabled: true, auto_on_insights: false } } as AppConfig}
      />,
    );
    expect(screen.queryByTestId("wiki-compile-wiki-1")).toBeNull();
  });

  it("hides retry for wiki_page", () => {
    render(
      <LibraryView
        {...baseProps}
        sources={[indexedWikiPage]}
        config={{ wiki: { enabled: true, auto_on_insights: false } } as AppConfig}
      />,
    );
    expect(screen.queryByTestId("retry-source-wiki-1")).toBeNull();
  });

  it("shows retry for local_file", () => {
    render(<LibraryView {...baseProps} />);
    expect(screen.getByTestId("retry-source-src-1")).toBeTruthy();
  });

  it("still lists wiki_page rows when wiki is disabled", () => {
    render(
      <LibraryView
        {...baseProps}
        sources={[indexedWikiPage]}
        config={{ wiki: { enabled: false, auto_on_insights: false } } as AppConfig}
      />,
    );
    expect(screen.getByText("笔记 a")).toBeTruthy();
    expect(screen.getByText(/笔记页/)).toBeTruthy();
    expect(screen.queryByTestId("wiki-compile-wiki-1")).toBeNull();
  });

  it("clicking 生成笔记 calls onCompileWiki with source id", () => {
    render(<LibraryView {...baseProps} />);
    fireEvent.click(screen.getByTestId("wiki-compile-src-1"));
    expect(baseProps.onCompileWiki).toHaveBeenCalledWith("src-1");
  });
});

describe("LibraryView wiki export", () => {
  beforeEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  it("shows 导出 Wiki when wiki.enabled", () => {
    render(<LibraryView {...baseProps} />);
    const btn = screen.getByTestId("wiki-export");
    expect(btn).toBeTruthy();
    expect(btn.textContent).toContain("导出 Wiki");
  });

  it("places wiki-export after 选择文件索引 in toolbar", () => {
    render(<LibraryView {...baseProps} />);
    const exportBtn = screen.getByTestId("wiki-export");
    const fileIndexBtn = screen.getByText("选择文件索引");
    const toolbar = exportBtn.parentElement;
    expect(toolbar).toBeTruthy();
    const buttons = Array.from(toolbar!.querySelectorAll("button"));
    const fileIdx = buttons.indexOf(fileIndexBtn as HTMLButtonElement);
    const exportIdx = buttons.indexOf(exportBtn as HTMLButtonElement);
    expect(fileIdx).toBeGreaterThanOrEqual(0);
    expect(exportIdx).toBeGreaterThan(fileIdx);
  });

  it("hides wiki-export when wiki.enabled is false", () => {
    render(
      <LibraryView
        {...baseProps}
        config={{ wiki: { enabled: false, auto_on_insights: false } } as AppConfig}
      />,
    );
    expect(screen.queryByTestId("wiki-export")).toBeNull();
  });

  it("hides wiki-export when wiki is missing", () => {
    render(<LibraryView {...baseProps} config={{} as AppConfig} />);
    expect(screen.queryByTestId("wiki-export")).toBeNull();
  });

  it("disables wiki-export when busy", () => {
    render(<LibraryView {...baseProps} busy />);
    expect(
      (screen.getByTestId("wiki-export") as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  it("clicking wiki-export calls onExportWiki", () => {
    render(<LibraryView {...baseProps} />);
    fireEvent.click(screen.getByTestId("wiki-export"));
    expect(baseProps.onExportWiki).toHaveBeenCalled();
  });
});

const pendingLocal = {
  id: "src-pending",
  kind: "local_file",
  uri: "file:///tmp/pending.md",
  title: "pending.md",
  status: "pending",
  indexed_at: null,
  error: null,
  summary: null,
};

const neighborLocal = {
  id: "src-2",
  kind: "local_file",
  uri: "file:///tmp/b.md",
  title: "b.md",
  status: "indexed",
  indexed_at: 2,
  error: null,
  summary: null,
};

describe("LibraryView related docs", () => {
  beforeEach(() => {
    cleanup();
    vi.clearAllMocks();
    listRelatedSources.mockReset();
    listRelatedSources.mockResolvedValue([]);
  });

  afterEach(() => {
    cleanup();
  });

  it("hides related panel when no source is selected (D-02)", () => {
    render(<LibraryView {...baseProps} />);
    expect(screen.queryByTestId("related-docs-panel")).toBeNull();
  });

  it("selecting indexed source shows panel and empty copy when no neighbors (D-10, D-12)", async () => {
    listRelatedSources.mockResolvedValue([]);
    render(<LibraryView {...baseProps} />);
    fireEvent.click(screen.getByTestId("source-row-src-1"));
    expect(screen.getByTestId("related-docs-panel")).toBeTruthy();
    expect(screen.getByTestId("related-docs-loading").textContent).toContain(
      "加载相关文档…",
    );
    await waitFor(() => {
      expect(screen.getByTestId("related-docs-empty").textContent).toBe(
        "暂无相关文档",
      );
    });
    expect(listRelatedSources).toHaveBeenCalledWith("src-1", undefined);
  });

  it("non-indexed selection shows D-11 copy and skips IPC (D-06)", async () => {
    render(
      <LibraryView {...baseProps} sources={[indexedLocal, pendingLocal]} />,
    );
    fireEvent.click(screen.getByTestId("source-row-src-pending"));
    expect(screen.getByTestId("related-docs-empty").textContent).toBe(
      "仅已索引来源可查看相关文档",
    );
    expect(listRelatedSources).not.toHaveBeenCalled();
  });

  it("shows error copy when listRelatedSources rejects (D-12)", async () => {
    listRelatedSources.mockRejectedValue(new Error("boom"));
    render(<LibraryView {...baseProps} />);
    fireEvent.click(screen.getByTestId("source-row-src-1"));
    await waitFor(() => {
      expect(screen.getByTestId("related-docs-error").textContent).toContain(
        "相关文档加载失败",
      );
    });
  });

  it("ready rows show title, kind label, snippet without scores (D-09)", async () => {
    listRelatedSources.mockResolvedValue([
      {
        sourceId: "src-2",
        title: "Neighbor Doc",
        kind: "local_file",
        snippet: "overlap snippet xyzzy",
      },
    ]);
    render(
      <LibraryView {...baseProps} sources={[indexedLocal, neighborLocal]} />,
    );
    fireEvent.click(screen.getByTestId("source-row-src-1"));
    await waitFor(() => {
      expect(screen.getByTestId("related-docs-list")).toBeTruthy();
    });
    const row = screen.getByTestId("related-docs-row-src-2");
    expect(row.textContent).toContain("Neighbor Doc");
    expect(row.textContent).toContain(sourceKindLabel("local_file"));
    expect(row.textContent).toContain("overlap snippet xyzzy");
    expect(row.textContent).not.toMatch(/score|affinity|%/i);
  });

  it("second click on same source clears selection and unmounts panel (D-08)", async () => {
    render(<LibraryView {...baseProps} />);
    const row = screen.getByTestId("source-row-src-1");
    fireEvent.click(row);
    expect(screen.getByTestId("related-docs-panel")).toBeTruthy();
    fireEvent.click(row);
    expect(screen.queryByTestId("related-docs-panel")).toBeNull();
  });

  it("action button click does not select source (D-05)", () => {
    render(<LibraryView {...baseProps} />);
    fireEvent.click(screen.getByTestId("summarize-src-1"));
    expect(screen.queryByTestId("related-docs-panel")).toBeNull();
    expect(baseProps.onSummarizeSource).toHaveBeenCalledWith("src-1");
  });

  it("clicking related row selects neighbor and refetches (D-13)", async () => {
    listRelatedSources
      .mockResolvedValueOnce([
        {
          sourceId: "src-2",
          title: "Neighbor Doc",
          kind: "local_file",
          snippet: "overlap",
        },
      ])
      .mockResolvedValueOnce([]);
    render(
      <LibraryView {...baseProps} sources={[indexedLocal, neighborLocal]} />,
    );
    fireEvent.click(screen.getByTestId("source-row-src-1"));
    await waitFor(() => {
      expect(screen.getByTestId("related-docs-row-src-2")).toBeTruthy();
    });
    fireEvent.click(screen.getByTestId("related-docs-row-src-2"));
    const neighborRow = screen.getByTestId("source-row-src-2");
    expect(neighborRow.getAttribute("aria-selected")).toBe("true");
    expect(neighborRow.getAttribute("data-selected")).toBe("true");
    await waitFor(() => {
      expect(listRelatedSources).toHaveBeenCalledWith("src-2", undefined);
    });
  });
});
