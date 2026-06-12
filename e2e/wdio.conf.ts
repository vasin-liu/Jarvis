import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(fileURLToPath(import.meta.url));

export const config = {
  runner: "local",
  specs: [path.join(root, "specs/**/*.ts")],
  maxInstances: 1,
  capabilities: [
    {
      browserName: "msedge",
      "tauri:options": {
        application:
          process.env.JARVIS_E2E_APP ??
          path.join(root, "../src-tauri/target/release/tauri-app.exe"),
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
};
