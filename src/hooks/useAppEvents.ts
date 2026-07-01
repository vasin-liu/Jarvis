import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import type { ChatMessage } from "../types/chat";
import type { IndexProgressView, RebuildReport } from "../types/ipc";

export interface JarvisAskDonePayload {
  session_id: string;
  error?: string | null;
  messages: ChatMessage[];
}

export interface UseAppEventsOptions {
  onIndexComplete?: (report: RebuildReport) => void;
}

export function useAppEvents({ onIndexComplete }: UseAppEventsOptions = {}) {
  const [indexProgress, setIndexProgress] = useState<IndexProgressView | null>(
    null,
  );
  const onIndexCompleteRef = useRef(onIndexComplete);
  onIndexCompleteRef.current = onIndexComplete;

  useEffect(() => {
    const unsubs: Array<() => void> = [];
    void listen<IndexProgressView>("index-progress", (event) => {
      setIndexProgress(event.payload);
    }).then((unlisten) => unsubs.push(unlisten));
    void listen<RebuildReport>("index-complete", (event) => {
      setIndexProgress(null);
      onIndexCompleteRef.current?.(event.payload);
    }).then((unlisten) => unsubs.push(unlisten));
    return () => {
      for (const unlisten of unsubs) unlisten();
    };
  }, []);

  const listenJarvisAskDone = useCallback(
    (handler: (payload: JarvisAskDonePayload) => void) =>
      listen<JarvisAskDonePayload>("jarvis-ask-done", (event) => {
        handler(event.payload);
      }),
    [],
  );

  return { indexProgress, setIndexProgress, listenJarvisAskDone };
}
