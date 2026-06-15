# Jarvis

Personal AI knowledge hub — Tauri 2 + Rust + React. Local-first RAG over documents and Feishu content.

**Version:** 1.3.0 (v4 — scheduled sync & learning memory; v3 insights; v2 Cursor; v1 M1–M10)

**Docs:** `docs/superpowers/specs/2026-06-12-personal-ai-knowledge-hub-design.md`  
**Agent guide:** `AGENTS.md`

### Reference assets (non-normative)

| Asset | Path |
| --- | --- |
| UI mockup | `docs/personal_agent_ui_mockup.html` |
| Architecture diagram | `docs/personal_ai_agent_architecture.svg` |

---

## Tauri + React + TypeScript

This project uses Tauri, React and TypeScript with Vite.

### Rust toolchain

- **Channel:** latest stable (see `rust-toolchain.toml`)
- **Setup:** [rustup](https://rustup.rs/) → `rustup update stable`
- **Verify:** `rustc --version` (should report stable)

## Tests

```bash
cargo test --workspace   # Rust unit + integration
npm test                 # Vitest (frontend helpers)
npm run test:e2e:local   # WebDriver (Windows + Edge WebDriver)
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
