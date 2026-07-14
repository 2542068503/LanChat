# Chat Input Limits Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add rate limiting, an input box row limit, and a long-text-to-file conversion prompt to the chat input area.

**Architecture:** We will modify the Vue component `ChatArea.vue` to track the last send time, watch the input for line length to enforce the row limit, and intercept the `sendMsg` function to show a confirmation dialog for long texts.

**Tech Stack:** Vue 3 Composition API, Tauri API.

## Global Constraints
- Rate limit interval: 0.5s.
- Row limit: 50 lines.
- Long text limit: > 2000 characters.

---

### Task 1: Add Rate Limiting

**Files:**
- Modify: `d:/LanChat/src/components/ChatArea.vue`

**Interfaces:**
- Modifies internal state `lastSendTime` and `sendMsg` behavior.

- [ ] **Step 1: Add `lastSendTime` ref**

Inside the `<script setup>` block, add:
```typescript
const lastSendTime = ref(0);
```

- [ ] **Step 2: Update `sendMsg` to check rate limit**

Modify the beginning of `sendMsg` function:
```typescript
function sendMsg() {
  const content = messageInput.value.trim();
  if (!content) return;

  const now = Date.now();
  if (now - lastSendTime.value < 500) {
    showToast("发送太频繁，请稍后再试", "error");
    return;
  }
  lastSendTime.value = now;
  // ... rest of sendMsg
```

- [ ] **Step 3: Commit**

```bash
git add src/components/ChatArea.vue
git commit -m "feat: add rate limiting to chat input"
```

---

### Task 2: Add Input Box Row Limit

**Files:**
- Modify: `d:/LanChat/src/components/ChatArea.vue`

**Interfaces:**
- Watches `messageInput`.

- [ ] **Step 1: Add `watch` on `messageInput`**

Inside the `<script setup>` block, add a watch to enforce 50 lines limit:
```typescript
watch(messageInput, (newVal) => {
  const lines = newVal.split('\n');
  if (lines.length > 50) {
    messageInput.value = lines.slice(0, 50).join('\n');
    showToast("输入不能超过 50 行", "error");
  }
});
```

- [ ] **Step 2: Remove the old > 50 lines block from `sendMsg`**

Remove these lines from `sendMsg`:
```typescript
  const linesCount = content.split('\n').length;
  if (linesCount > 50) {
    showToast("发送的消息不能超过 50 行", "error");
    return;
  }
```

- [ ] **Step 3: Commit**

```bash
git add src/components/ChatArea.vue
git commit -m "feat: enforce 50-line limit in chat input box"
```

---

### Task 3: Long Text to File Conversion

**Files:**
- Modify: `d:/LanChat/src/components/ChatArea.vue`

**Interfaces:**
- Modifies `sendMsg` behavior for `content.length > 2000`.

- [ ] **Step 1: Update `sendMsg` to prompt on long text and convert to file**

Replace the old `content.length > 2000` block:
```typescript
  if (content.length > 2000) {
    showToast("发送的文本不能超过 2000 个字符", "error");
    return;
  }
```

With the new confirmation and conversion logic:
```typescript
  if (content.length > 2000) {
    if (window.confirm("文本过长(超过2000字符)，是否将其转换为文件发送？")) {
      const isLatex = useLatexForCurrentMessage.value;
      const filename = isLatex ? "message.md" : "message.txt";
      
      // Convert to Base64 (supporting UTF-8)
      const base64Data = btoa(unescape(encodeURIComponent(content)));
      
      import('@tauri-apps/api/core').then(({ invoke }) => {
        invoke<string>("save_clipboard_file", { base64Data, filename }).then(filePath => {
          if (filePath) {
            emit('select-share-file', filePath);
            messageInput.value = "";
            useLatexForCurrentMessage.value = defaultRenderLatex.value;
          }
        }).catch(err => {
          console.error("Failed to save long text as file", err);
          showToast("保存文件失败", "error");
        });
      });
    }
    return;
  }
```

- [ ] **Step 2: Commit**

```bash
git add src/components/ChatArea.vue
git commit -m "feat: convert >2000 chars text to file on send"
```
