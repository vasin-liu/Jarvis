# Jarvis E2E Tests

End-to-end tests use [WebdriverIO](https://webdriver.io/) with [tauri-driver](https://github.com/tauri-apps/tauri-driver).

## Prerequisites (Windows)

1. Build the app: `npm run tauri build`
2. Install [Microsoft Edge WebDriver](https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/) matching your Edge version
3. Install tauri-driver: `cargo install tauri-driver --locked`

## Run

```bash
npm run test:e2e
```

## Selectors

UI elements expose `data-testid` attributes for stable automation (see `src/App.tsx`).

## CI note

E2E is optional in local dev when WebDriver is not installed. Unit tests (`npm test`) cover parsing and Rust integration tests cover index rebuild.
