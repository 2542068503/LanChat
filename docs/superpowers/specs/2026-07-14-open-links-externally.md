# Open Links in System Browser Design

## Requirement
Do not allow external links (e.g., from Markdown rendering) to open inside the Tauri app window.

## Design
- Add a global `click` event listener in `src/main.ts`.
- When a click occurs, check if `target.closest('a')` exists.
- If it is an `<a>` tag with an `href` attribute starting with `http` or `https`:
  - Call `e.preventDefault()`.
  - Import `openUrl` from `@tauri-apps/plugin-opener`.
  - Execute `openUrl(href)` to delegate opening the URL to the default system browser.
