import fs from "node:fs";
import os from "node:os";
import path from "node:path";

export async function openNav(id: string, readySelector?: string) {
  const nav = await $(`[data-testid="nav-${id}"]`);
  await nav.waitForDisplayed({ timeout: 15_000 });
  await nav.click();
  if (readySelector) {
    await $(readySelector).waitForDisplayed({ timeout: 15_000 });
  }
}

export async function expandSettingsSection(testId: string) {
  const section = await $(`[data-testid="${testId}"]`);
  await section.waitForExist({ timeout: 15_000 });
  const header = await section.$("button");
  if ((await header.getAttribute("aria-expanded")) !== "true") {
    await clickViaDom(`[data-testid="${testId}"] button`);
    await browser.waitUntil(
      async () => (await header.getAttribute("aria-expanded")) === "true",
      {
        timeout: 10_000,
        timeoutMsg: `${testId} did not expand`,
      },
    );
  }
}

export async function waitForDomText(
  selector: string,
  substring: string,
  timeoutMs = 15_000,
) {
  await browser.waitUntil(
    async () => {
      const text = await browser.execute((sel) => {
        const el = document.querySelector(sel);
        return el?.textContent ?? "";
      }, selector);
      return text.includes(substring);
    },
    {
      timeout: timeoutMs,
      timeoutMsg: `"${substring}" not found in ${selector}`,
    },
  );
}

/** Set value on a React-controlled textarea/input (plain setValue does not update state). */
export async function setReactCheckbox(
  element: WebdriverIO.Element,
  checked: boolean,
) {
  await element.waitForDisplayed({ timeout: 15_000 });
  await browser.execute(
    (elem, val) => {
      const setter = Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        "checked",
      )?.set;
      setter?.call(elem, val);
      elem.dispatchEvent(
        new InputEvent("input", {
          bubbles: true,
          inputType: "insertText",
          data: val,
        }),
      );
      elem.dispatchEvent(new Event("change", { bubbles: true }));
    },
    element,
    checked,
  );
}

export async function setReactInputValue(
  element: WebdriverIO.Element,
  value: string,
) {
  await element.waitForDisplayed({ timeout: 15_000 });
  await browser.execute(
    (elem, val) => {
      const proto =
        elem instanceof HTMLTextAreaElement
          ? HTMLTextAreaElement.prototype
          : HTMLInputElement.prototype;
      const setter = Object.getOwnPropertyDescriptor(proto, "value")?.set;
      setter?.call(elem, val);
      elem.dispatchEvent(
        new InputEvent("input", {
          bubbles: true,
          inputType: "insertText",
          data: val,
        }),
      );
      elem.dispatchEvent(new Event("change", { bubbles: true }));
    },
    element,
    value,
  );
}

export async function setOrchestrationMode(
  mode: "pipeline" | "router" | "single",
) {
  await expandSettingsSection("settings-section-agent");
  await browser.execute((m) => {
    (
      window as Window & {
        __JARVIS_E2E_SET_ORCHESTRATION__?: (
          mode: "pipeline" | "router" | "single",
        ) => void;
      }
    ).__JARVIS_E2E_SET_ORCHESTRATION__?.(m);
  }, mode);
  await browser.waitUntil(
    async () =>
      (await $(`[data-testid="${mode}-mode-toggle"]`)).isSelected(),
    {
      timeout: 15_000,
      timeoutMsg: `orchestration mode not set to ${mode}`,
    },
  );
  await browser.execute(async () => {
    const w = window as Window & {
      __JARVIS_E2E_REFRESH_CONFIG__?: () => Promise<void>;
    };
    await w.__JARVIS_E2E_REFRESH_CONFIG__?.();
  });
}

export async function setWikiExportPath(destPath: string) {
  await browser.execute((p: string) => {
    (
      window as Window & { __JARVIS_E2E_WIKI_EXPORT_PATH__?: string }
    ).__JARVIS_E2E_WIKI_EXPORT_PATH__ = p;
  }, destPath);
}

export function wikiE2eZipPath() {
  return path.join(os.tmpdir(), "jarvis-e2e-wiki.zip");
}

export function assertWikiZipNonEmpty(zipPath: string) {
  const st = fs.statSync(zipPath);
  if (st.size <= 0) {
    throw new Error(`wiki zip empty or missing: ${zipPath}`);
  }
}

export async function setReactRadio(selector: string) {
  await browser.execute((sel) => {
    const el = document.querySelector(sel) as HTMLInputElement | null;
    if (!el) return;
    const setter = Object.getOwnPropertyDescriptor(
      HTMLInputElement.prototype,
      "checked",
    )?.set;
    setter?.call(el, true);
    el.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    el.dispatchEvent(
      new InputEvent("input", { bubbles: true, inputType: "insertReplacementText" }),
    );
    el.dispatchEvent(new Event("change", { bubbles: true }));
  }, selector);
}

/** Click via DOM — WebDriver elementClick often misses React onClick handlers. */
export async function clickViaDom(selector: string) {
  await browser.execute((sel) => {
    (document.querySelector(sel) as HTMLElement | null)?.click();
  }, selector);
}

export async function waitForAssistantMessage(timeoutMs = 90_000) {
  await browser.waitUntil(
    async () => {
      if ((await $$('[data-testid="chat-message-assistant"]')).length > 0) {
        return true;
      }
      const errors = await $$('[data-testid="chat-error"]');
      return errors.length > 0;
    },
    {
      timeout: timeoutMs,
      timeoutMsg: "assistant message not rendered",
    },
  );
  const errors = await $$('[data-testid="chat-error"]');
  if (errors.length > 0) {
    throw new Error(`chat error: ${await errors[0].getText()}`);
  }
  const answers = await $$('[data-testid="chat-message-assistant"]');
  return answers[answers.length - 1];
}

export async function submitChatForm() {
  await browser.waitUntil(
    async () => (await $('[data-testid="ask-submit"]')).isEnabled(),
    {
      timeout: 15_000,
      timeoutMsg: "ask-submit stayed disabled (no active session?)",
    },
  );
  const before = await browser.execute(() =>
    Number(
      document
        .querySelector('[data-testid="ask-handle-count"]')
        ?.getAttribute("data-count") ?? "0",
    ),
  );
  await clickViaDom('[data-testid="ask-submit"]');
  await browser.waitUntil(
    async () => {
      const now = await browser.execute(() =>
        Number(
          document
            .querySelector('[data-testid="ask-handle-count"]')
            ?.getAttribute("data-count") ?? "0",
        ),
      );
      if (now > before) return true;
      try {
        return (await $$('[data-testid="ask-busy"]')).length > 0;
      } catch {
        return false;
      }
    },
    {
      timeout: 5_000,
      timeoutMsg: "handleAsk not invoked after submit",
    },
  );
}

export async function submitAgentForm() {
  await browser.waitUntil(
    async () => (await $('[data-testid="agent-submit"]')).isEnabled(),
    {
      timeout: 15_000,
      timeoutMsg: "agent-submit stayed disabled (no active session?)",
    },
  );
  const before = await browser.execute(() =>
    Number(
      document
        .querySelector('[data-testid="ask-handle-count"]')
        ?.getAttribute("data-count") ?? "0",
    ),
  );
  await clickViaDom('[data-testid="agent-submit"]');
  await browser.waitUntil(
    async () => {
      const now = await browser.execute(() =>
        Number(
          document
            .querySelector('[data-testid="ask-handle-count"]')
            ?.getAttribute("data-count") ?? "0",
        ),
      );
      if (now > before) return true;
      try {
        return (await $$('[data-testid="ask-busy"]')).length > 0;
      } catch {
        return false;
      }
    },
    {
      timeout: 5_000,
      timeoutMsg: "agent ask not invoked after submit",
    },
  );
}

export async function askAgentQuestion(text: string) {
  await openNav("chat", '[data-testid="chat-input"]');
  await browser.waitUntil(
    async () => (await $$('[data-testid="e2e-active"]')).length > 0,
    {
      timeout: 30_000,
      timeoutMsg: "e2e mode not active in app",
    },
  );
  await $('[data-testid="chat-session-ready"]').waitForExist({
    timeout: 30_000,
  });
  const textarea = await $('[data-testid="chat-input"]');
  await setReactInputValue(textarea, text);
  await submitAgentForm();
  return waitForAssistantMessage();
}

export async function askQuestion(text: string) {
  await openNav("chat", '[data-testid="chat-input"]');
  await browser.waitUntil(
    async () => (await $$('[data-testid="e2e-active"]')).length > 0,
    {
      timeout: 30_000,
      timeoutMsg: "e2e mode not active in app",
    },
  );
  await $('[data-testid="chat-session-ready"]').waitForExist({
    timeout: 30_000,
  });
  const textarea = await $('[data-testid="chat-input"]');
  await setReactInputValue(textarea, text);
  await submitChatForm();
  return waitForAssistantMessage();
}
