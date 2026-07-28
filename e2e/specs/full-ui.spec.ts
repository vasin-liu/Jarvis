import {
  askQuestion,
  clickViaDom,
  expandSettingsSection,
  openNav,
  setReactCheckbox,
  setReactInputValue,
  setOrchestrationMode,
  waitForDomText,
} from "../helpers.ts";

/**
 * Comprehensive UI E2E journey — single session, ordered flow.
 * Requires tauri-driver + release binary (see e2e/README.md).
 */
describe("Jarvis full UI journey", () => {
  it("loads primary navigation and all main views", async () => {
    await expect($('[data-testid="nav-chat"]')).toBeDisplayed();

    await openNav("library");
    await expect($('[data-testid="library-stats"]')).toBeDisplayed();

    await openNav("tasks");
    await expect($('[data-testid="task-list"]')).toBeDisplayed();

    await openNav("memory");
    await expect($('[data-testid="new-memory-input"]')).toBeDisplayed();

    await openNav("settings");
    await expect($('[data-testid="settings-panel"]')).toBeDisplayed();
    await expect($('[data-testid="index-status"]')).toBeDisplayed();
  });

  it("navigates Settings accordion and returns to chat", async () => {
    await openNav("settings", '[data-testid="settings-panel"]');
    await expect($('[data-testid="settings-section-providers"]')).toExist();
    await expect($('[data-testid="settings-section-index"]')).toExist();
    await openNav("chat");
    await expect($('[data-testid="nav-chat"]')).toBeDisplayed();
    await expect($('[data-testid="chat-input"]')).toBeDisplayed();
  });

  it("answers using the seeded knowledge base", async () => {
    const answer = await askQuestion("What is xyzzy-plugh?");
    const text = await answer.getText();
    expect(text).toContain("Mock");
  });

  it("shows library stats after seeding", async () => {
    await openNav("library");
    const stats = await $('[data-testid="library-stats"]');
    await expect(stats).toHaveText(expect.stringContaining("已索引"));
    // Wiki default-off (D-12): no compile/export until Settings enables wiki
    expect(await $$('[data-testid="wiki-export"]')).toHaveLength(0);
    expect(await $$('[data-testid^="wiki-compile-"]')).toHaveLength(0);
  });

  it("shows related-docs panel after selecting a library source", async () => {
    await openNav("library", '[data-testid="library-stats"]');

    await browser.waitUntil(
      async () => {
        const text = await $('[data-testid="library-stats"]').getText();
        const m = text.match(/已索引 (\d+)/);
        return m !== null && Number(m[1]) >= 2;
      },
      {
        timeout: 60_000,
        timeoutMsg: "dual E2E fixtures not indexed yet (need ≥2 indexed)",
      },
    );

    // Source ids are Windows path URIs — click by index via DOM, not CSS attribute selectors.
    await browser.execute(() => {
      const rows = document.querySelectorAll('[data-testid^="source-row-"]');
      const el = rows[0] as HTMLElement | undefined;
      if (!el) throw new Error(`source-row index 0 missing (have ${rows.length})`);
      el.click();
    });

    await expect($('[data-testid="related-docs-panel"]')).toBeDisplayed();
  });

  it("manages memories: add, edit, forget", async () => {
    const unique = `E2E-${Date.now()}`;
    const memoryText = `${unique}: user prefers dark mode`;

    await openNav("memory", '[data-testid="new-memory-input"]');
    const input = await $('[data-testid="new-memory-input"]');
    await setReactInputValue(input, memoryText);
    await clickViaDom('[data-testid="add-memory-submit"]');

    const list = await $('[data-testid="memory-list"]');
    await browser.waitUntil(
      async () => (await list.getText()).includes(unique),
      { timeout: 30_000, timeoutMsg: "memory not listed after add" },
    );

    await browser.waitUntil(
      async () => (await $$('[data-testid^="memory-edit-"]')).length > 0,
      { timeout: 30_000, timeoutMsg: "memory edit button missing" },
    );
    await clickViaDom('[data-testid^="memory-edit-"]');
    await $('[data-testid="memory-edit-panel"]').waitForDisplayed();
    const content = await $('[data-testid="memory-edit-content"]');
    const updated = `${unique}: user prefers light mode`;
    await setReactInputValue(content, updated);
    await browser.waitUntil(
      async () => (await content.getValue()).includes("light mode"),
      { timeout: 10_000, timeoutMsg: "memory edit field not updated" },
    );
    await clickViaDom('[data-testid="memory-save-edit"]');
    await browser.waitUntil(
      async () => {
        try {
          return !(await $('[data-testid="memory-edit-panel"]').isDisplayed());
        } catch {
          return true;
        }
      },
      { timeout: 15_000, timeoutMsg: "memory edit panel stayed open" },
    );

    await browser.waitUntil(
      async () => (await list.getText()).includes("light"),
      { timeout: 30_000, timeoutMsg: "memory not updated" },
    );

    await clickViaDom('[data-testid^="memory-forget-"]');
    await browser.waitUntil(
      async () => (await $$(`[data-testid^="memory-forget-"]`)).length === 0,
      { timeout: 30_000, timeoutMsg: "memory still present after forget" },
    );
  });

  it("runs agent mode and switches orchestration UI", async () => {
    await openNav("chat");
    const toggle = await $('[data-testid="agent-mode-toggle"]');
    if (!(await toggle.isSelected())) {
      await setReactCheckbox(toggle, true);
    }
    await expect(toggle).toBeSelected();

    const answer = await askQuestion("search knowledge about Jarvis");
    expect((await answer.getText()).length).toBeGreaterThan(0);

    await openNav("settings");
    await setOrchestrationMode("pipeline");
    await openNav("chat");
    const agentToggle = await $('[data-testid="agent-mode-toggle"]');
    if (!(await agentToggle.isSelected())) {
      await setReactCheckbox(agentToggle, true);
    }
    await $('[data-testid="orchestration-mode-pipeline"]').waitForExist({
      timeout: 15_000,
    });

    await openNav("settings");
    await setOrchestrationMode("router");
    await openNav("chat");
    if (!(await agentToggle.isSelected())) {
      await setReactCheckbox(agentToggle, true);
    }
    await $('[data-testid="orchestration-mode-router"]').waitForExist({
      timeout: 15_000,
    });
  });

  it("shows index health and rebuild control", async () => {
    await openNav("settings");
    const status = await $('[data-testid="index-status"]');
    const text = await status.getText();
    expect(text).toMatch(/来源|分块|Embedder/);

    const rebuild = await $('[data-testid="rebuild-index"]');
    await expect(rebuild).toBeDisplayed();
    await expect(rebuild).toBeEnabled();
  });

  it("checks lark-cli mock connection and syncs e2e feishu file", async () => {
    await openNav("settings", '[data-testid="settings-panel"]');
    await expandSettingsSection("settings-section-lark");
    await clickViaDom('[data-testid="lark-check-connection"]');
    await waitForDomText('[data-testid="lark-status-panel"]', "E2E User");
    const larkText = await browser.execute(() => {
      const el = document.querySelector('[data-testid="lark-status-panel"]');
      return el?.textContent ?? "";
    });
    expect(larkText).toContain("E2E User");

    await expandSettingsSection("settings-section-advanced");
    const input = await $('[data-testid="lark-sync-url-input"]');
    await setReactInputValue(
      input,
      "https://e2e.feishu.cn/file/e2e-mock-token",
    );
    await clickViaDom('[data-testid="lark-sync-submit"]');
    await openNav("library", '[data-testid="library-stats"]');
    const list = await $('[data-testid="source-list"]');
    await browser.waitUntil(
      async () => (await list.getText()).includes("E2E Lark Cloud File"),
      { timeout: 30_000, timeoutMsg: "lark source missing from library" },
    );
  });
});
