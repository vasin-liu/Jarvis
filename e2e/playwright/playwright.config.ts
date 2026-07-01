import { defineConfig } from "@playwright/test";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  testDir: root,
  testMatch: "**/*.spec.ts",
  timeout: 180_000,
  workers: 1,
  globalSetup: path.join(root, "global-setup.ts"),
  globalTeardown: path.join(root, "global-teardown.ts"),
  reporter: [["list"]],
});
