import {
  clickViaDom,
  expandSettingsSection,
  openNav,
  setReactInputValue,
  waitForDomText,
} from "../helpers.ts";

const E2E_LARK_URL = "https://e2e.feishu.cn/file/e2e-mock-token";
const E2E_LARK_MARKER = "xyzzy-lark-e2e";

describe("Lark / Feishu integration (E2E mock)", () => {
  it("shows mocked lark-cli connection status in settings", async () => {
    await openNav("settings", '[data-testid="settings-panel"]');
    await expandSettingsSection("settings-section-lark");
    await clickViaDom('[data-testid="lark-check-connection"]');
    await waitForDomText(
      '[data-testid="lark-status-panel"]',
      "lark-cli 可用",
    );
    const text = await browser.execute(() => {
      const el = document.querySelector('[data-testid="lark-status-panel"]');
      return el?.textContent ?? "";
    });
    expect(text).toContain("lark-cli 可用");
    expect(text).toContain("E2E User");
  });

  it("syncs a Feishu URL into the library without live lark-cli", async () => {
    await openNav("settings", '[data-testid="settings-panel"]');
    await expandSettingsSection("settings-section-advanced");
    const input = await $('[data-testid="lark-sync-url-input"]');
    await setReactInputValue(input, E2E_LARK_URL);
    await clickViaDom('[data-testid="lark-sync-submit"]');

    await openNav("library", '[data-testid="library-stats"]');
    const list = await $('[data-testid="source-list"]');
    await browser.waitUntil(
      async () => (await list.getText()).includes("E2E Lark Cloud File"),
      {
        timeout: 30_000,
        timeoutMsg: "lark mock source not indexed in library",
      },
    );
    expect(await list.getText()).toContain("E2E Lark Cloud File");
  });
});
