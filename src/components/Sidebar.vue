<template>
  <div class="peers-pane">
    <div class="search-bar">
      <span class="search-icon">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
      </span>
      <input type="text" v-model="searchInput" placeholder="搜索联系人..." class="search-input" />
      <button @click="promptAddPeer" title="手动添加跨网段IP" style="background: none; border: none; color: var(--text-secondary); cursor: pointer; padding: 4px; display: flex; align-items: center; justify-content: center; margin-left: 4px;">
        <Plus :size="16" />
      </button>
    </div>

    <div class="peers-list">
      <div 
        v-for="peer in filteredPeers" 
        :key="peer.id" 
        class="peer-item" 
        :class="{ active: activePeerId === peer.id }"
        @click="$emit('select-peer', peer.id)"
        @contextmenu.prevent="showContextMenu($event, peer)"
      >
        <div class="avatar-container">
          <template v-if="peer.id === 'lobby'">
            <div class="peer-avatar-fallback" style="background: linear-gradient(135deg, #0ea5e9 0%, #3b82f6 50%, #8b5cf6 100%); background-size: 200% 200%; color: white; box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4); border: 2px solid rgba(255, 255, 255, 0.2); animation: gradientMove 3s ease infinite;">
              <Globe :size="22" stroke-width="1.5" />
            </div>
          </template>
          <template v-else>
            <img v-if="peerProfiles[peer.id]?.avatarBase64 && peerProfiles[peer.id]?.avatarId === 0" :src="peerProfiles[peer.id].avatarBase64" class="peer-avatar-img" />
            <div v-else class="peer-avatar-fallback" :style="getAvatarStyle(peerProfiles[peer.id]?.avatarId || 1)">
              {{ getInitials(peerProfiles[peer.id]?.remark || peer.username) }}
            </div>
          </template>
          <span class="status-indicator-dot" :class="peer.isOnline ? 'online' : 'offline'"></span>
        </div>
        
        <div class="peer-details">
          <div class="peer-header">
            <span class="peer-name">
              <span v-if="peerProfiles[peer.id]?.isPinned" style="margin-right: 4px; display: inline-flex; align-items: center;" title="已置顶">
                <Pin :size="14" style="color: var(--text-secondary);" />
              </span>
              {{ peerProfiles[peer.id]?.remark || peer.username }}
            </span>
            <span class="os-badge" :class="peerProfiles[peer.id]?.os?.toLowerCase()" v-if="peerProfiles[peer.id]?.os">{{ peerProfiles[peer.id].os }}</span>
          </div>
          <div class="peer-status" style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
            <span>{{ peer.ip || 'Local' }}</span>
            <div class="unread-badge" v-if="unreadCounts[peer.id] > 0">
              {{ unreadCounts[peer.id] > 99 ? '99+' : unreadCounts[peer.id] }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="contextMenu.show" class="context-menu" :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }">
      <div class="menu-item" @click="handleContextAction('detail')">详细信息</div>
      <div class="menu-item" @click="handleContextAction('pin')">{{ peerProfiles[contextMenu.peer?.id || '']?.isPinned ? '取消置顶' : '置顶联系人' }}</div>
      <div class="menu-divider"></div>
      <div class="menu-item danger" @click="handleContextAction('delete')">删除记录</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Pin, Globe, Plus } from 'lucide-vue-next';
import { searchInput, peers, activePeerId, peerProfiles, unreadCounts, chats, showToast } from '../store';
import type { Peer } from '../types';
import { invoke } from '@tauri-apps/api/core';

const emit = defineEmits(['change-tab', 'select-peer', 'show-detail']);

function promptAddPeer() {
  const ip = prompt("请输入要查找的跨网段好友 IP 地址 (例如: 10.10.1.5):");
  if (ip && ip.trim()) {
    invoke('add_peer_manual', { ip: ip.trim() }).then(() => {
      showToast("已向目标 IP 发送探测信号", "success");
    }).catch(e => {
      showToast("添加失败: " + e, "error");
    });
  }
}

const filteredPeers = computed(() => {
  const query = searchInput.value.toLowerCase().trim();
  let list = [...peers.value].filter(p => p.isOnline || peerProfiles.value[p.id]?.isPinned || (chats.value[p.id] && chats.value[p.id].length > 0));
  
  // Sort pinned first, then online, then offline
  list.sort((a, b) => {
    const aPinned = peerProfiles.value[a.id]?.isPinned || false;
    const bPinned = peerProfiles.value[b.id]?.isPinned || false;
    if (aPinned !== bPinned) return aPinned ? -1 : 1;
    if (a.isOnline !== b.isOnline) return a.isOnline ? -1 : 1;
    return a.username.localeCompare(b.username);
  });

  // Always inject Lobby at the top
  const lobbyPeer: Peer = {
    id: "lobby",
    username: "局域网大厅",
    ip: "All Users",
    lastSeen: new Date().toISOString(),
    isOnline: true,
    avatarId: 0
  };
  
  list.unshift(lobbyPeer);

  if (!query) return list;
  return list.filter(p =>
    p.id === "lobby" || // Always show lobby when searching (or you can filter it out)
    p.username.toLowerCase().includes(query) ||
    (p.remark && p.remark.toLowerCase().includes(query)) ||
    p.ip.includes(query)
  );
});

// Context Menu State
const contextMenu = ref({ show: false, x: 0, y: 0, peer: null as Peer | null });

function showContextMenu(e: MouseEvent, peer: Peer) {
  if (peer.id === 'lobby') return; // No context menu for lobby
  contextMenu.value = { show: true, x: e.clientX, y: e.clientY, peer };
}

function closeContextMenu() {
  contextMenu.value.show = false;
}

function handleContextAction(action: 'detail' | 'pin' | 'delete') {
  if (!contextMenu.value.peer) return;
  const peer = contextMenu.value.peer;
  
  if (action === 'detail') {
    emit('show-detail', peer);
  } else if (action === 'pin') {
    if (!peerProfiles.value[peer.id]) {
      peerProfiles.value[peer.id] = { username: peer.username, avatarId: 1 };
    }
    peerProfiles.value[peer.id].isPinned = !peerProfiles.value[peer.id].isPinned;
    localStorage.setItem("peerProfiles", JSON.stringify(peerProfiles.value));
    } else if (action === 'delete') {
      invoke('delete_peer', { peerId: peer.id }).catch(e => console.error("Failed to delete peer in backend", e));
      delete peerProfiles.value[peer.id];
      delete chats.value[peer.id];
      localStorage.setItem("peerProfiles", JSON.stringify(peerProfiles.value));
      localStorage.setItem("lanchat_chats", JSON.stringify(chats.value));
      if (activePeerId.value === peer.id) activePeerId.value = '';
    }
  closeContextMenu();
}

onMounted(() => {
  document.addEventListener('click', closeContextMenu);
});

onUnmounted(() => {
  document.removeEventListener('click', closeContextMenu);
});

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

function getInitials(name: string) {
  if (!name) return 'U';
  const trimmed = name.trim();
  if (!trimmed) return 'U';
  if (/[\u4e00-\u9fa5]/.test(trimmed)) {
    return trimmed.slice(-2);
  }
  const parts = trimmed.split(/[\s_-]+/);
  if (parts.length > 1) {
    return (parts[0][0] + parts[1][0]).toUpperCase();
  }
  return trimmed.slice(0, 2).toUpperCase();
}
</script>
