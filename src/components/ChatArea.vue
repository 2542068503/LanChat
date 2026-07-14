<template>
  <div class="chat-workspace">
    <div v-if="!activePeerId" style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">
    </div>

    <div v-else class="chat-panel" style="display: flex; flex-direction: column; height: 100%;">
      <!-- HEADER -->
      <div class="chat-header" data-tauri-drag-region>
        <div class="chat-header-user" data-tauri-drag-region>
          <span class="header-name" data-tauri-drag-region>{{ activePeerId === 'lobby' ? '大厅群聊' : (peerProfiles[activePeerId]?.remark || activePeer?.username || '未知联系人') }}</span>
          <span class="header-status" v-if="activePeerId !== 'lobby' && activePeer?.isOnline">
            <div class="pulse-dot"></div>在线
          </span>
        </div>
        <div style="margin-left: auto;">
          <button style="background: none; border: none; color: var(--text-secondary); cursor: pointer; display: flex; align-items: center; justify-content: center;" @click="$emit('clear-history')" title="清空聊天记录"><Trash2 :size="18" /></button>
        </div>
      </div>

      <!-- MESSAGES FLOW -->
      <div class="messages-flow" ref="msgHistoryRef" @scroll="saveScrollPosition">
        <div v-if="activeMessages.length === 0" style="text-align: center; color: var(--text-secondary); margin-top: 40px;">
          暂无消息记录
        </div>

        <template v-for="(msg, index) in activeMessages" :key="msg.messageId">
          <div v-if="shouldShowDateSeparator(msg, index)" class="date-separator">
            {{ formatDateSeparator(msg.timestamp) }}
          </div>
          <div 
            class="message-wrapper"
            :class="{ mine: msg.senderId === selfInfo.id }"
            @contextmenu="handleContextMenu($event, msg)"
          >
          <div class="message-content-container">
            <div class="msg-avatar-container" style="cursor: pointer;" @click="$emit('show-detail', getSenderPeer(msg.senderId))">
               <img v-if="getSenderProfile(msg.senderId, msg).avatarBase64" :src="getSenderProfile(msg.senderId, msg).avatarBase64" class="msg-avatar-img" />
               <div v-else class="msg-avatar-fallback" :style="getAvatarStyle(getSenderProfile(msg.senderId, msg).avatarId)">
                 {{ getInitials(getSenderProfile(msg.senderId, msg).username) }}
               </div>
            </div>
            
            <div style="display:flex; flex-direction: column; min-width: 0;">
              <div class="msg-bubble" :class="{ 'file-bubble': msg.contentType === 'file' }">
                
                <div class="msg-quote" v-if="msg.quoteMsgId" style="border-left: 2px solid var(--accent-color); padding-left: 8px; margin-bottom: 8px; opacity: 0.8; font-size: 11px;">
                  <div>回复 {{ msg.quoteSender }}</div>
                  <div>{{ msg.quoteContent }}</div>
                </div>

                <div v-if="msg.contentType === 'text'" class="msg-text" style="white-space: pre-wrap; word-break: break-word; position: relative;">
                  <div :style="{ 
                    maxHeight: (!expandedLatex[msg.messageId] && (msg.content.length > 200 || msg.content.split('\n').length > 5)) ? '150px' : 'none', 
                    overflow: 'hidden',
                    maskImage: (!expandedLatex[msg.messageId] && (msg.content.length > 200 || msg.content.split('\n').length > 5)) ? 'linear-gradient(to bottom, black 50%, transparent 100%)' : 'none',
                    WebkitMaskImage: (!expandedLatex[msg.messageId] && (msg.content.length > 200 || msg.content.split('\n').length > 5)) ? 'linear-gradient(to bottom, black 50%, transparent 100%)' : 'none'
                  }">
                    <template v-if="msg.renderLatex">
                      <div class="markdown-body" v-html="renderContent(msg.content)"></div>
                    </template>
                    <template v-else>{{ msg.content }}</template>
                  </div>
                  
                  <div v-if="msg.content.split('\n').length > 5 || msg.content.length > 200" style="text-align: center; margin-top: 6px; padding: 4px; background: rgba(0,0,0,0.05); border-radius: 8px;">
                    <button 
                      @click="toggleLatex(msg.messageId)" 
                      style="background: var(--accent-color); padding: 4px 16px; border-radius: 12px; border: none; color: white; cursor: pointer; font-size: 12px; font-weight: bold; box-shadow: 0 2px 8px rgba(0,0,0,0.2);"
                    >
                      {{ expandedLatex[msg.messageId] ? '收起长消息' : '展开长消息' }}
                    </button>
                  </div>
                </div>
                
                <div v-else-if="msg.contentType === 'file' && msg.fileInfo">
                  <div v-if="isImage(msg.fileInfo.name)" style="max-width: 250px;">
                    <div v-if="msg.localPath" style="display: flex; flex-direction: column; gap: 4px; align-items: flex-start;">
                      <img 
                          :src="getAssetUrl(msg.localPath)" 
                          style="max-width: 100%; border-radius: 8px; cursor: pointer; border: 2px solid var(--border-color, rgba(0,0,0,0.1)); box-shadow: 0 4px 12px rgba(0,0,0,0.15); box-sizing: border-box;" 
                          @click="$emit('open-file', msg.localPath)" 
                        />
                      <div style="display: flex; gap: 6px;">
                        <button @click="$emit('open-file', msg.localPath)" class="file-action-btn" style="padding: 2px 6px; font-size: 11px; margin: 0; opacity: 0.8;">在外部打开</button>
                        <button @click="$emit('download-file', msg)" class="file-action-btn" style="padding: 2px 6px; font-size: 11px; margin: 0; opacity: 0.8;">下载图片</button>
                      </div>
                    </div>
                    <div v-else>
                      <div v-if="msg.isDownloading" style="font-size: 11px; color: var(--accent-color);">
                        正在下载... {{ Math.round(((msg.downloadBytesProcessed || 0) / (msg.downloadTotalBytes || msg.fileInfo.size || 1)) * 100) }}%
                      </div>
                      <button v-else @click="$emit('download-file', msg)" class="file-action-btn">下载图片</button>
                    </div>
                  </div>
                  
                  <div v-else class="file-card">
                    <div class="file-card-header">
                       <div class="file-card-icon-wrapper">
                         <FileText :size="32" style="color: inherit; opacity: 0.8;" />
                       </div>
                       <div class="file-card-meta">
                         <span class="file-card-name">{{ msg.fileInfo.name }}</span>
                         <span class="file-card-size">{{ formatFileSize(msg.fileInfo.size) }}</span>
                       </div>
                    </div>
                    <div class="file-card-action" style="display: flex; align-items: center; justify-content: flex-end; width: 100%;">
                      <div v-if="msg.isDownloading" style="flex: 1; margin-right: 8px;">
                        <div style="font-size: 11px; margin-bottom: 4px; display: flex; justify-content: space-between;">
                          <span style="color: var(--accent-color);">{{ msg.downloadSpeed || '计算中...' }}</span>
                          <span style="color: var(--text-secondary);">{{ formatFileSize(msg.downloadBytesProcessed || 0) }} / {{ formatFileSize(msg.downloadTotalBytes || msg.fileInfo.size || 0) }}</span>
                        </div>
                        <div style="width: 100%; height: 4px; background: rgba(0,0,0,0.1); border-radius: 8px; overflow: hidden;">
                          <div style="height: 100%; background: var(--accent-color); transition: width 0.2s;" :style="{ width: ((msg.downloadBytesProcessed || 0) / (msg.downloadTotalBytes || msg.fileInfo.size || 1)) * 100 + '%' }"></div>
                        </div>
                      </div>
                      <button v-else-if="!msg.localPath" class="file-action-btn" @click="$emit('download-file', msg)">
                        下载文件
                      </button>
                      <template v-else>
                        <button class="open-file-btn" @click="$emit('open-file', msg.localPath)">打开文件</button>
                      </template>
                    </div>
                  </div>
                </div>
              </div>
              <span class="msg-time-footer">{{ formatTime(msg.timestamp) }}</span>
            </div>
          </div>
          </div>
        </template>
      </div>

      <!-- HORIZONTAL RESIZER -->
      <div class="horizontal-resizer" @mousedown="startResizeInput"></div>

      <!-- INPUT AREA -->
      <div class="chat-input-bar" :style="{ height: inputHeight + 'px', flex: 'none' }">
        <div class="reply-bar" v-if="replyTo" style="font-size: 11px; color: var(--text-secondary); background: rgba(0,0,0,0.2); padding: 4px 8px; border-radius: 8px; display: flex; justify-content: space-between;">
          <span>回复 {{ replyTo.senderName || replyTo.senderId }}: {{ replyTo.content }}</span>
          <button @click="cancelReply" style="background: none; border: none; color: white; cursor: pointer; display: flex; align-items: center; justify-content: center;"><X :size="14" /></button>
        </div>

        <!-- NEW TOOLBAR -->
        <div class="input-toolbar" style="display: flex; gap: 16px; padding: 0 4px;">
          <button class="icon-btn" @click="$emit('select-share-file')" title="发送文件" style="background: none; border: none; cursor: pointer; color: var(--text-secondary); display: flex; align-items: center; justify-content: center; padding: 4px;">
            <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
          </button>
          <button class="icon-btn" @click="useLatexForCurrentMessage = !useLatexForCurrentMessage" :style="{ color: useLatexForCurrentMessage ? 'var(--accent-color, #10b981)' : 'var(--text-secondary)' }" title="LaTeX / Markdown" style="background: none; border: none; cursor: pointer; display: flex; align-items: center; justify-content: center; padding: 4px;">
            <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19V5a2 2 0 0 1 2-2h13.4a1.5 1.5 0 0 1 1.5 1.5v11L14 22H6a2 2 0 0 1-2-2z"></path><path d="M9 10l3-3 3 3"></path><path d="M12 7v8"></path></svg>
          </button>
        </div>

        <div class="input-container" style="flex: 1; display: flex; flex-direction: column;">
          <div class="textarea-wrapper" style="flex: 1;">
              <textarea 
                class="message-textarea"
                v-model="messageInput" 
                placeholder="输入消息，Enter 发送，Ctrl/Shift+Enter 换行..."
                @keydown.enter.exact="handleEnter"
                @keydown.ctrl.enter="handleNewLine"
                style="height: 100%;"
                maxlength="2000"
              ></textarea>
          </div>
          <div class="input-footer" style="display: flex; justify-content: flex-end; align-items: center; padding-top: 8px;">
            <span style="font-size: 11px; color: var(--text-secondary); opacity: 0.7; margin-right: 12px;" :style="{ color: messageInput.length >= 2000 ? '#ef4444' : '' }">
              {{ messageInput.length }} / 2000
            </span>
            <button class="send-btn" @click="sendMsg" :disabled="!messageInput.trim()">发送</button>
          </div>
        </div>
      </div>
    </div>
    
    <Teleport to="body">
      <MsgContextMenu
        :show="contextMenuData.show"
        :x="contextMenuData.x"
        :y="contextMenuData.y"
        :msg="contextMenuData.msg"
        :selectedText="contextMenuData.selectedText"
        @close="contextMenuData.show = false"
        @reply="handleReply"
        @copy="(msg: any, text: string) => handleCopyText(msg, text)"
      />
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import katex from 'katex';
import 'katex/dist/katex.min.css';
import { Trash2, FileText, X } from 'lucide-vue-next';
import { showToast } from '../store';

import { 
  activePeerId, activePeer, activeMessages,
  selfInfo, peerProfiles, getSenderPeer, getSenderProfile, scrollPositions
} from '../store';
import MsgContextMenu from './MsgContextMenu.vue';
import { useSettings } from '../composables/useSettings';
import type { Message } from '../types';
import { convertFileSrc } from '@tauri-apps/api/core';

const { defaultRenderLatex } = useSettings();
const useLatexForCurrentMessage = ref(defaultRenderLatex.value);

const msgHistoryRef = ref<HTMLElement | null>(null);

let isSwitchingPeer = false;
const saveScrollPosition = (e: Event) => {
  if (isSwitchingPeer) return;
  const target = e.target as HTMLElement;
  if (target.scrollTop + target.clientHeight >= target.scrollHeight - 10) {
    scrollPositions.value[activePeerId.value] = -1;
  } else {
    scrollPositions.value[activePeerId.value] = target.scrollTop;
  }
};

const scrollToBottom = () => {
  nextTick(() => {
    if (msgHistoryRef.value) {
      const sPos = scrollPositions.value[activePeerId.value];
      if (sPos !== undefined && sPos >= 0) {
        msgHistoryRef.value.scrollTop = sPos;
      } else {
        msgHistoryRef.value.scrollTop = msgHistoryRef.value.scrollHeight;
      }
      setTimeout(() => {
        if (msgHistoryRef.value) {
          const sPos2 = scrollPositions.value[activePeerId.value];
          if (sPos2 !== undefined && sPos2 >= 0) {
            msgHistoryRef.value.scrollTop = sPos2;
          } else {
            msgHistoryRef.value.scrollTop = msgHistoryRef.value.scrollHeight;
          }
        }
      }, 50);
      setTimeout(() => {
        if (msgHistoryRef.value) {
          const sPos3 = scrollPositions.value[activePeerId.value];
          if (sPos3 !== undefined && sPos3 >= 0) {
            msgHistoryRef.value.scrollTop = sPos3;
          } else {
            msgHistoryRef.value.scrollTop = msgHistoryRef.value.scrollHeight;
          }
        }
      }, 200);
    }
  });
};

watch(activePeerId, (_newId, oldId) => {
  if (oldId && msgHistoryRef.value) {
    if (msgHistoryRef.value.scrollTop + msgHistoryRef.value.clientHeight >= msgHistoryRef.value.scrollHeight - 10) {
      scrollPositions.value[oldId] = -1;
    } else {
      scrollPositions.value[oldId] = msgHistoryRef.value.scrollTop;
    }
  }
  isSwitchingPeer = true;
  setTimeout(() => isSwitchingPeer = false, 300);
  scrollToBottom();
}, { flush: 'pre' });

watch(activeMessages, () => {
  if (isSwitchingPeer) return;
  scrollToBottom();
}, { deep: true });

onMounted(() => {
  scrollToBottom();
  window.addEventListener('paste', handlePaste);
});

onUnmounted(() => {
  window.removeEventListener('paste', handlePaste);
});

const expandedLatex = ref<Record<string, boolean>>({});
function toggleLatex(msgId: string) {
  expandedLatex.value[msgId] = !expandedLatex.value[msgId];
}

const shouldShowDateSeparator = (msg: Message, index: number) => {
  if (index === 0) return true;
  const prevMsg = activeMessages.value[index - 1];
  const date1 = new Date(prevMsg.timestamp).toDateString();
  const date2 = new Date(msg.timestamp).toDateString();
  return date1 !== date2;
};

const formatDateSeparator = (timestamp: number) => {
  const date = new Date(timestamp);
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
};

const getAssetUrl = (path: string) => {
  return convertFileSrc(path);
};

const emit = defineEmits([
  'show-detail', 'clear-history', 'open-preview', 'download-file', 'open-file',
  'select-share-file', 'send-message', 'paste-image'
]);

const props = defineProps({
  replyTo: Object
});

const messageInput = ref("");
const lastSendTime = ref(0);

watch(messageInput, (newVal) => {
  const lines = newVal.split('\n');
  if (lines.length > 50) {
    messageInput.value = lines.slice(0, 50).join('\n');
    showToast("输入不能超过 50 行", "error");
  }
});

const inputHeight = ref(150);
let isResizingInput = false;

const startResizeInput = () => {
  isResizingInput = true;
  document.body.style.cursor = 'row-resize';
  document.body.style.userSelect = 'none';
  document.body.style.webkitUserSelect = 'none';
  document.addEventListener('mousemove', handleResizeInput);
  document.addEventListener('mouseup', stopResizeInput);
};

const handleResizeInput = (e: MouseEvent) => {
  if (!isResizingInput) return;
  e.preventDefault();
  const chatPanel = document.querySelector('.chat-panel') as HTMLElement;
  if (!chatPanel) return;
  
  // Calculate zoom factor
  const zoomFactor = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--global-zoom')) || 1;
  
  const panelRect = chatPanel.getBoundingClientRect();
  const newHeight = (panelRect.bottom - e.clientY) / zoomFactor;
  
  if (newHeight > 100 && newHeight < 600) {
    inputHeight.value = newHeight;
  }
};

const stopResizeInput = () => {
  isResizingInput = false;
  document.body.style.cursor = '';
  document.body.style.userSelect = '';
  document.body.style.webkitUserSelect = '';
  document.removeEventListener('mousemove', handleResizeInput);
  document.removeEventListener('mouseup', stopResizeInput);
};

const handlePaste = (e: ClipboardEvent) => {
  if (e.clipboardData && e.clipboardData.items) {
    const items = e.clipboardData.items;
    for (let i = 0; i < items.length; i++) {
      if (items[i].type.indexOf('image') !== -1) {
        const file = items[i].getAsFile();
        if (file) {
          e.preventDefault();
          const reader = new FileReader();
          reader.onload = (event) => {
            if (event.target && event.target.result) {
              emit('paste-image', event.target.result as string);
            }
          };
          reader.readAsDataURL(file);
          return;
        }
      } else if (items[i].kind === 'file') {
        const file = items[i].getAsFile();
        if (file && file.size > 0 && file.size < 100 * 1024 * 1024) { // Max 100MB for temp save
          e.preventDefault();
          const reader = new FileReader();
          reader.onload = async (event) => {
            if (event.target && event.target.result) {
              const buffer = event.target.result as ArrayBuffer;
              const bytes = new Uint8Array(buffer);
              let binary = '';
              const chunkSize = 8192;
              for (let j = 0; j < bytes.length; j += chunkSize) {
                binary += String.fromCharCode.apply(null, bytes.subarray(j, j + chunkSize) as any);
              }
              const base64Data = btoa(binary);
              
              // Now we can emit this to a new event 'paste-file-buffer'
              // but actually it's easier to just save it via invoke right here
              try {
                const { invoke } = await import('@tauri-apps/api/core');
                const filePath = await invoke<string>("save_clipboard_file", { base64Data, filename: file.name });
                if (filePath) {
                  emit('select-share-file', filePath);
                }
              } catch(err) {
                console.error("Failed to save clipboard file", err);
              }
            }
          };
          reader.readAsArrayBuffer(file);
          return;
        }
      }
    }
  }
};

const contextMenuData = ref({ show: false, x: 0, y: 0, msg: null as any, selectedText: "" });

function handleContextMenu(event: MouseEvent, msg: Message) {
  event.preventDefault(); // Always prevent default so our custom menu handles it
  const selectedText = window.getSelection()?.toString() || "";
  
  contextMenuData.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
    msg,
    selectedText
  };
}

function handleReply(msg: Message) {
  emit('send-message', { type: 'reply', msg });
}

function handleCopyText(msg: Message, text?: string) {
  if (msg.contentType === 'text') {
    navigator.clipboard.writeText(text || msg.content);
  }
}

function cancelReply() {
  emit('send-message', { type: 'reply', msg: null });
}

function handleEnter(e: KeyboardEvent) {
  // Prevent sending and prevent default ONLY if not composing (i.e. not using IME)
  if (e.isComposing || (e as any).keyCode === 229) {
    return;
  }
  e.preventDefault();
  sendMsg();

  // Fallback: Some browsers/IMEs might still inject a newline on keyup/keypress after sending
  setTimeout(() => {
    if (messageInput.value.trim() === '') {
      messageInput.value = '';
    }
  }, 10);
}

function handleNewLine(e: KeyboardEvent) {
  const target = e.target as HTMLTextAreaElement;
  if (!target) return;
  const start = target.selectionStart;
  const end = target.selectionEnd;
  messageInput.value = messageInput.value.substring(0, start) + '\n' + messageInput.value.substring(end);
  nextTick(() => {
    target.selectionStart = target.selectionEnd = start + 1;
  });
}

function sendMsg() {
  const content = messageInput.value.trim();
  if (!content) return;
  
  const now = Date.now();
  if (now - lastSendTime.value < 500) {
    showToast("发送太频繁，请稍后再试", "error");
    return;
  }
  lastSendTime.value = now;

  if (content.length > 2000) {
    if (window.confirm("文本过长(超过2000字符)，是否将其转换为文件发送？")) {
      const isLatex = useLatexForCurrentMessage.value;
      const filename = isLatex ? "message.md" : "message.txt";
      
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
  
  emit('send-message', { 
    content, 
    type: 'text', 
    isLatex: useLatexForCurrentMessage.value,
    onSuccess: () => {
      messageInput.value = "";
      useLatexForCurrentMessage.value = defaultRenderLatex.value;
    }
  });
}

function isImage(filename: string) {
  return /\.(jpg|jpeg|png|gif|webp)$/i.test(filename);
}

function formatTime(timestamp: number) {
  return new Date(timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

function getInitials(name: string) {
  if (!name) return "U";
  const trimmed = name.trim();
  if (!trimmed) return "U";
  if (/[\u4e00-\u9fa5]/.test(trimmed)) {
    return trimmed.slice(-2);
  }
  const parts = trimmed.split(/[\s_-]+/).filter(p => p.length > 0);
  if (parts.length > 1) {
    return (parts[0][0] + parts[1][0]).toUpperCase();
  } else if (parts.length === 1) {
    return parts[0].slice(0, 2).toUpperCase();
  }
  return trimmed.slice(0, 2).toUpperCase();
}

function getAvatarStyle(id: number) {
  const colors = [
    'linear-gradient(135deg, #FF6B6B 0%, #FF8E8B 100%)',
    'linear-gradient(135deg, #4FACFE 0%, #00F2FE 100%)',
    'linear-gradient(135deg, #43E97B 0%, #38F9D7 100%)',
    'linear-gradient(135deg, #FA709A 0%, #FEE140 100%)',
    'linear-gradient(135deg, #a18cd1 0%, #fbc2eb 100%)',
    'linear-gradient(135deg, #ff9a9e 0%, #fecfef 100%)',
    'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    'linear-gradient(135deg, #89f7fe 0%, #66a6ff 100%)',
    'linear-gradient(135deg, #fdfbfb 0%, #ebedee 100%)'
  ];
  return { background: colors[id % colors.length] || colors[0] };
}

function formatFileSize(bytes: number) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function renderContent(content: string) {
  // Pre-process for KaTeX
  let processed = content
    .replace(/\$\$(.*?)\$\$/gs, (match, p1) => {
      try { return katex.renderToString(p1, { displayMode: true, throwOnError: false }); } 
      catch (e) { return match; }
    })
    .replace(/\$(.*?)\$/g, (match, p1) => {
      try { return katex.renderToString(p1, { displayMode: false, throwOnError: false }); } 
      catch (e) { return match; }
    });

  // Convert markdown to HTML
  const rawHtml = marked.parse(processed, { breaks: true });
  // Sanitize
  return DOMPurify.sanitize(rawHtml as string).trim();
}
</script>
