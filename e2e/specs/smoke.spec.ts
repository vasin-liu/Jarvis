/**
 * Smoke journey: app loads and shows primary navigation.
 * Requires tauri-driver + built binary (see e2e/README.md).
 */
describe("Jarvis smoke", () => {
  it("shows chat navigation", async () => {
    const nav = await $('[data-testid="nav-chat"]');
    await expect(nav).toBeDisplayed();
  });

  it("can open settings view", async () => {
    await $('[data-testid="nav-settings"]').click();
    await expect($('[data-testid="settings-panel"]')).toBeDisplayed();
  });
});
