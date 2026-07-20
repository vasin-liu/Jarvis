/**
 * @vitest-environment jsdom
 */
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ChatView } from "./ChatView";

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

vi.mock("@tauri-apps/plugin-opener", () => ({
  openPath: vi.fn(),
  openUrl: vi.fn(),
}));

const baseProps = {
  sessions: [
    {
      id: "s1",
      title: "Test",
      created_at: 0,
      updated_at: 0,
    },
  ],
  activeSessionId: "s1",
  messages: [],
  question: "",
  streamingDraft: "",
  onSetQuestion: vi.fn(),
  onCreateSession: vi.fn(),
  onSwitchSession: vi.fn(),
  onDeleteSession: vi.fn(),
  agentMode: true,
  setAgentMode: vi.fn(),
  lastToolCalls: [],
  lastToolParseWarnings: ["markdown fence not allowed: ```json"],
  lastOrchestrationSteps: [],
  onRagAsk: vi.fn(),
  onAgentAsk: vi.fn(),
  onManageAgent: vi.fn(),
  onSetActiveAgent: vi.fn(),
  busy: false,
  err: null,
  setErr: vi.fn(),
  indexProgress: null,
  e2eMode: false,
  config: null,
};

describe("ChatView warnings panel", () => {
  it("renders agent-tool-parse-warnings when warnings present", () => {
    render(<ChatView {...baseProps} />);
    expect(screen.getByTestId("agent-tool-parse-warnings")).toBeTruthy();
    expect(screen.getByText(/markdown fence not allowed/)).toBeTruthy();
  });
});
