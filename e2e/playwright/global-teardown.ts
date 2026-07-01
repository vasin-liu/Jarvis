import type { ChildProcess } from "node:child_process";

export default async function globalTeardown() {
  const child = (
    globalThis as typeof globalThis & { __JARVIS_TAURI_DRIVER__?: ChildProcess }
  ).__JARVIS_TAURI_DRIVER__;
  child?.kill();
}
