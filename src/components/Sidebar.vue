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
      <button @click="refreshNetwork" title="刷新好友状态" style="background: none; border: none; color: var(--text-secondary); cursor: pointer; padding: 4px; display: flex; align-items: center; justify-content: center; margin-left: 2px;" :class="{'spin-anim': isRefreshing}">
        <RefreshCw :size="14" />
      </button>
    </div>

    <div class="peers-list">
      <template v-for="group in peerGroups" :key="group.id">
        <div class="peer-group" v-if="group.peers.length > 0">
          <div class="group-header" @click="toggleGroup(group.id)" v-if="group.id !== 'lobby'">
            <ChevronRight :class="{'expanded': !collapsedGroups[group.id]}" :size="16" class="group-chevron" />
            <span style="flex: 1;">{{ group.title }}</span>
            <span class="group-count">{{ group.peers.length }}</span>
          </div>
          <div class="group-content" v-show="group.id === 'lobby' || !collapsedGroups[group.id]">
            <div 
              v-for="peer in group.peers" 
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
                  <div class="peer-badges">
                    <span class="os-badge" :class="formatOS(peerProfiles[peer.id]?.os)" v-if="peerProfiles[peer.id]?.os">{{ formatOS(peerProfiles[peer.id].os) }}</span>
                    <span class="state-badge active" v-if="peer.appState === 'active' && peer.isOnline">活跃</span>
                    <span class="state-badge background" v-else-if="peer.appState === 'background' && peer.isOnline">后台</span>
                  </div>
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
        </div>
      </template>
    </div>

    <div v-if="contextMenu.show" class="custom-context-menu" :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }">
      <div class="context-item" @click="handleContextAction('detail')">详细信息</div>
      <div class="context-item" @click="handleContextAction('pin')">{{ peerProfiles[contextMenu.peer?.id || '']?.isPinned ? '取消置顶' : '置顶联系人' }}</div>
      <div class="context-divider"></div>
      <div class="context-item danger" @click="handleContextAction('delete')">删除记录</div>
    </div>

    <Teleport to="body">
      <Transition name="modal">
        <div class="modal-overlay" v-if="showAddPeerModal" @click.self="showAddPeerModal = false">
          <div class="modal-card" style="width: 320px;">
            <div class="modal-header">
              <h3>添加跨网段联系人</h3>
              <button class="modal-close-btn" @click="showAddPeerModal = false"><X :size="18" /></button>
            </div>
            <div class="modal-body" style="padding: 20px;">
              <p style="margin-bottom: 12px; font-size: 13px; color: var(--text-secondary);">请输入对方的 IP 地址 (例如: 10.10.1.5)</p>
              <input type="text" v-model="manualPeerIp" placeholder="输入 IP 地址..." class="search-input" style="width: 100%; box-sizing: border-box; padding: 10px; font-size: 14px; background: rgba(0,0,0,0.1);" @keyup.enter="confirmAddPeer" />
            </div>
            <div class="modal-footer" style="padding: 16px 20px; display: flex; justify-content: flex-end; gap: 12px; background: rgba(0,0,0,0.02); border-top: 1px solid var(--border-color);">
              <button class="btn secondary" @click="showAddPeerModal = false" style="background: rgba(255,255,255,0.1); border: 1px solid rgba(255,255,255,0.1); padding: 6px 16px; border-radius: 6px; color: var(--text-primary); cursor: pointer;">取消</button>
              <button class="btn primary" @click="confirmAddPeer" :disabled="!manualPeerIp.trim()" style="background: var(--accent-color); border: none; padding: 6px 16px; border-radius: 6px; color: white; cursor: pointer;">确认添加</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Pin, Globe, Plus, ChevronRight, RefreshCw, X } from 'lucide-vue-next';
import { searchInput, peers, activePeerId, peerProfiles, unreadCounts, chats, showToast, selfInfo } from '../store';
import type { Peer } from '../types';
import { invoke } from '@tauri-apps/api/core';

const emit = defineEmits(['change-tab', 'select-peer', 'show-detail']);

const isRefreshing = ref(false);

async function refreshNetwork() {
  if (isRefreshing.value) return;
  isRefreshing.value = true;
  try {
    await invoke('refresh_discovery');
    await invoke('scan_subnets');
    showToast("已广播刷新请求并扫描网段", "success");
  } catch (e: any) {
    showToast("刷新失败: " + e, "error");
  } finally {
    setTimeout(() => {
      isRefreshing.value = false;
    }, 1000);
  }
}

const showAddPeerModal = ref(false);
const manualPeerIp = ref("");

function promptAddPeer() {
  manualPeerIp.value = "";
  showAddPeerModal.value = true;
}

function confirmAddPeer() {
  const ip = manualPeerIp.value.trim();
  if (ip) {
    invoke('add_peer_manual', { ip }).then(() => {
      showToast("已向目标 IP 发送探测信号", "success");
    }).catch(e => {
      showToast("添加失败: " + e, "error");
    });
  }
  showAddPeerModal.value = false;
}

const collapsedGroups = ref<Record<string, boolean>>({
  lobby: false,
  pinned: false,
  online: false,
  offline: false
});

function toggleGroup(groupId: string) {
  collapsedGroups.value[groupId] = !collapsedGroups.value[groupId];
}

const peerGroups = computed(() => {
  const query = searchInput.value.toLowerCase().trim();
  const allKnownPeers: Peer[] = [];
  const onlinePeerIds = new Set(peers.value.map(p => p.id));
  allKnownPeers.push(...peers.value);

  for (const [id, profile] of Object.entries(peerProfiles.value)) {
    if (id !== selfInfo.value.id && !onlinePeerIds.has(id)) {
      allKnownPeers.push({
        id,
        username: profile.remark || profile.username,
        ip: "离线", 
        os: profile.os,
        isOnline: false,
        lastSeen: "", 
        avatarId: profile.avatarId,
        avatarBase64: profile.avatarBase64,
        appState: 'offline'
      });
    }
  }

  let list = allKnownPeers;
  
  list.sort((a, b) => a.username.localeCompare(b.username));

  if (query) {
    list = list.filter(p =>
      p.username.toLowerCase().includes(query) ||
      (p.remark && p.remark.toLowerCase().includes(query)) ||
      p.ip.includes(query)
    );
  }

  const lobbyPeer: Peer = {
    id: "lobby",
    username: "局域网大厅",
    ip: "All Users",
    lastSeen: new Date().toISOString(),
    isOnline: true,
    avatarId: 0
  };

  const pinnedPeers = list.filter(p => peerProfiles.value[p.id]?.isPinned);
  const onlinePeers = list.filter(p => !peerProfiles.value[p.id]?.isPinned && p.isOnline);
  const offlinePeers = list.filter(p => !peerProfiles.value[p.id]?.isPinned && !p.isOnline);

  const groups = [];
  
  if (!query || "局域网大厅".includes(query)) {
    groups.push({ id: 'lobby', title: '大厅', peers: [lobbyPeer] });
  }
  
  if (pinnedPeers.length > 0) groups.push({ id: 'pinned', title: '置顶联系人', peers: pinnedPeers });
  if (onlinePeers.length > 0) groups.push({ id: 'online', title: '在线联系人', peers: onlinePeers });
  if (offlinePeers.length > 0) groups.push({ id: 'offline', title: '离线联系人', peers: offlinePeers });

  return groups;
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
  const parts = trimmed.split(/[\s_-]+/).filter(p => p.length > 0);
  if (parts.length > 1) {
    return (parts[0][0] + parts[1][0]).toUpperCase();
  } else if (parts.length === 1) {
    return parts[0].slice(0, 2).toUpperCase();
  }
  return trimmed.slice(0, 2).toUpperCase();
}

const formatOS = (os?: string) => {
  if (!os) return '';
  const lower = os.toLowerCase();
  if (lower.includes('win')) return 'win';
  if (lower.includes('mac')) return 'mac';
  if (lower.includes('linux')) return 'linux';
  return os;
};

defineExpose({
  switchPeerTab(direction: 1 | -1) {
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
</script>

<style scoped>
.group-header {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
  user-select: none;
  background: rgba(0, 0, 0, 0.15);
  border-bottom: 1px solid var(--border-color);
  transition: background 0.2s;
}

.group-header:hover {
  background: rgba(255, 255, 255, 0.05);
}

.group-chevron {
  margin-right: 6px;
  transition: transform 0.2s;
}

.group-chevron.expanded {
  transform: rotate(90deg);
}

.group-count {
  font-size: 11px;
  background: rgba(255, 255, 255, 0.1);
  padding: 2px 6px;
  border-radius: 10px;
  margin-left: 8px;
}

@keyframes spin {
  100% { transform: rotate(360deg); }
}
.spin-anim {
  animation: spin 1s linear infinite;
}
</style>
