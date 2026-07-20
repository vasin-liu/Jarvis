/**
 * Q&A journey with E2E fixture seeded at startup (JARVIS_E2E=1).
 */
import { askQuestion } from "../helpers.ts";

describe("Jarvis Q&A", () => {
  it("answers using the seeded knowledge base", async () => {
    await $('[data-testid="nav-chat"]').waitForDisplayed();
    const answer = await askQuestion("What is xyzzy-plugh?");
    const text = await answer.getText();
    expect(text).toContain("Mock");
  });

  it("renders a Chinese answer with a citation excerpt (regression: char-boundary panic)", async () => {
    // Long multibyte chunks once panicked when slicing excerpts at a byte
    // boundary, aborting the IPC future so the UI hung on "思考中...".
    const answer = await askQuestion("Jarvis 是什么？它支持哪些功能？");
    const text = await answer.getText();
    expect(text.length).toBeGreaterThan(0);
    const excerpts = await $$('[data-testid="citation-excerpt"]');
    expect(excerpts.length).toBeGreaterThan(0);
  });

  it("shows library stats after seeding", async () => {
    await $('[data-testid="nav-library"]').click();
    const stats = await $('[data-testid="library-stats"]');
    await expect(stats).toBeDisplayed();
    await expect(stats).toHaveText(expect.stringContaining("已索引"));
  });
});
