import { clickViaDom, openNav } from "../helpers.ts";

/**
 * Related-docs Library panel (Phase 16): select indexed source → panel → navigate neighbor.
 */
describe("Jarvis related docs", () => {
  it("selects a source, shows related neighbors, and navigates on click", async () => {
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

    const sourceRows = await $$('[data-testid^="source-row-"]');
    expect(sourceRows.length).toBeGreaterThanOrEqual(2);

    const firstRowId = await sourceRows[0].getAttribute("data-testid");
    await clickViaDom(`[data-testid="${firstRowId}"]`);

    const panel = await $('[data-testid="related-docs-panel"]');
    await panel.waitForDisplayed({ timeout: 15_000 });

    await browser.waitUntil(
      async () => (await $$('[data-testid="related-docs-loading"]')).length === 0,
      {
        timeout: 30_000,
        timeoutMsg: "related-docs loading did not clear",
      },
    );

    const list = await $('[data-testid="related-docs-list"]');
    await list.waitForDisplayed({
      timeout: 30_000,
    });

    const relatedRows = await $$('[data-testid^="related-docs-row-"]');
    expect(relatedRows.length).toBeGreaterThanOrEqual(1);

    const relatedTestId = await relatedRows[0].getAttribute("data-testid");
    const neighborId = relatedTestId.replace(/^related-docs-row-/, "");
    await clickViaDom(`[data-testid="${relatedTestId}"]`);

    await browser.waitUntil(
      async () => {
        const row = await $(`[data-testid="source-row-${neighborId}"]`);
        const aria = await row.getAttribute("aria-selected");
        const data = await row.getAttribute("data-selected");
        return aria === "true" || data === "true";
      },
      {
        timeout: 15_000,
        timeoutMsg: `neighbor source-row-${neighborId} not selected after related click`,
      },
    );

    await expect($('[data-testid="related-docs-panel"]')).toBeDisplayed();

    await clickViaDom(`[data-testid="source-row-${neighborId}"]`);
    await browser.waitUntil(
      async () => (await $$('[data-testid="related-docs-panel"]')).length === 0,
      {
        timeout: 10_000,
        timeoutMsg: "related panel still visible after deselect (D-08)",
      },
    );
  });
});
