import { test as base, expect } from "@playwright/test";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { remote, type Browser } from "webdriverio";

const root = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.join(root, "..", "..");

const appBinary =
  process.env.JARVIS_E2E_APP ??
  path.join(repoRoot, "target/release/tauri-app.exe");

type WdioFixtures = {
  wdio: Browser;
};

export const test = base.extend<WdioFixtures>({
  wdio: async ({}, use) => {
    const browser = await remote({
      hostname: "127.0.0.1",
      port: 4444,
      path: "/",
      capabilities: {
        "tauri:options": {
          application: appBinary,
        },
      },
      connectionRetryTimeout: 120_000,
      waitforTimeout: 15_000,
    });

    await browser.waitUntil(
      async () => {
        try {
          return await (
            await browser.$('[data-testid="app-root"]')
          ).isDisplayed();
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

    await use(browser);
    await browser.deleteSession();
  },
});

export { expect };
