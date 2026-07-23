/**
 * @vitest-environment jsdom
 */
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

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
  },
  useReducedMotion: () => true,
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
