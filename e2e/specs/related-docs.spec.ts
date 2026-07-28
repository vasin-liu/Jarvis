import { openNav } from "../helpers.ts";

/**
 * Related-docs Library panel (Phase 16): select indexed source → panel → navigate neighbor.
 * Source ids are filesystem URIs (Windows `\\?\` paths) — avoid brittle CSS attribute
 * selectors; click / assert via DOM query + exact data-testid string match.
 */
async function clickSourceRowByIndex(index: number) {
  await browser.execute((i) => {
    const rows = document.querySelectorAll('[data-testid^="source-row-"]');
    const el = rows[i] as HTMLElement | undefined;
    if (!el) throw new Error(`source-row index ${i} missing (have ${rows.length})`);
    el.click();
  }, index);
}

async function clickRelatedRowByIndex(index: number): Promise<string> {
  return browser.execute((i) => {
    const rows = document.querySelectorAll('[data-testid^="related-docs-row-"]');
    const el = rows[i] as HTMLElement | undefined;
    if (!el) throw new Error(`related-docs-row index ${i} missing (have ${rows.length})`);
    const tid = el.getAttribute("data-testid") ?? "";
    const neighborId = tid.replace(/^related-docs-row-/, "");
    el.click();
    return neighborId;
  }, index);
}

async function clickSourceRowById(sourceId: string) {
  await browser.execute((id) => {
    const want = `source-row-${id}`;
    for (const el of document.querySelectorAll('[data-testid^="source-row-"]')) {
      if (el.getAttribute("data-testid") === want) {
        (el as HTMLElement).click();
        return;
      }
    }
    throw new Error(`source-row not found for id length=${id.length}`);
  }, sourceId);
}

async function isSourceRowSelected(sourceId: string): Promise<boolean> {
  return browser.execute((id) => {
    const want = `source-row-${id}`;
    for (const el of document.querySelectorAll('[data-testid^="source-row-"]')) {
      if (el.getAttribute("data-testid") === want) {
        return (
          el.getAttribute("aria-selected") === "true" ||
          el.getAttribute("data-selected") === "true"
        );
      }
    }
    return false;
  }, sourceId);
}

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

    await clickSourceRowByIndex(0);

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

    const neighborId = await clickRelatedRowByIndex(0);

    await browser.waitUntil(async () => isSourceRowSelected(neighborId), {
      timeout: 15_000,
      timeoutMsg: "neighbor source-row not selected after related click",
    });

    await expect($('[data-testid="related-docs-panel"]')).toBeDisplayed();

    await clickSourceRowById(neighborId);
    await browser.waitUntil(
      async () => (await $$('[data-testid="related-docs-panel"]')).length === 0,
      {
        timeout: 10_000,
        timeoutMsg: "related panel still visible after deselect (D-08)",
      },
    );
  });
});
