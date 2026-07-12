<template>
  <!-- CLEAR HISTORY MODAL -->
  <Transition name="modal">
    <div v-if="showClearCurrentConfirm" class="modal-overlay" @click.self="$emit('close-clear-modal')">
      <div class="modal-card">
        <div class="modal-header">
          <h3>清空记录</h3>
          <button class="modal-close-btn" @click="$emit('close-clear-modal')">×</button>
        </div>
        <div class="modal-body">
          <p>确定要清空与当前联系人的所有聊天记录吗？此操作不可恢复。</p>
        </div>
        <div class="modal-footer">
          <button class="settings-cancel-btn" @click="$emit('close-clear-modal')">取消</button>
          <button class="settings-danger-btn" @click="$emit('clear-history-confirm')">确认清空</button>
        </div>
      </div>
    </div>
  </Transition>

  <!-- PEER DETAILS MODAL -->
  <Transition name="modal">
    <div v-if="showPeerDetailModal && detailPeer" class="modal-overlay" @click.self="$emit('close-detail-modal')">
      <div class="detail-modal-card">
        <div class="modal-header">
          <h3>详细信息</h3>
          <button class="modal-close-btn" @click="$emit('close-detail-modal')">×</button>
        </div>
        <div class="modal-body">
          <div class="modal-user-profile">
            <div class="modal-avatar-wrapper">
              <img v-if="detailPeer.avatarId === 0 && detailPeer.avatarBase64" :src="detailPeer.avatarBase64" class="modal-avatar-img" />
              <div v-else class="modal-avatar-fallback" :style="getAvatarStyle(detailPeer.avatarId || 1)">
                {{ getInitials(detailPeer.username) }}
              </div>
            </div>
            <div class="modal-user-meta">
              <span class="modal-username-title">{{ detailPeer.username }}</span>
            </div>
          </div>
          
          <div class="modal-details-grid">
            <div class="details-row">
              <span class="details-label">用户 ID (UUID)</span>
              <span class="details-val font-mono">{{ detailPeer.id }}</span>
            </div>
            <div class="details-row">
              <span class="details-label">IP 地址</span>
              <span class="details-val font-mono">{{ detailPeer.ip || '未知' }}</span>
            </div>
            <div class="details-row">
              <span class="details-label">操作系统</span>
              <span class="details-val">{{ formatOS(detailPeer.os) || '未知' }}</span>
            </div>
            <div class="details-row">
              <span class="details-label">当前状态</span>
              <span class="details-val" :class="detailPeer.isOnline ? 'online-txt' : 'offline-txt'">
                {{ detailPeer.isOnline ? (detailPeer.appState === 'active' ? '在线 (活跃)' : (detailPeer.appState === 'background' ? '在线 (后台)' : '在线')) : '离线' }}
              </span>
            </div>
            <div class="details-row">
              <span class="details-label">LanChat 版本</span>
              <span class="details-val font-mono">{{ detailPeer.version || '未知' }}</span>
            </div>
            <div v-if="!detailPeer.isOnline" class="details-row">
              <span class="details-label">最后在线时间</span>
              <span class="details-val">{{ formatTime(detailPeer.lastSeen) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Transition>

  <!-- FILE RECEIVE CONFIRM MODAL -->
  <Transition name="modal">
    <div v-if="fileConfirmModal.show" class="modal-overlay" @click.self="$emit('close-file-modal')">
      <div class="modal-card">
        <div class="modal-header">
          <h3>接收文件</h3>
          <button class="modal-close-btn" @click="$emit('close-file-modal')">×</button>
        </div>
        <div class="modal-body">
          <p>对方发送了一个文件，是否接收？</p>
          <div v-if="fileConfirmModal.msg && fileConfirmModal.msg.fileInfo">
            <strong>{{ fileConfirmModal.msg.fileInfo.name }}</strong> ({{ formatFileSize(fileConfirmModal.msg.fileInfo.size) }})
          </div>
        </div>
        <div class="modal-footer">
          <button class="settings-cancel-btn" @click="$emit('file-receive-confirm', 'reject')">拒绝</button>
          <button class="settings-save-btn confirm-send-btn" @click="$emit('file-receive-confirm', 'accept')">接收</button>
        </div>
      </div>
    </div>
  </Transition>

  <!-- FILE PROGRESS MODAL (HASHING) -->
  <Transition name="modal">
    <div v-if="hashProgress.show" class="modal-overlay">
      <div class="modal-card progress-modal-card">
        <div class="progress-header">
          <div class="spinner"></div>
          <h3>正在处理文件...</h3>
        </div>
        <div class="progress-body">
          <div class="file-name" :title="hashProgress.fileName">{{ hashProgress.fileName }}</div>
          
          <div class="progress-bar-container">
            <div class="progress-bar" :style="{ width: hashPercent + '%' }"></div>
          </div>
          
          <div class="progress-stats">
            <span class="percent">{{ hashPercent }}%</span>
            <span class="speed">{{ hashProgress.speed }}</span>
          </div>
          
          <template v-if="!showSkipWarning">
            <div class="progress-details">
              <span class="bytes">{{ formatFileSize(hashProgress.bytesProcessed) }} / {{ formatFileSize(hashProgress.totalBytes) }}</span>
              <span class="eta">剩余时间: {{ hashProgress.eta }}</span>
            </div>
            <div class="progress-tip">
              {{ hashProgress.isHashing ? '正在计算 SHA256 校验码，请稍候...' : '处理中...' }}
            </div>
            
            <div class="progress-actions">
              <button class="settings-cancel-btn" @click="cancelHash">取消发送</button>
              <button class="settings-save-btn" style="background-color: var(--accent-color); opacity: 0.9;" @click="handleSkipHash">快速发送 (跳过校验)</button>
            </div>
          </template>
          <template v-else>
            <div class="warning-tip" style="color: #ffaa00; font-size: 13px; margin-bottom: 15px; margin-top: 10px; line-height: 1.4;">
              【警告】跳过 SHA256 校验将无法在接收端验证文件的完整性。<br>如果您确认该文件在本地网络传输中是安全的，请点击确认以快速发送。
            </div>
            <div class="progress-actions">
              <button class="settings-cancel-btn" @click="cancelSkipHash">返回</button>
              <button class="settings-save-btn" style="background-color: #ff5555; color: white;" @click="confirmSkipHash">确认跳过</button>
            </div>
          </template>
        </div>
      </div>
    </div>
  </Transition>

  <!-- FILE PROGRESS MODAL (DOWNLOADING) -->
  <Transition name="modal">
    <div v-if="downloadProgress.show" class="modal-overlay">
      <div class="modal-card progress-modal-card">
        <div class="progress-header">
          <div class="spinner"></div>
          <h3>正在下载文件...</h3>
        </div>
        <div class="progress-body">
          <div class="file-name" :title="downloadProgress.fileName">{{ downloadProgress.fileName }}</div>
          
          <div class="progress-bar-container">
            <div class="progress-bar" :style="{ width: downloadPercent + '%' }"></div>
          </div>
          
          <div class="progress-stats">
            <span class="percent">{{ downloadPercent }}%</span>
            <span class="speed">{{ downloadProgress.speed }}</span>
          </div>
          
          <div class="progress-details">
            <span class="bytes">{{ formatFileSize(downloadProgress.bytesProcessed) }} / {{ formatFileSize(downloadProgress.totalBytes) }}</span>
            <span class="eta">剩余时间: {{ downloadProgress.eta }}</span>
          </div>
          <div class="progress-tip">
            保存至: {{ downloadProgress.filePath }}
          </div>
        </div>
      </div>
    </div>
  </Transition>

  <!-- FILE SEND CONFIRM MODAL -->
  <Transition name="modal">
    <div v-if="fileSendConfirm.show" class="modal-overlay" @click.self="cancelSendFile">
      <div class="modal-card file-send-card">
        <div class="modal-header">
          <h3>发送文件确认</h3>
          <button class="modal-close-btn" @click="cancelSendFile">×</button>
        </div>
        <div class="modal-body file-send-body">
          <div v-if="fileSendConfirm.imagePreviewUrl" class="file-preview-container">
            <img :src="fileSendConfirm.imagePreviewUrl" class="file-preview-img" alt="预览图" />
          </div>
          <div class="file-info-box">
            <div class="file-name" :title="fileSendConfirm.fileInfo?.name">{{ fileSendConfirm.fileInfo?.name }}</div>
            <div class="file-size">{{ formatFileSize(fileSendConfirm.fileInfo?.size || 0) }}</div>
            <div class="file-hash" title="SHA256">
              <span class="hash-label">SHA256:</span> 
              <span class="hash-value">{{ fileSendConfirm.fileInfo?.sha256 }}</span>
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button class="settings-cancel-btn" @click="cancelSendFile">取消</button>
          <button class="settings-save-btn confirm-send-btn" @click="confirmSendFile">确认发送</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { hashProgress, fileSendConfirm, downloadProgress } from '../store';
import { useFileTransfer } from '../composables/useFileTransfer';
import type { Peer } from '../types';

const { cancelHash, skipHash } = useFileTransfer();

const showSkipWarning = ref(false);

watch(() => hashProgress.value.show, (newVal) => {
  if (!newVal) {
    showSkipWarning.value = false;
  }
});

const handleSkipHash = () => {
  showSkipWarning.value = true;
};

const confirmSkipHash = () => {
  skipHash();
  showSkipWarning.value = false;
};

const cancelSkipHash = () => {
  showSkipWarning.value = false;
};

const emit = defineEmits([
  'clear-history-confirm', 
  'file-receive-confirm', 
  'send-file-confirm',
  'close-clear-modal', 
  'close-detail-modal', 
  'close-file-modal'
]);

const props = defineProps<{
  showClearCurrentConfirm: boolean;
  showPeerDetailModal: boolean;
  detailPeer: Peer | null;
  fileConfirmModal: { show: boolean, msg: any };
}>();

const cancelSendFile = () => {
  fileSendConfirm.value.show = false;
};
const confirmSendFile = () => {
  emit('send-file-confirm');
};

const hashPercent = computed(() => {
  if (hashProgress.value.totalBytes === 0) return 0;
  return Math.floor((hashProgress.value.bytesProcessed / hashProgress.value.totalBytes) * 100);
});

const downloadPercent = computed(() => {
  if (downloadProgress.value.totalBytes === 0) return 0;
  return Math.floor((downloadProgress.value.bytesProcessed / downloadProgress.value.totalBytes) * 100);
});

function formatTime(timestamp: string | number) {
  if (!timestamp) return '未知';
  const num = typeof timestamp === 'string' ? parseInt(timestamp, 10) : timestamp;
  if (isNaN(num) || num === 0) return '未知';
  return new Date(num).toLocaleString();
}

function formatFileSize(bytes: number) {
  if (bytes === 0 || !bytes) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

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

function formatOS(os: string | undefined | null) {
  if (!os) return '';
  const lower = os.toLowerCase();
  if (lower.includes('win')) return 'win';
  if (lower.includes('mac')) return 'mac';
  if (lower.includes('linux')) return 'linux';
  return os;
}
</script>
