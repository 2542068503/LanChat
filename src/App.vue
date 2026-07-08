<template>
  <div class="window-container" :class="{ light: !isDarkTheme, minimal: isMinimalMode }">
    <div class="window-titlebar" @mousedown="handleTitlebarMouseDown" @dblclick="handleTitlebarDblClick">
      <div class="window-title">
        <img :src="globalAppIconUrl" alt="App Icon" style="width: 16px; height: 16px; margin-right: 8px; border-radius: 4px; object-fit: cover;" />
        <span class="title-text">LanChat - {{ selfInfo.username || 'Connecting...' }}</span>
      </div>
      <div class="window-controls">
        <button v-if="!isMinimalMode" class="control-btn minimize" @click="minimizeWindow">
          <Minus :size="14" />
        </button>
        <button class="control-btn maximize" @click="toggleMaximizeWindow">
          <Square :size="12" v-if="!isMaximized" />
          <Copy :size="12" v-else style="transform: rotate(180deg);" />
        </button>
        <button class="control-btn close" @click="closeWindow">
          <X :size="16" />
        </button>
      </div>
    </div>
    
    <div class="app-layout" :style="{ '--sidebar-width': sidebarWidth + 'px' }">
      <!-- 54px Sidebar directly in App.vue -->
      <div class="sidebar">
        <div class="self-avatar-container" @click="currentTab = 'settings'">
          <img v-if="selfInfo.avatarBase64 && selfInfo.avatarId === 0" :src="selfInfo.avatarBase64" class="self-avatar-img" />
          <div v-else class="self-avatar-fallback" :style="getAvatarStyle(selfInfo.avatarId || 1)">
            {{ getInitials(selfInfo.username) }}
          </div>
        </div>
        
        <div class="nav-tabs">
          <button class="nav-tab-btn" :class="{ active: currentTab === 'chat' }" @click="currentTab = 'chat'">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"></path></svg>
          </button>
          <button class="nav-tab-btn" :class="{ active: currentTab === 'settings' }" @click="currentTab = 'settings'">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
          </button>
        </div>
      </div>

      <div class="main-body-container">
        <div class="chats-view-layout" v-if="currentTab === 'chat'">
          <Sidebar 
            @select-peer="selectPeer" 
            @show-detail="showPeerDetail"
            :style="{ width: sidebarWidth + 'px' }"
          />
          <div class="vertical-resizer" @mousedown="startResizeSidebar"></div>
          <ChatArea  
            :replyTo="replyTo"
            @send-message="handleSendMessage"
            @clear-history="showClearCurrentConfirm = true"
            @open-preview="openImagePreview"
            @download-file="downloadFile"
            @open-file="openFile"
            @select-share-file="selectAndShareFile"
            @show-detail="showPeerDetail"
          />
        </div>
        <SettingsView v-else-if="currentTab === 'settings'" />
      </div>
      
      <button v-if="currentTab === 'chat'" class="minimal-toggle-btn" @click="isMinimalMode = !isMinimalMode" :class="{ active: isMinimalMode }" title="极简/透明模式">
        <EyeOff v-if="isMinimalMode" :size="16" />
        <Eye v-else :size="16" />
      </button>
    </div>

    <Modals 
      :showClearCurrentConfirm="showClearCurrentConfirm"
      :showPeerDetailModal="showPeerDetailModal"
      :detailPeer="detailPeer"
      :fileConfirmModal="fileConfirmModal"
      @clear-history-confirm="clearCurrentHistory"
      @file-receive-confirm="handleFileReceiveConfirm"
      @send-file-confirm="handleSendFileConfirm"
      @close-clear-modal="showClearCurrentConfirm = false"
      @close-detail-modal="showPeerDetailModal = false"
      @close-file-modal="fileConfirmModal.show = false"
    />

    <Transition name="toast">
      <div class="toast-container" v-if="globalToast.show">
        <div class="custom-toast" :class="globalToast.type">
          <span class="toast-icon">{{ globalToast.type === 'success' ? '✓' : globalToast.type === 'error' ? '!' : 'i' }}</span>
          <span class="toast-message">{{ globalToast.message }}</span>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';

// Stores and Composables
import { currentTab, selfInfo, activePeerId, chats, globalToast, showToast, globalAppIconUrl, unreadCounts } from './store';
import { useNetwork } from './composables/useNetwork';
import { useChat } from './composables/useChat';
import { useFileTransfer } from './composables/useFileTransfer';
import { useSettings } from './composables/useSettings';

// Components
import Sidebar from './components/Sidebar.vue';
import ChatArea from './components/ChatArea.vue';
import SettingsView from './components/SettingsView.vue';
import Modals from './components/Modals.vue';
import type { Peer } from './types';
import { Minus, Square, Copy, X, Eye, EyeOff } from 'lucide-vue-next';

const isMinimalMode = ref(false);
const appWindow = getCurrentWindow();

// Global State refs for App.vue
const isMaximized = ref(false);
const showClearCurrentConfirm = ref(false);
const showPeerDetailModal = ref(false);
const detailPeer = ref<Peer | null>(null);
const fileConfirmModal = ref({ show: false, msg: null as any });
const replyTo = ref<any>(null);

const sidebarWidth = ref(250);
let isResizingSidebar = false;

const startResizeSidebar = () => {
  isResizingSidebar = true;
  document.body.style.cursor = 'col-resize';
  document.body.style.userSelect = 'none';
  (document.body.style as any).webkitUserSelect = 'none';
  document.addEventListener('mousemove', handleResizeSidebar);
  document.addEventListener('mouseup', stopResizeSidebar);
};

const handleResizeSidebar = (e: MouseEvent) => {
  if (!isResizingSidebar) return;
  e.preventDefault();
  const appLayout = document.querySelector('.app-layout') as HTMLElement;
  if (!appLayout) return;
  
  // Calculate zoom factor
  const zoomFactor = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--global-zoom')) || 1;
  
  const layoutRect = appLayout.getBoundingClientRect();
  // Sidebar starts after the 54px leftmost sidebar (which is scaled by zoom!)
  const sidebarLeft = layoutRect.left + 54 * zoomFactor;
  const newWidth = (e.clientX - sidebarLeft) / zoomFactor;
  
  if (newWidth > 180 && newWidth < 500) {
    sidebarWidth.value = newWidth;
  }
};

const stopResizeSidebar = () => {
  isResizingSidebar = false;
  document.body.style.cursor = '';
  document.body.style.userSelect = '';
  (document.body.style as any).webkitUserSelect = '';
  document.removeEventListener('mousemove', handleResizeSidebar);
  document.removeEventListener('mouseup', stopResizeSidebar);
};

const { fetchSelfInfo, setupNetworkListeners, loadProfilesFromLocalStorage } = useNetwork();
const { setupChatListeners, sendMessage, loadChatsFromLocalStorage } = useChat();
const { setupFileListeners, selectAndShareFile, sendConfirmedFile, downloadFile, openFile, openImagePreview, autoDownloadImage } = useFileTransfer();
const { initSettings, isDarkTheme, enableCtrlWClose, silentStartup } = useSettings();

watch(isMinimalMode, async (val) => {
  await appWindow.setSkipTaskbar(val);
});

function getInitials(name: string) {
  if (!name) return "U";
  const trimmed = name.trim();
  if (!trimmed) return "U";
  if (/[\u4e00-\u9fa5]/.test(trimmed)) {
    return trimmed.slice(-2);
  }
  const parts = trimmed.split(/[\s_-]+/);
  if (parts.length > 1) {
    return (parts[0][0] + parts[1][0]).toUpperCase();
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

// Window Controls
let isMouseDown = false;

function handleTitlebarMouseDown(event: MouseEvent) {
  if ((event.target as HTMLElement).closest('.window-controls')) return;
  isMouseDown = true;
  window.addEventListener('mousemove', handleTitlebarMouseMove);
  window.addEventListener('mouseup', handleMouseUp);
}

function handleMouseUp() {
  isMouseDown = false;
  window.removeEventListener('mousemove', handleTitlebarMouseMove);
  window.removeEventListener('mouseup', handleMouseUp);
}

function handleTitlebarMouseMove() {
  if (isMouseDown) {
    isMouseDown = false; 
    appWindow.startDragging();
  }
}

function handleTitlebarDblClick(event: MouseEvent) {
  if (!(event.target as HTMLElement).closest('.window-controls')) {
    toggleMaximizeWindow();
  }
}

function minimizeWindow() { appWindow.minimize(); }
function closeWindow() { appWindow.hide(); }
async function toggleMaximizeWindow() {
  const isMax = await appWindow.isMaximized();
  if (isMax) {
    await appWindow.unmaximize();
    isMaximized.value = false;
  } else {
    await appWindow.maximize();
    isMaximized.value = true;
  }
}

// Peer Actions
function selectPeer(peerId: string) {
  activePeerId.value = peerId;
  currentTab.value = "chat";
  unreadCounts.value[peerId] = 0;
}

function showPeerDetail(peer: Peer) {
  detailPeer.value = peer;
  showPeerDetailModal.value = true;
}

// Chat Actions
async function handleSendMessage(payload: any) {
  if (payload.type === 'reply') {
    replyTo.value = payload.msg;
  } else if (payload.type === 'text') {
    const success = await sendMessage(payload.content, "text", payload.isLatex, replyTo.value);
    if (success) {
      replyTo.value = null;
      if (payload.onSuccess) payload.onSuccess();
    } else {
      showToast("发送失败，对方可能已离线", "error");
    }
  }
}

function clearCurrentHistory() {
  if (activePeerId.value) {
    chats.value[activePeerId.value] = [];
  }
  showClearCurrentConfirm.value = false;
}

async function handleFileReceiveConfirm(action: 'accept' | 'reject') {
  const msg = fileConfirmModal.value.msg;
  if (!msg) return;
  
  if (action === 'accept') {
    try {
      await downloadFile(msg);
      msg.isDownloading = true; // State is managed in useFileTransfer mostly, just stub here
    } catch (e) {
      showToast("Download failed", "error");
    }
  }
  fileConfirmModal.value.show = false;
}

async function handleSendFileConfirm() {
  await sendConfirmedFile();
}

// Lifecycle
onMounted(async () => {
  loadProfilesFromLocalStorage();
  loadChatsFromLocalStorage();
  
  await fetchSelfInfo();
  initSettings();

  if (!silentStartup.value) {
    // Slight delay to prevent flickering during initial setup
    setTimeout(async () => {
      await appWindow.show();
      await appWindow.setFocus();
    }, 50);
  }

  await setupNetworkListeners();
  await setupChatListeners();
  await setupFileListeners();

  await listen<any>("file-receive-request", (event) => {
    autoDownloadImage(event.payload);
  });

  isMaximized.value = await appWindow.isMaximized();
  await appWindow.onResized(async () => {
    isMaximized.value = await appWindow.isMaximized();
  });
  
  // Custom context menu logic for blocking default
  document.addEventListener("contextmenu", handleContextMenu);

  // Global keydown listener for shortcuts
  document.addEventListener("keydown", handleGlobalKeydown);
});

onUnmounted(() => {
  document.removeEventListener("contextmenu", handleContextMenu);
  document.removeEventListener("keydown", handleGlobalKeydown);
});

function handleGlobalKeydown(e: KeyboardEvent) {
  if (e.ctrlKey && e.key.toLowerCase() === 'w' && enableCtrlWClose.value) {
    e.preventDefault();
    closeWindow();
  }
}

function handleContextMenu(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (target && target.closest && (target.closest('.msg-bubble') || target.tagName === 'INPUT' || target.tagName === 'TEXTAREA')) {
    return;
  }
  e.preventDefault();
}
</script>

<style>
/* CSS will be migrated to src/assets/main.css or retained here */
@import './assets/main.css';
</style>
