/**
 * Wiki journey (WIKI-08 / WIKI-09): default-off Library controls, then
 * enable → compile →「笔记页」→ export zip → citation URI trust.
 */
import fs from "node:fs";
import {
  askQuestion,
  assertWikiZipNonEmpty,
  clickViaDom,
  expandSettingsSection,
  openNav,
  setReactCheckbox,
  setWikiExportPath,
  wikiE2eZipPath,
} from "../helpers.ts";

async function waitBusyClear(timeoutMs = 30_000) {
  await browser.waitUntil(
    async () => (await $$('[data-testid="busy-overlay"]')).length === 0,
    {
      timeout: timeoutMs,
      timeoutMsg: "busy-overlay still present",
    },
  );
}

async function refreshAppConfig() {
  await browser.execute(async () => {
    const w = window as Window & {
      __JARVIS_E2E_REFRESH_CONFIG__?: () => Promise<void>;
    };
    await w.__JARVIS_E2E_REFRESH_CONFIG__?.();
  });
}

/** Click first wiki-compile-* via DOM — ids may be Windows paths with \\?\ that break CSS selectors. */
async function clickFirstWikiCompile() {
  const clicked = await browser.execute(() => {
    const el = document.querySelector(
      '[data-testid^="wiki-compile-"]',
    ) as HTMLElement | null;
    el?.click();
    return el?.getAttribute("data-testid") ?? null;
  });
  if (!clicked) {
    throw new Error("no wiki-compile-* button to click");
  }
}

describe("Jarvis wiki journey", () => {
  it("hides wiki Library controls when disabled (default-off)", async () => {
    await openNav("library", '[data-testid="library-stats"]');
    expect(await $$('[data-testid="wiki-export"]')).toHaveLength(0);
    expect(await $$('[data-testid^="wiki-compile-"]')).toHaveLength(0);
  });

  it("enable → compile → 笔记页 → export → citation trust", async () => {
    await openNav("library", '[data-testid="library-stats"]');
    await browser.waitUntil(
      async () =>
        (await $('[data-testid="library-stats"]').getText()).match(
          /已索引 [1-9]/,
        ) !== null,
      {
        timeout: 60_000,
        timeoutMsg: "seeded fixture not indexed yet",
      },
    );

    await openNav("settings", '[data-testid="settings-panel"]');
    await expandSettingsSection("settings-section-wiki");
    const toggle = await $('[data-testid="wiki-enabled-toggle"]');
    await setReactCheckbox(toggle, true);
    await expect(toggle).toBeSelected();
    await browser.pause(200);
    await clickViaDom('[data-testid="settings-save-config"]');
    await waitBusyClear();
    await refreshAppConfig();

    await openNav("library", '[data-testid="source-list"]');
    await $('[data-testid="wiki-export"]').waitForDisplayed({
      timeout: 30_000,
    });
    await $('[data-testid^="wiki-compile-"]').waitForDisplayed({
      timeout: 30_000,
    });
    await clickFirstWikiCompile();
    await browser.waitUntil(
      async () => (await $$('[data-testid="busy-overlay"]')).length > 0,
      {
        timeout: 10_000,
        timeoutMsg: "compile click did not start busy overlay",
      },
    );
    await waitBusyClear(90_000);
    await browser.waitUntil(
      async () =>
        (await $('[data-testid="source-list"]').getText()).includes("笔记页"),
      {
        timeout: 90_000,
        timeoutMsg: "wiki_page not listed after compile",
      },
    );

    const zipPath = wikiE2eZipPath();
    if (fs.existsSync(zipPath)) {
      fs.unlinkSync(zipPath);
    }
    await setWikiExportPath(zipPath);
    await clickViaDom('[data-testid="wiki-export"]');
    await $('[data-testid="wiki-export-done"]').waitForDisplayed({
      timeout: 60_000,
    });
    assertWikiZipNonEmpty(zipPath);

    const answer = await askQuestion("What is xyzzy-plugh?");
    expect((await answer.getText()).length).toBeGreaterThan(0);
    const uris = await browser.execute(() =>
      [...document.querySelectorAll("[data-source-uri]")].map((e) =>
        e.getAttribute("data-source-uri"),
      ),
    );
    expect(uris.some((u) => !!u && !u.startsWith("wiki://"))).toBe(true);

    // Reduce shared-session pollution for later specs (Pitfall 3)
    await openNav("settings", '[data-testid="settings-panel"]');
    await expandSettingsSection("settings-section-wiki");
    await setReactCheckbox(
      await $('[data-testid="wiki-enabled-toggle"]'),
      false,
    );
    await clickViaDom('[data-testid="settings-save-config"]');
    await waitBusyClear();
    await refreshAppConfig();
  });
});
