import { askAgentQuestion, openNav, setOrchestrationMode } from "../helpers.ts";

/**
 * Agent mode UI and orchestration settings.
 */
describe("Jarvis agent mode", () => {
  async function ensureAgentMode() {
    await openNav("chat");
    const toggle = await $('[data-testid="agent-mode-toggle"]');
    await toggle.waitForDisplayed();
    if (!(await toggle.isSelected())) {
      await toggle.click();
    }
    await expect(toggle).toBeSelected();
  }

  it("asks a question in agent mode", async () => {
    await ensureAgentMode();
    const answer = await askAgentQuestion("search knowledge about Jarvis");
    const text = await answer.getText();
    expect(text.length).toBeGreaterThan(0);
  });

  it("answers a Chinese question in agent mode (regression: char-boundary panic)", async () => {
    // The agent search_knowledge tool sliced citation excerpts at a byte
    // boundary; multibyte (Chinese) chunks panicked → UI stuck on "思考中...".
    await ensureAgentMode();
    const answer = await askAgentQuestion("Jarvis 支持哪些工具和功能？");
    const text = await answer.getText();
    expect(text.length).toBeGreaterThan(0);
  });

  it("reflects orchestration mode in chat UI", async () => {
    // Orchestration radios are React-controlled; WebDriver .click() does not
    // reliably fire onChange, so drive state via the deterministic e2e hook.
    // The chat view renders an orchestration-mode-{mode} marker reflecting state
    // (a CJK partial-text selector like *=流水线模式 is unreliable in WebdriverIO).
    await openNav("settings");
    await setOrchestrationMode("pipeline");
    await ensureAgentMode();
    await $('[data-testid="orchestration-mode-pipeline"]').waitForExist({
      timeout: 15_000,
    });

    await openNav("settings");
    await setOrchestrationMode("router");
    await ensureAgentMode();
    await $('[data-testid="orchestration-mode-router"]').waitForExist({
      timeout: 15_000,
    });
  });
});
