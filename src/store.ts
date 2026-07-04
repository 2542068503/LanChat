import { ref, computed } from 'vue';
import type { Peer, Message, Profile, ProgressState } from './types'; // We will define this next

export const peers = ref<Peer[]>([]);
export const peerProfiles = ref<Record<string, Profile>>({});
export const chats = ref<Record<string, Message[]>>({});
export const activePeerId = ref<string>("");
export const currentTab = ref<"chat" | "settings">("chat");
export const selfInfo = ref({
  id: "",
  username: "",
  avatarId: 1,
  avatarBase64: null as string | null,
  localIp: "",
  tcpPort: 0,
  hostname: "",
  interfaces: [] as {name: string, ip: string}[]
});

export const unreadCounts = ref<Record<string, number>>({});
export const scrollPositions = ref<Record<string, number>>({});

export const searchInput = ref("");
export const isOnline = ref(true);
export const hashProgress = ref<ProgressState>({
  show: false,
  filePath: "",
  fileName: "",
  bytesProcessed: 0,
  totalBytes: 0,
  speed: "",
  startTime: 0,
  isHashing: false,
  eta: ""
});

export const downloadProgress = ref<ProgressState>({
  show: false,
  filePath: "", // Will hold save path
  fileName: "",
  bytesProcessed: 0,
  totalBytes: 0,
  speed: "",
  startTime: 0,
  isHashing: false, // false means downloading
  eta: ""
});

export const fileSendConfirm = ref<{
  show: boolean;
  filePath: string;
  fileInfo: any;
  targetPeerId: string;
  imagePreviewUrl: string | null;
}>({
  show: false,
  filePath: "",
  fileInfo: null,
  targetPeerId: "",
  imagePreviewUrl: null
});

// Computed properties
export const activePeer = computed(() => {
  return peers.value.find(p => p.id === activePeerId.value) || null;
});

export const activeMessages = computed(() => {
  return chats.value[activePeerId.value] || [];
});

export const filteredPeers = computed(() => {
  const query = searchInput.value.toLowerCase().trim();
  if (!query) return peers.value;
  return peers.value.filter(p =>
    p.username.toLowerCase().includes(query) ||
    (p.remark && p.remark.toLowerCase().includes(query)) ||
    p.ip.includes(query)
  );
});

export function getSenderPeer(senderId: string): Peer {
  if (senderId === selfInfo.value.id) {
    return {
      id: selfInfo.value.id,
      username: selfInfo.value.username,
      ip: '127.0.0.1 (本机)',
      os: '本机',
      isOnline: true,
      lastSeen: new Date().toISOString(),
      avatarId: selfInfo.value.avatarId,
      avatarBase64: selfInfo.value.avatarBase64 || undefined
    };
  }
  
  const onlinePeer = peers.value.find(p => p.id === senderId);
  if (onlinePeer) {
    return onlinePeer;
  }
  
  const cachedProfile = peerProfiles.value[senderId];
  return {
    id: senderId,
    username: cachedProfile?.username || '未知',
    ip: '未知',
    os: '未知',
    isOnline: false,
    lastSeen: new Date().toISOString(),
    avatarId: cachedProfile?.avatarId || 1,
    avatarBase64: cachedProfile?.avatarBase64
  };
}

export function getSenderProfile(senderId: string, msg?: Message): Profile {
  if (senderId === selfInfo.value.id) {
    return {
      username: selfInfo.value.username,
      avatarId: selfInfo.value.avatarId,
      avatarBase64: selfInfo.value.avatarBase64 || undefined
    };
  }
  
  const cached = peerProfiles.value[senderId];
  if (cached) {
    return {
      username: cached.remark || cached.username,
      avatarId: cached.avatarId,
      avatarBase64: cached.avatarBase64
    };
  }
  
  if (msg) {
    return {
      username: (msg as any).senderUsername || "Unknown",
      avatarId: 1,
      avatarBase64: undefined
    };
  }
  
  return { username: "Unknown", avatarId: 1, avatarBase64: undefined };
}

export const globalToast = ref({ show: false, message: "", type: "info" });
export function showToast(message: string, type: "info" | "success" | "error" = "info") {
  globalToast.value = { show: true, message, type };
  setTimeout(() => { globalToast.value.show = false; }, 3000);
}

export function saveChatsToLocalStorage() {
  const chatsToSave: Record<string, Message[]> = {};
  for (const peerId in chats.value) {
    chatsToSave[peerId] = chats.value[peerId].slice(-100);
  }
  localStorage.setItem("chatHistory", JSON.stringify(chatsToSave));
}

import defaultLogoUrl from './assets/logo.png';
export const globalAppIconUrl = ref(defaultLogoUrl);
