import type { ReactNode } from "react";
import {
  IconBooks,
  IconBrain,
  IconListCheck,
  IconMessageChatbot,
  IconSettings,
} from "@tabler/icons-react";

import type { View } from "./types/view";

export interface AppShellProps {
  view: View;
  setView: (view: View) => void;
  busy: boolean;
  err: string | null;
  setErr: (err: string | null) => void;
  sourceCount: number;
  e2eMode?: boolean;
  askHandleCount?: number;
  children: ReactNode;
}

export function AppShell({
  view,
  setView,
  busy,
  err,
  setErr,
  sourceCount,
  e2eMode = false,
  askHandleCount = 0,
  children,
}: AppShellProps) {
  return (
    <div className="flex min-h-screen" data-testid="app-root">
      {e2eMode && <div data-testid="e2e-active" className="hidden" />}
      {e2eMode && (
        <div
          data-testid="ask-handle-count"
          data-count={askHandleCount}
          className="hidden"
        />
      )}

      <aside className="glass-panel m-4 flex w-56 shrink-0 flex-col gap-2 p-4">
        <div className="mb-2 px-2">
          <div className="text-xs uppercase tracking-widest text-cyan-300/80">
            Jarvis
          </div>
          <h1 className="text-lg font-semibold text-zinc-50">知识中枢</h1>
        </div>
        {(
          [
            ["chat", "对话", IconMessageChatbot],
            ["library", "资料库", IconBooks],
            ["tasks", "任务", IconListCheck],
            ["memory", "记忆", IconBrain],
            ["settings", "设置", IconSettings],
          ] as const
        ).map(([id, label, Icon]) => (
          <button
            key={id}
            type="button"
            data-testid={`nav-${id}`}
            className={`nav-btn flex items-center gap-2 ${view === id ? "nav-btn-active" : "nav-btn-idle"}`}
            onClick={() => setView(id)}
          >
            <Icon className="size-4 shrink-0 opacity-80" aria-hidden />
            {label}
          </button>
        ))}
        <div className="mt-auto px-2 pt-4 text-xs text-zinc-500">
          来源 {sourceCount} · v9
        </div>
      </aside>

      <main className="relative flex flex-1 flex-col gap-4 p-4 pl-0">
        {err && (
          <div
            role="alert"
            className="glass-panel border-red-400/30 bg-red-950/40 px-4 py-3 text-sm text-red-200"
          >
            <div className="flex items-start justify-between gap-3">
              <span>{err}</span>
              <button
                type="button"
                className="btn-ghost shrink-0 px-2 py-0.5 text-xs"
                onClick={() => setErr(null)}
              >
                关闭
              </button>
            </div>
          </div>
        )}

        {children}

        {busy && (
          <div
            className="pointer-events-none absolute inset-0 z-50 flex items-center justify-center bg-zinc-950/40 backdrop-blur-[2px]"
            data-testid="busy-overlay"
            aria-hidden
          >
            <div className="size-8 animate-spin rounded-full border-2 border-cyan-400/30 border-t-cyan-400" />
          </div>
        )}
      </main>
    </div>
  );
}
