# Ctrl+Tab Switch Contacts Design

## Requirement
Add support for using `Ctrl+Tab` and `Ctrl+Shift+Tab` to switch between contacts (like switching browser tabs). Implement it carefully to avoid logic errors, and provide a toggle in the Settings page to enable/disable it.

## Design

### 1. Settings state management (`useSettings.ts`)
- Add a new ref: `const enableCtrlTabSwitch = ref(true)`.
- Persist it to `localStorage` (and sync via backend `save_settings` like other settings).
- Expose `enableCtrlTabSwitch` and a `saveCtrlTabSwitch` function to toggle it.

### 2. Settings UI (`SettingsView.vue`)
- Add a new toggle row in the "功能设置" (Features) or "快捷键" (Shortcuts) section for "启用 Ctrl+Tab 切换联系人".
- Bind it to `enableCtrlTabSwitch` with `@change="saveCtrlTabSwitch"`.

### 3. Contact switching logic (`Sidebar.vue`)
- Introduce a new method `switchPeerTab(direction: 1 | -1)` and expose it via `defineExpose`.
- Inside `switchPeerTab`:
  1. Flatten `peerGroups.value` to get an ordered, 1D array of visible `peers`.
  2. If the array is empty or has only 1 element, do nothing.
  3. Find the index of the currently active peer (`activePeerId.value`).
  4. If the active peer isn't in the list (e.g., search filter), start from index 0.
  5. Compute the next index using `(index + direction + length) % length`.
  6. Emit `'select-peer'` with the new peer's ID.

### 4. Global Keyboard Listener (`App.vue`)
- Reference the Sidebar component: `<Sidebar ref="sidebarRef" ... />`.
- In `handleGlobalKeydown`:
  - Check `e.ctrlKey && e.key.toLowerCase() === 'tab' && enableCtrlTabSwitch.value`.
  - Call `e.preventDefault()` to stop browser/system tab switching.
  - Call `sidebarRef.value.switchPeerTab(e.shiftKey ? -1 : 1)`.
