/**
 * Q&A journey with E2E fixture seeded at startup (JARVIS_E2E=1).
 */
describe("Jarvis Q&A", () => {
  it("answers using the seeded knowledge base", async () => {
    await $('[data-testid="nav-chat"]').waitForDisplayed();
    const textarea = await $("textarea");
    await textarea.setValue("What is xyzzy-plugh?");
    await $('[data-testid="ask-submit"]').click();
    await browser.waitUntil(
      async () => (await $$('[data-testid="chat-message-assistant"]')).length > 0,
      { timeout: 60000, timeoutMsg: "assistant message not rendered" },
    );
    const answers = await $$('[data-testid="chat-message-assistant"]');
    const answer = await answers[answers.length - 1].getText();
    expect(answer).toContain("Mock");
  });

  it("shows library stats after seeding", async () => {
    await $('[data-testid="nav-library"]').click();
    const stats = await $('[data-testid="library-stats"]');
    await expect(stats).toBeDisplayed();
    await expect(stats).toHaveText(expect.stringContaining("已索引"));
  });
});
