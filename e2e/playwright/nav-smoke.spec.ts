import { expect, test } from "./fixtures.ts";

async function openNav(wdio: import("webdriverio").Browser, id: string) {
  const nav = await wdio.$(`[data-testid="nav-${id}"]`);
  await nav.waitForDisplayed({ timeout: 15_000 });
  await nav.click();
}

test("navigates all primary views and shows Settings panel", async ({ wdio }) => {
  await expect(
    await wdio.$('[data-testid="nav-chat"]'),
  ).toBeDisplayed();

  await openNav(wdio, "library");
  await expect(await wdio.$('[data-testid="library-stats"]')).toBeDisplayed();

  await openNav(wdio, "tasks");
  await expect(await wdio.$('[data-testid="task-list"]')).toBeDisplayed();

  await openNav(wdio, "memory");
  await expect(
    await wdio.$('[data-testid="new-memory-input"]'),
  ).toBeDisplayed();

  await openNav(wdio, "settings");
  await expect(
    await wdio.$('[data-testid="settings-panel"]'),
  ).toBeDisplayed();
  await expect(
    await wdio.$('[data-testid="settings-section-providers"]'),
  ).toExist();
  await expect(
    await wdio.$('[data-testid="settings-section-index"]'),
  ).toExist();
});
