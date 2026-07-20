import { spawn } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.join(root, "..");

const appBinary =
  process.env.JARVIS_E2E_APP ??
  path.join(repoRoot, "target/release/tauri-app.exe");

const msedgedriverPath =
  process.env.MSEDGEDRIVER_PATH ??
  path.join(repoRoot, "msedgedriver.exe");

const e2eFixture =
  process.env.JARVIS_E2E_FIXTURE ??
  path.join(repoRoot, "e2e/fixtures/sample.md");

let tauriDriver: ReturnType<typeof spawn> | undefined;
let shuttingDown = false;

function closeTauriDriver() {
  shuttingDown = true;
  tauriDriver?.kill();
}

process.on("exit", closeTauriDriver);
process.on("SIGINT", closeTauriDriver);

export const config = {
  runner: "local",
  host: "127.0.0.1",
  port: 4444,
  path: "/",
  specs: [path.join(root, "specs/**/*.spec.ts")],
  maxInstances: 1,
  waitforTimeout: 15_000,
  connectionRetryTimeout: 120_000,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        application: appBinary,
      },
    },
  ],
  logLevel: "info",
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: {
    ui: "bdd",
    timeout: 180_000,
  },
  onPrepare: () => {
    process.env.JARVIS_E2E = process.env.JARVIS_E2E ?? "1";
    process.env.JARVIS_E2E_FIXTURE = e2eFixture;
  },
  beforeSession: async () => {
    const driverPath = path.resolve(
      os.homedir(),
      ".cargo",
      "bin",
      process.platform === "win32" ? "tauri-driver.exe" : "tauri-driver",
    );
    tauriDriver = spawn(
      driverPath,
      ["--native-driver", msedgedriverPath],
      {
        stdio: [null, process.stdout, process.stderr],
        env: {
          ...process.env,
          JARVIS_E2E: "1",
          JARVIS_E2E_FIXTURE: e2eFixture,
        },
      },
    );
    tauriDriver.on("error", (error) => {
      console.error("tauri-driver error:", error);
      process.exit(1);
    });
    tauriDriver.on("exit", (code) => {
      if (!shuttingDown) {
        console.error("tauri-driver exited with code:", code);
        process.exit(1);
      }
    });
    // Allow tauri-driver + native app to finish booting before WebDriver connects.
    await new Promise((resolve) => setTimeout(resolve, 5_000));
  },
  afterSession: () => {
    closeTauriDriver();
  },
  before: async () => {
    await browser.waitUntil(
      async () => {
        try {
          return await (await $('[data-testid="app-root"]')).isDisplayed();
        } catch {
          return false;
        }
      },
      {
        timeout: 120_000,
        interval: 500,
        timeoutMsg: "Jarvis UI did not load (app-root missing)",
      },
    );
    await browser.waitUntil(
      async () => {
        try {
          return await (await $('[data-testid="nav-chat"]')).isDisplayed();
        } catch {
          return false;
        }
      },
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: "Jarvis navigation did not load (nav-chat missing)",
      },
    );
  },
};
