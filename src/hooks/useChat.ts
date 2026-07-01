import { useCallback, useEffect, useRef, useState } from "react";
import { flushSync } from "react-dom";
import { Channel, invoke } from "@tauri-apps/api/core";

import {
  askInSession,
  askInSessionStream,
  createChatSession,
  deleteChatSession,
  listChatMessages,
  listChatSessions,
} from "../lib/tauri";
import type { JarvisAskDonePayload } from "./useAppEvents";
import type { ChatMessage, ChatSession } from "../types/chat";
import type {
  AskResponse,
  OrchestrationStepInfo,
  ToolCallInfo,
} from "../types/ipc";

export interface UseChatOptions {
  onError?: (message: string | null) => void;
  setBusy?: (value: boolean) => void;
  listenJarvisAskDone?: (
    handler: (payload: JarvisAskDonePayload) => void,
  ) => Promise<() => void>;
  onAfterAsk?: () => void | Promise<void>;
}

export function useChat({
  onError,
  setBusy,
  listenJarvisAskDone,
  onAfterAsk,
}: UseChatOptions = {}) {
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [question, setQuestion] = useState("");
  const [streamingDraft, setStreamingDraft] = useState("");
  const [lastToolCalls, setLastToolCalls] = useState<ToolCallInfo[]>([]);
  const [lastToolParseWarnings, setLastToolParseWarnings] = useState<string[]>(
    [],
  );
  const [lastOrchestrationSteps, setLastOrchestrationSteps] = useState<
    OrchestrationStepInfo[]
  >([]);
  const [agentMode, setAgentMode] = useState(true);
  const [e2eMode, setE2eMode] = useState(false);
  const [askHandleCount, setAskHandleCount] = useState(0);
  const handleAskRef = useRef<() => void>(() => {});

  const reportError = useCallback(
    (error: unknown) => {
      onError?.(String(error));
    },
    [onError],
  );

  const refreshSessions = useCallback(async () => {
    const list = await listChatSessions();
    setSessions(list);
    return list;
  }, []);

  const refreshMessages = useCallback(async (sessionId: string) => {
    const list = await listChatMessages(sessionId);
    setMessages(list);
  }, []);

  const ensureSession = useCallback(async () => {
    let list = await refreshSessions();
    if (list.length === 0) {
      const created = await createChatSession(null);
      list = [created];
      setSessions(list);
    }
    const id = list[0].id;
    setActiveSessionId(id);
    await refreshMessages(id);
    return id;
  }, [refreshMessages, refreshSessions]);

  useEffect(() => {
    ensureSession().catch(reportError);
    invoke<boolean>("is_e2e_mode_cmd")
      .then(setE2eMode)
      .catch(() => setE2eMode(false));
  }, [ensureSession, reportError]);

  useEffect(() => {
    if (activeSessionId) {
      refreshMessages(activeSessionId).catch(reportError);
    }
  }, [activeSessionId, refreshMessages, reportError]);

  const handleRagAsk = useCallback(async () => {
    const q = question.trim();
    if (!q || !activeSessionId) return;
    if (e2eMode) {
      setAskHandleCount((c) => c + 1);
    }
    flushSync(() => {
      onError?.(null);
      setBusy?.(true);
      setStreamingDraft("");
    });
    try {
      if (e2eMode && listenJarvisAskDone) {
        await new Promise<AskResponse>(async (resolve, reject) => {
          let unlisten: (() => void) | undefined;
          const timer = window.setTimeout(() => {
            unlisten?.();
            reject(new Error("E2E ask timed out waiting for assistant reply"));
          }, 60_000);
          try {
            unlisten = await listenJarvisAskDone((event) => {
              if (event.session_id !== activeSessionId) return;
              window.clearTimeout(timer);
              unlisten?.();
              if (event.error) {
                reject(new Error(event.error));
                return;
              }
              const lastAssistant = event.messages
                .filter((m) => m.role === "assistant")
                .at(-1);
              flushSync(() => {
                setMessages(event.messages);
                setQuestion("");
                setStreamingDraft("");
              });
              resolve({
                answer: lastAssistant?.content ?? "",
                citations: [],
              });
            });
            await invoke("start_ask_e2e", {
              sessionId: activeSessionId,
              question: q,
              agentMode: false,
            });
          } catch (err) {
            window.clearTimeout(timer);
            unlisten?.();
            reject(err);
          }
        });
      } else {
        const useE2e = await invoke<boolean>("is_e2e_mode_cmd");
        if (useE2e !== e2eMode) {
          setE2eMode(useE2e);
        }
        if (useE2e) {
          await askInSession(activeSessionId, q);
        } else {
          const channel = new Channel<{ token: string }>();
          channel.onmessage = (msg) => {
            setStreamingDraft((prev) => prev + msg.token);
          };
          await askInSessionStream(activeSessionId, q, channel);
        }
      }
      setLastToolCalls([]);
      setLastOrchestrationSteps([]);
      if (!e2eMode) {
        setQuestion("");
        setStreamingDraft("");
        await refreshMessages(activeSessionId);
      }
      await refreshSessions();
      await onAfterAsk?.();
    } catch (error) {
      reportError(error);
    } finally {
      flushSync(() => {
        setBusy?.(false);
        setStreamingDraft("");
      });
    }
  }, [
    activeSessionId,
    e2eMode,
    listenJarvisAskDone,
    onAfterAsk,
    onError,
    question,
    refreshMessages,
    refreshSessions,
    reportError,
    setBusy,
  ]);

  const handleAgentAsk = useCallback(async () => {
    const q = question.trim();
    if (!q || !activeSessionId) return;
    if (e2eMode) {
      setAskHandleCount((c) => c + 1);
    }
    flushSync(() => {
      onError?.(null);
      setBusy?.(true);
      setStreamingDraft("");
      setLastToolParseWarnings([]);
    });
    try {
      let resp: AskResponse;
      if (e2eMode && listenJarvisAskDone) {
        resp = await new Promise<AskResponse>(async (resolve, reject) => {
          let unlisten: (() => void) | undefined;
          const timer = window.setTimeout(() => {
            unlisten?.();
            reject(new Error("E2E ask timed out waiting for assistant reply"));
          }, 60_000);
          try {
            unlisten = await listenJarvisAskDone((event) => {
              if (event.session_id !== activeSessionId) return;
              window.clearTimeout(timer);
              unlisten?.();
              if (event.error) {
                reject(new Error(event.error));
                return;
              }
              const lastAssistant = event.messages
                .filter((m) => m.role === "assistant")
                .at(-1);
              flushSync(() => {
                setMessages(event.messages);
                setQuestion("");
                setStreamingDraft("");
              });
              resolve({
                answer: lastAssistant?.content ?? "",
                citations: [],
                tool_calls: [],
                orchestration_steps: [],
              });
            });
            await invoke("start_ask_e2e", {
              sessionId: activeSessionId,
              question: q,
              agentMode: true,
            });
          } catch (err) {
            window.clearTimeout(timer);
            unlisten?.();
            reject(err);
          }
        });
      } else {
        const useE2e = await invoke<boolean>("is_e2e_mode_cmd");
        if (useE2e !== e2eMode) {
          setE2eMode(useE2e);
        }
        if (useE2e) {
          resp = await invoke<AskResponse>("ask_agent_in_session", {
            sessionId: activeSessionId,
            question: q,
          });
        } else {
          const channel = new Channel<{ token: string }>();
          channel.onmessage = (msg) => {
            setStreamingDraft((prev) => prev + msg.token);
          };
          resp = await invoke<AskResponse>("ask_agent_in_session_stream", {
            sessionId: activeSessionId,
            question: q,
            onToken: channel,
          });
        }
      }
      setLastToolCalls(resp.tool_calls ?? []);
      setLastOrchestrationSteps(resp.orchestration_steps ?? []);
      setLastToolParseWarnings(
        resp.toolParseWarnings ?? resp.tool_parse_warnings ?? [],
      );
      if (!e2eMode) {
        setQuestion("");
        setStreamingDraft("");
        await refreshMessages(activeSessionId);
      }
      await refreshSessions();
      await onAfterAsk?.();
    } catch (error) {
      reportError(error);
    } finally {
      flushSync(() => {
        setBusy?.(false);
        setStreamingDraft("");
      });
    }
  }, [
    activeSessionId,
    e2eMode,
    listenJarvisAskDone,
    onAfterAsk,
    onError,
    question,
    refreshMessages,
    refreshSessions,
    reportError,
    setBusy,
  ]);

  const handleAsk = useCallback(() => {
    if (agentMode) {
      void handleAgentAsk();
    } else {
      void handleRagAsk();
    }
  }, [agentMode, handleAgentAsk, handleRagAsk]);

  handleAskRef.current = () => {
    void handleAsk();
  };

  useEffect(() => {
    if (!e2eMode) {
      delete (window as Window & { __JARVIS_E2E_ASK__?: () => void })
        .__JARVIS_E2E_ASK__;
      return;
    }
    (window as Window & { __JARVIS_E2E_ASK__?: () => void }).__JARVIS_E2E_ASK__ =
      () => handleAskRef.current();
    return () => {
      delete (window as Window & { __JARVIS_E2E_ASK__?: () => void })
        .__JARVIS_E2E_ASK__;
    };
  }, [e2eMode]);

  const createSession = useCallback(async () => {
    try {
      const session = await createChatSession(null);
      await refreshSessions();
      setActiveSessionId(session.id);
      setMessages([]);
      setQuestion("");
      setStreamingDraft("");
    } catch (error) {
      reportError(error);
    }
  }, [refreshSessions, reportError]);

  const switchSession = useCallback((sessionId: string) => {
    setActiveSessionId(sessionId);
  }, []);

  const deleteSession = useCallback(
    async (id: string) => {
      try {
        await deleteChatSession(id);
        const list = await refreshSessions();
        if (activeSessionId === id) {
          if (list.length > 0) {
            setActiveSessionId(list[0].id);
          } else {
            const session = await createChatSession(null);
            setSessions([session]);
            setActiveSessionId(session.id);
            setMessages([]);
          }
        }
      } catch (error) {
        reportError(error);
      }
    },
    [activeSessionId, refreshSessions, reportError],
  );

  return {
    sessions,
    activeSessionId,
    messages,
    question,
    streamingDraft,
    setQuestion,
    setMessages,
    setStreamingDraft,
    createSession,
    switchSession,
    deleteSession,
    refreshMessages,
    refreshSessions,
    agentMode,
    setAgentMode,
    lastToolCalls,
    lastToolParseWarnings,
    lastOrchestrationSteps,
    e2eMode,
    askHandleCount,
    handleRagAsk,
    handleAgentAsk,
    handleAsk,
  };
}
