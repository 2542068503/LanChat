# Open Links Externally Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Intercept clicks on links and open them in the system's default browser instead of the Tauri app webview.

**Architecture:** We will add a global `click` event listener in the application's entry file `src/main.ts`.

**Tech Stack:** Vue 3, Tauri API (`@tauri-apps/plugin-opener`).

## Global Constraints
- Only intercept `<a>` elements with an `href` that starts with `http` or `https`.
- Prevent the default action to stop the webview from navigating.

---

### Task 1: Add Global Click Interceptor

**Files:**
- Modify: `d:/LanChat/src/main.ts`

**Interfaces:**
- Modifies global document behavior.

- [ ] **Step 1: Import `openUrl`**

Add the import at the top of `src/main.ts`:
```typescript
import { openUrl } from '@tauri-apps/plugin-opener';
```

- [ ] **Step 2: Add the `click` event listener**

Add the following code before or after `createApp(App).mount("#app");`:
```typescript
document.addEventListener('click', (e) => {
  const target = e.target as HTMLElement;
  const a = target.closest('a');
  if (a && a.href && (a.href.startsWith('http://') || a.href.startsWith('https://'))) {
    e.preventDefault();
    openUrl(a.href).catch(console.error);
  }
});
```

- [ ] **Step 3: Commit**

```bash
git add src/main.ts
git commit -m "feat: open external links in system browser"
```
