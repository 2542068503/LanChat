# Chat Input Limits and Enhancements Design

## 1. Rate Limiting
**Requirement**: Prevent users from sending messages with less than 0.5s interval between them.
**Design**: 
- Add a `lastSendTime` reactive reference in `ChatArea.vue` (initialized to 0).
- Inside `sendMsg()`, before processing the message, check if `Date.now() - lastSendTime.value < 500`.
- If true, display an error toast: "发送太频繁，请稍后再试" (Sending too fast, please wait) and abort the send.
- If false, proceed with sending and update `lastSendTime.value = Date.now()`.

## 2. Input Box Row Limit
**Requirement**: Add a row limit to the input box to prevent typing or pasting more than 50 lines.
**Design**:
- Add a `watch` on the `messageInput` reactive reference.
- When `messageInput` changes, split the new value by `\n` to check the number of lines.
- If the line count exceeds 50:
  - Slice the array to 50 lines and join it back with `\n`.
  - Update `messageInput.value` with the truncated string.
  - Display a warning toast: "输入不能超过 50 行" (Input cannot exceed 50 lines).
- Note: This ensures that both typing and pasting are caught, and the text never physically exceeds 50 lines in the input box state.

## 3. Long Text to File Conversion
**Requirement**: If the sent text is too long (over 2000 characters), prompt the user to send it as a `.txt` file. If the message uses LaTeX rendering, use a `.md` file instead.
**Design**:
- In `sendMsg()`, remove the current hard block for `content.length > 2000`.
- Instead, if `content.length > 2000`, open a confirmation dialog (e.g., using `window.confirm("文本过长(超过2000字符)，是否将其转换为文件发送？")`).
- If the user confirms:
  - Determine the filename: `useLatexForCurrentMessage.value ? 'message.md' : 'message.txt'`.
  - Convert the `content` string to a Base64 encoded string supporting UTF-8 (e.g., `btoa(unescape(encodeURIComponent(content)))`).
  - Invoke the existing Tauri command `save_clipboard_file`:
    ```ts
    const filePath = await invoke<string>("save_clipboard_file", { base64Data, filename });
    ```
  - Emit the existing `select-share-file` event with the `filePath`:
    ```ts
    emit('select-share-file', filePath);
    ```
  - Clear the input box and reset latex state (similar to standard success).
- If the user cancels, do nothing.

## Testing and Verification
- Attempt to send messages rapidly and ensure the toast appears and messages are dropped.
- Paste 60 lines of text into the input box and verify it gets truncated to 50 lines with a toast warning.
- Type/Paste > 2000 characters and send. Verify the confirmation dialog appears. Confirm, and verify a file is sent with the correct extension based on the LaTeX toggle state.
