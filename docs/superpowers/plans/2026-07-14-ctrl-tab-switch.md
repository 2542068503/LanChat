# Ctrl+Tab Switch Contacts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Ctrl+Tab / Ctrl+Shift+Tab to switch contacts with a toggle in settings.

**Architecture:**
- State: `useSettings.ts` (export `enableCtrlTabSwitch`)
- UI Toggle: `SettingsView.vue`
- Logic: `Sidebar.vue` (export `switchPeerTab`)
- Listener: `App.vue` (handle `Ctrl+Tab` in `handleGlobalKeydown`)

---

### Task 1: Update Settings State
**Files:**
- Modify: `d:/LanChat/src/composables/useSettings.ts`

- [ ] Add `const enableCtrlTabSwitch = ref(true);`
- [ ] Load from local storage in `initSettings`:
  ```typescript
  const savedCtrlTab = localStorage.getItem("enableCtrlTabSwitch");
  if (savedCtrlTab) {
    enableCtrlTabSwitch.value = savedCtrlTab === "true";
  }
  ```
- [ ] Add it to `syncSettings` backend call.
- [ ] Add `saveCtrlTabSwitch` function:
  ```typescript
  const saveCtrlTabSwitch = () => {
    localStorage.setItem("enableCtrlTabSwitch", enableCtrlTabSwitch.value.toString());
    syncSettings();
  };
  ```
- [ ] Return `enableCtrlTabSwitch` and `saveCtrlTabSwitch` in the composable.

### Task 2: Update Settings UI
**Files:**
- Modify: `d:/LanChat/src/components/SettingsView.vue`

- [ ] Import `enableCtrlTabSwitch` and `saveCtrlTabSwitch` from `useSettings`.
- [ ] Add a UI toggle row in the settings template (under `enableCtrlWClose`):
  ```html
  <div class="setting-item">
    <div class="setting-info">
      <h4>启用 Ctrl+Tab 快捷键切换联系人</h4>
      <p>使用 Ctrl+Tab 和 Ctrl+Shift+Tab 在好友列表间快速切换</p>
    </div>
    <div class="setting-action">
      <label class="switch">
        <input type="checkbox" v-model="enableCtrlTabSwitch" @change="saveCtrlTabSwitch">
        <span class="slider round"></span>
      </label>
    </div>
  </div>
  ```

### Task 3: Implement Switch Logic in Sidebar
**Files:**
- Modify: `d:/LanChat/src/components/Sidebar.vue`

- [ ] Expose `switchPeerTab` method:
  ```typescript
  defineExpose({
    switchPeerTab(direction: 1 | -1) {
      // Flatten all peers from visible groups
      const visiblePeers = [];
      for (const group of peerGroups.value) {
        if (group.peers.length > 0) {
          visiblePeers.push(...group.peers);
        }
      }
      
      if (visiblePeers.length <= 1) return;

      const currentIndex = visiblePeers.findIndex(p => p.id === activePeerId.value);
      let nextIndex = 0;
      
      if (currentIndex !== -1) {
        nextIndex = (currentIndex + direction + visiblePeers.length) % visiblePeers.length;
      }
      
      emit('select-peer', visiblePeers[nextIndex].id);
    }
  });
  ```

### Task 4: Global Event Listener
**Files:**
- Modify: `d:/LanChat/src/App.vue`

- [ ] Import `enableCtrlTabSwitch` from `useSettings`.
- [ ] Add a template ref for `<Sidebar>`: `const sidebarRef = ref<any>(null);`
- [ ] Update the `<Sidebar>` tag to include `ref="sidebarRef"`.
- [ ] Update `handleGlobalKeydown`:
  ```typescript
  if (e.ctrlKey && e.key.toLowerCase() === 'tab' && enableCtrlTabSwitch.value) {
    e.preventDefault();
    if (sidebarRef.value) {
      sidebarRef.value.switchPeerTab(e.shiftKey ? -1 : 1);
    }
  }
  ```

### Task 5: Verification & Commit
- [ ] Test the toggle in settings.
- [ ] Test Ctrl+Tab going forward and Ctrl+Shift+Tab going backward.
- [ ] Test edge cases (no active peer, 1 peer).
- [ ] Commit changes.
