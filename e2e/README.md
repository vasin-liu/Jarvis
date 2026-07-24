# Jarvis E2E Tests

End-to-end tests use [WebdriverIO](https://webdriver.io/) with [tauri-driver](https://github.com/tauri-apps/tauri-driver).

## Policy

**Every user-facing feature must have E2E coverage** before merge.

- Spec file under `e2e/specs/`
- `data-testid` on interactive UI
- Deterministic mocks in `JARVIS_E2E=1` mode (`src-tauri/src/e2e.rs`)
- CI `e2e` job green on Windows

See `.cursor/rules/e2e-required.mdc` and `AGENTS.md`.

## Prerequisites (Windows)

1. Build the app: `npm run tauri build`
2. Install [Microsoft Edge WebDriver](https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/) matching your Edge version (or use `msedgedriver-tool`)
3. Install tauri-driver: `cargo install tauri-driver --locked`

## Run locally

```bash
# Ensure msedgedriver is on PATH (repo root after msedgedriver-tool, or system install),
# then build a production binary AND run the full UI suite in one step:
npm run test:e2e:local
```

`test:e2e:local` first runs `npm run build:e2e` (a **production** `tauri build --no-bundle`
that embeds `dist` — never a dev-mode binary, see Troubleshooting), then sets `JARVIS_E2E=1`,
seeds `e2e/fixtures/sample.md`, and runs **all** `e2e/specs/**/*.spec.ts` with Mock providers.

To rebuild the binary alone: `npm run build:e2e`. To re-run specs against an already-built
binary (skip the build): `npm run test:e2e`.

For a single spec:

```bash
cross-env JARVIS_E2E=1 JARVIS_E2E_FIXTURE=./e2e/fixtures/sample.md npx wdio run e2e/wdio.conf.ts --spec e2e/specs/lark.spec.ts
```

## Specs

| File | Coverage |
|------|----------|
| `full-ui.spec.ts` | Full journey: nav, Q&A, memory, agent, settings, **Lark** |
| `lark.spec.ts` | Feishu connection check + URL sync (E2E mock, no live lark-cli) |
| `smoke.spec.ts` | App load, settings |
| `navigation.spec.ts` | All main nav views |
| `qa.spec.ts` | Seeded fixture Q&A + library stats |
| `memory.spec.ts` | Add / edit / forget memory (v9) |
| `agent.spec.ts` | Agent mode + orchestration UI |
| `settings.spec.ts` | Index status + rebuild control + accordion layout |
| `wiki.spec.ts` | Wiki enable→compile→笔记页→export + citation URI trust; default-off Library controls |

## Supplemental Playwright smoke (not CI)

Local-only nav smoke using `@playwright/test` with the same `tauri-driver` + release binary as WDIO:

```bash
npm run build:e2e
npm run e2e:playwright:local
```

Spec: `e2e/playwright/nav-smoke.spec.ts`. **CI merge gate remains WebdriverIO** (`npm run test:e2e`).

## Adding a feature (checklist)

```
[ ] data-testid on new UI controls
[ ] E2E mock in src-tauri/src/e2e.rs (if external API)
[ ] e2e/specs/<area>.spec.ts test(s)
[ ] full-ui.spec.ts updated if primary journey
[ ] npm run test:e2e:local passes
```

## Troubleshooting

**Blank / white screen, or specs fail with `app-root missing`:** the binary at
`target/release/tauri-app.exe` was built in **dev mode** (e.g. via `cargo build`),
so the WebView loads `devUrl` (`http://localhost:1420`) and shows
*"localhost refused to connect"* (`chrome-error://chromewebdata/`). Rebuild a
**production** binary that embeds `dist` with `npm run tauri build` (prod mode
uses `frontendDist`, not `devUrl`). Verify the WebView URL is `http://tauri.localhost/`.

## Selectors

UI elements expose `data-testid` attributes for stable automation (see `src/App.tsx`).

## E2E mocks

| Service | Mock location |
|---------|----------------|
| LLM / embedder | `apply_e2e_config` |
| Feishu | `e2e::lark_auth_status`, `e2e::lark_fixture_from_input` |
| Fixture file | `JARVIS_E2E_FIXTURE` → `e2e/fixtures/sample.md` |

E2E Lark URL for tests: `https://e2e.feishu.cn/file/e2e-mock-token` (marker: `xyzzy-lark-e2e`).

## CI

The `e2e` job on Windows installs `tauri-driver` and `msedgedriver-tool`, builds the release app, and runs the full WebDriver suite on every push to `dev` / `master`.
