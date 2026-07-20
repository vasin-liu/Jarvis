import { openNav } from "../helpers.ts";

/**
 * Primary navigation — every main view loads without error.
 */
describe("Jarvis navigation", () => {
  it("opens all main views", async () => {
    await openNav("chat");
    await expect($('[data-testid="ask-submit"]')).toBeDisplayed();

    await openNav("library");
    await expect($('[data-testid="library-stats"]')).toBeDisplayed();

    await openNav("tasks");
    await expect($('[data-testid="task-list"]')).toBeDisplayed();

    await openNav("memory");
    await expect($('[data-testid="new-memory-input"]')).toBeDisplayed();
    await expect($('[data-testid="memory-list"]')).toBeDisplayed();

    await openNav("settings");
    await expect($('[data-testid="settings-panel"]')).toBeDisplayed();
    await expect($('[data-testid="index-status"]')).toBeDisplayed();
  });
});
