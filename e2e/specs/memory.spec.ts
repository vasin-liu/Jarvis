import { openNav } from "../helpers.ts";

/**
 * Memory CRUD journey (v9): add → edit → forget.
 */
describe("Jarvis memory", () => {
  it("adds, edits, and forgets a manual memory", async () => {
    const unique = `E2E-${Date.now()}`;
    const memoryText = `${unique}: user prefers dark mode`;

    await openNav("memory");

    const input = await $('[data-testid="new-memory-input"]');
    await input.setValue(memoryText);
    await $('[data-testid="add-memory-submit"]').click();

    const list = await $('[data-testid="memory-list"]');
    await browser.waitUntil(
      async () => (await list.getText()).includes(unique),
      { timeout: 30_000, timeoutMsg: "memory not listed after add" },
    );

    const editBtn = await $(`[data-testid^="memory-edit-"]`);
    await editBtn.click();

    await $('[data-testid="memory-edit-panel"]').waitForDisplayed();
    const content = await $('[data-testid="memory-edit-content"]');
    await content.setValue(`${unique}: user prefers light mode`);
    await $('[data-testid="memory-save-edit"]').click();

    await browser.waitUntil(
      async () => {
        const body = await browser.execute(() => document.body.textContent ?? "");
        return body.includes("light mode");
      },
      { timeout: 30_000, timeoutMsg: "memory not updated" },
    );

    const forgetBtn = await $(`[data-testid^="memory-forget-"]`);
    await forgetBtn.click();

    await browser.waitUntil(
      async () => (await $$(`[data-testid^="memory-forget-"]`)).length === 0,
      { timeout: 30_000, timeoutMsg: "memory still present after forget" },
    );
  });
});
