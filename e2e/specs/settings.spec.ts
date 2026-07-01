import { clickViaDom, expandSettingsSection, openNav } from "../helpers.ts";

/**
 * Settings panel: index status, API key keychain UX, accordion layout, and rebuild controls.
 */
describe("Jarvis settings", () => {
  beforeEach(async () => {
    await openNav("settings", '[data-testid="settings-panel"]');
  });

  it("shows index health with seeded fixture", async () => {
    const status = await $('[data-testid="index-status"]');
    await expect(status).toBeDisplayed();
    const text = await status.getText();
    expect(text).toMatch(/来源|分块|Embedder/);
  });

  it("exposes rebuild index control", async () => {
    const rebuild = await $('[data-testid="rebuild-index"]');
    await expect(rebuild).toBeDisplayed();
    await expect(rebuild).toBeEnabled();
  });

  it("renders cloud API key section with clear button under E2E", async () => {
    const embedderSelect = (await $$("select.field"))[0];
    await embedderSelect.selectByAttribute("value", "cloud");

    const input = await $('[data-testid="cloud-api-key-input"]');
    await expect(input).toBeDisplayed();
    await expect(input).toHaveAttribute("type", "password");

    const clearBtn = await $('[data-testid="clear-api-key"]');
    await expect(clearBtn).toBeDisplayed();
    await clickViaDom('[data-testid="clear-api-key"]');
  });

  it("renders accordion section containers", async () => {
    await expect($('[data-testid="settings-section-providers"]')).toExist();
    await expect($('[data-testid="settings-section-index"]')).toExist();
    await expect($('[data-testid="settings-section-sync"]')).toExist();
    await expect($('[data-testid="settings-section-lark"]')).toExist();
    await expect($('[data-testid="settings-section-agent"]')).toExist();
    await expect($('[data-testid="settings-section-advanced"]')).toExist();
  });

  it("expands Providers and Index sections by default", async () => {
    const providers = await $('[data-testid="settings-section-providers"]');
    const index = await $('[data-testid="settings-section-index"]');
    await expect(await providers.$("button")).toHaveAttribute(
      "aria-expanded",
      "true",
    );
    await expect(await index.$("button")).toHaveAttribute(
      "aria-expanded",
      "true",
    );
    await expect($('[data-testid="rebuild-index"]')).toBeDisplayed();
    await expect((await $$("select.field"))[0]).toBeDisplayed();
  });

  it("keeps default-expanded controls accessible", async () => {
    await expect($('[data-testid="rebuild-index"]')).toBeDisplayed();
    const embedderSelect = (await $$("select.field"))[0];
    await embedderSelect.selectByAttribute("value", "cloud");
    await expect($('[data-testid="cloud-api-key-input"]')).toBeDisplayed();
  });

  it("expands sync section and shows run-scheduled-sync", async () => {
    await expandSettingsSection("settings-section-sync");
    const runSync = await $('[data-testid="run-scheduled-sync"]');
    await expect(runSync).toBeDisplayed();
    await expect(runSync).toBeEnabled();
  });
});
