import { spawn, type ChildProcess } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.join(root, "..", "..");

const msedgedriverPath =
  process.env.MSEDGEDRIVER_PATH ?? path.join(repoRoot, "msedgedriver.exe");

const e2eFixture =
  process.env.JARVIS_E2E_FIXTURE ??
  path.join(repoRoot, "e2e/fixtures/sample.md");

export default async function globalSetup() {
  process.env.JARVIS_E2E = process.env.JARVIS_E2E ?? "1";
  process.env.JARVIS_E2E_FIXTURE = e2eFixture;

  const driverPath = path.resolve(
    os.homedir(),
    ".cargo",
    "bin",
    process.platform === "win32" ? "tauri-driver.exe" : "tauri-driver",
  );

  const child = spawn(driverPath, ["--native-driver", msedgedriverPath], {
    stdio: [null, "pipe", "pipe"],
    env: {
      ...process.env,
      JARVIS_E2E: "1",
      JARVIS_E2E_FIXTURE: e2eFixture,
    },
  });

  child.on("error", (error) => {
    console.error("tauri-driver error:", error);
    process.exit(1);
  });

  (globalThis as typeof globalThis & { __JARVIS_TAURI_DRIVER__?: ChildProcess }).__JARVIS_TAURI_DRIVER__ =
    child;

  await new Promise((resolve) => setTimeout(resolve, 5_000));
}
