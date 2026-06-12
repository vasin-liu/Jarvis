import { spawn } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.join(root, "..");

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
  hostname: "127.0.0.1",
  port: 4444,
  path: "/",
  specs: [path.join(root, "specs/**/*.ts")],
  maxInstances: 1,
  capabilities: [
    {
      browserName: "msedge",
      "tauri:options": {
        application:
          process.env.JARVIS_E2E_APP ??
          path.join(repoRoot, "src-tauri/target/release/tauri-app.exe"),
      },
    },
  ],
  logLevel: "info",
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: {
    ui: "bdd",
    timeout: 120000,
  },
  beforeSession: () => {
    const driverPath = path.resolve(
      os.homedir(),
      ".cargo",
      "bin",
      process.platform === "win32" ? "tauri-driver.exe" : "tauri-driver",
    );
    tauriDriver = spawn(driverPath, [], {
      stdio: [null, process.stdout, process.stderr],
    });
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
  },
  afterSession: () => {
    closeTauriDriver();
  },
};
