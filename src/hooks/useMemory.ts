import { useCallback, useRef, useState } from "react";

import {
  addMemory as addMemoryCmd,
  forgetMemory as forgetMemoryCmd,
  getMemoryContent,
  listMemories,
  updateMemory as updateMemoryCmd,
} from "../lib/tauri";
import type { MemorySource } from "../types/memory";

export interface UseMemoryOptions {
  onError?: (message: string) => void;
}

export function useMemory({ onError }: UseMemoryOptions = {}) {
  const [memories, setMemories] = useState<MemorySource[]>([]);
  const [newMemoryText, setNewMemoryText] = useState("");
  const [editingMemoryId, setEditingMemoryId] = useState<string | null>(null);
  const [editingMemoryContent, setEditingMemoryContent] = useState("");
  const [editingMemoryTitle, setEditingMemoryTitle] = useState("");
  const memoryEditContentRef = useRef<HTMLTextAreaElement>(null);

  const reportError = useCallback(
    (error: unknown) => {
      onError?.(String(error));
    },
    [onError],
  );

  const refreshMemories = useCallback(async () => {
    const list = await listMemories();
    setMemories(list);
    return list;
  }, []);

  const addMemory = useCallback(async () => {
    const text = newMemoryText.trim();
    if (!text) return;
    try {
      await addMemoryCmd(text, null);
      setNewMemoryText("");
      await refreshMemories();
    } catch (error) {
      reportError(error);
      throw error;
    }
  }, [newMemoryText, refreshMemories, reportError]);

  const startEdit = useCallback(
    async (id: string) => {
      try {
        const content = await getMemoryContent(id);
        const memory = memories.find((m) => m.id === id);
        setEditingMemoryId(id);
        setEditingMemoryContent(content);
        const title = memory?.title?.replace(/^记忆:\s*/, "") ?? "";
        setEditingMemoryTitle(title);
      } catch (error) {
        reportError(error);
        throw error;
      }
    },
    [memories, reportError],
  );

  const saveEdit = useCallback(async () => {
    if (!editingMemoryId) return;
    const content = (
      memoryEditContentRef.current?.value ?? editingMemoryContent
    ).trim();
    if (!content) {
      reportError("记忆内容不能为空");
      return;
    }
    try {
      await updateMemoryCmd(editingMemoryId, content, null);
      setEditingMemoryId(null);
      setEditingMemoryContent("");
      setEditingMemoryTitle("");
      await refreshMemories();
    } catch (error) {
      reportError(error);
      throw error;
    }
  }, [editingMemoryContent, editingMemoryId, refreshMemories, reportError]);

  const cancelEdit = useCallback(() => {
    setEditingMemoryId(null);
    setEditingMemoryContent("");
    setEditingMemoryTitle("");
  }, []);

  const forgetMemory = useCallback(
    async (id: string) => {
      try {
        await forgetMemoryCmd(id);
        if (editingMemoryId === id) {
          setEditingMemoryId(null);
          setEditingMemoryContent("");
          setEditingMemoryTitle("");
        }
        await refreshMemories();
      } catch (error) {
        reportError(error);
        throw error;
      }
    },
    [editingMemoryId, refreshMemories, reportError],
  );

  return {
    memories,
    newMemoryText,
    setNewMemoryText,
    editingMemoryId,
    editingMemoryContent,
    setEditingMemoryContent,
    editingMemoryTitle,
    setEditingMemoryTitle,
    memoryEditContentRef,
    refreshMemories,
    addMemory,
    startEdit,
    saveEdit,
    cancelEdit,
    forgetMemory,
  };
}
