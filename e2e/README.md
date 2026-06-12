# Jarvis E2E Tests

End-to-end tests use [WebdriverIO](https://webdriver.io/) with [tauri-driver](https://github.com/tauri-apps/tauri-driver).

## Prerequisites (Windows)

1. Build the app: `npm run tauri build`
2. Install [Microsoft Edge WebDriver](https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/) matching your Edge version (or use `msedgedriver-tool`)
3. Install tauri-driver: `cargo install tauri-driver --locked`

## Run locally

```bash
npm run test:e2e:local
```

This sets `JARVIS_E2E=1` and seeds `e2e/fixtures/sample.md` with Mock providers before tests run.

## Selectors

UI elements expose `data-testid` attributes for stable automation (see `src/App.tsx`).

## CI

The `e2e` job on Windows installs `tauri-driver` and `msedgedriver-tool`, builds the release app, and runs the full WebDriver suite on every push to `dev` / `master`.
