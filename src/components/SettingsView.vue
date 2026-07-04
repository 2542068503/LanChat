<template>
  <div class="settings-view-layout">
    <div class="settings-container">
      <div class="settings-title">个人设置</div>
      
      <div class="settings-section">
        <h3>基本资料</h3>
        
        <div class="settings-field avatar-edit-area">
          <label class="settings-label">头像</label>
          <div class="self-avatar-container" style="width: 50px; height: 50px; border-radius: 50%; border: none; overflow: hidden; cursor: pointer; flex-shrink: 0;" @click="selectAndUploadAvatar">
            <img v-if="editAvatarBase64 && editAvatarId === 0" :src="editAvatarBase64" class="self-avatar-img" style="width: 100%; height: 100%; object-fit: cover;" />
            <div v-else class="self-avatar-fallback" :style="[getAvatarStyle(editAvatarId || 1), { width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'white', fontWeight: 'bold', fontSize: '20px' }]">
              {{ getInitials(editUsername) }}
            </div>
          </div>
          <button class="settings-upload-avatar-btn" @click="selectAndUploadAvatar">更换头像</button>
        </div>
        
        <div class="settings-divider"></div>
        
        <div class="settings-field">
          <label class="settings-label">昵称</label>
          <input type="text" v-model.trim="editUsername" class="settings-input" maxlength="16" placeholder="名字不能留空" />
        </div>
        
        <div class="settings-divider"></div>
        
        <button class="settings-save-btn" @click="handleUpdateProfile" :disabled="isUpdating || !isProfileModified || !editUsername.trim()">
          {{ isUpdating ? '保存中...' : (saveSuccess ? '已保存!' : '保存资料') }}
        </button>
      </div>
      
      <div class="settings-section">
        <h3>外观设置</h3>
        
        <div class="settings-field">
          <label class="settings-label">主题颜色</label>
          <div class="color-picker">
            <button class="color-btn" style="--btn-color: #10b981;" :class="{ active: appAccentColor === '#10b981' }" @click="setAccentColor('#10b981')"></button>
            <button class="color-btn" style="--btn-color: #3b82f6;" :class="{ active: appAccentColor === '#3b82f6' }" @click="setAccentColor('#3b82f6')"></button>
            <button class="color-btn" style="--btn-color: #8b5cf6;" :class="{ active: appAccentColor === '#8b5cf6' }" @click="setAccentColor('#8b5cf6')"></button>
            <button class="color-btn" style="--btn-color: #f43f5e;" :class="{ active: appAccentColor === '#f43f5e' }" @click="setAccentColor('#f43f5e')"></button>
            <button class="color-btn" style="--btn-color: #f59e0b;" :class="{ active: appAccentColor === '#f59e0b' }" @click="setAccentColor('#f59e0b')"></button>
          </div>
        </div>

        <div class="settings-divider"></div>

        <div class="settings-field">
          <label class="settings-label">深色模式</label>
          <button class="theme-toggle-btn" @click="toggleTheme">
            {{ isDarkTheme ? '切换为浅色模式' : '切换为深色模式' }}
          </button>
        </div>

        <div class="settings-divider"></div>
        
        <div class="settings-field-vertical">
          <label class="settings-label">全局缩放 ({{ (globalFontSize / 14 * 100).toFixed(0) }}%)</label>
          <input type="range" min="10" max="24" step="1" v-model="globalFontSize" @change="saveGlobalFontSize" class="settings-slider" />
        </div>
        
        <div class="settings-divider"></div>
        
        <div class="settings-field-vertical">
          <label class="settings-label">聊天字体大小 ({{ chatFontSize }}px)</label>
          <input type="range" min="12" max="24" step="1" v-model="chatFontSize" @change="saveFontSize" class="settings-slider" />
          <div class="chat-font-preview">
            <div class="preview-bubble" :style="{ fontSize: chatFontSize + 'px' }">
              预览文字 Aaa 123
            </div>
          </div>
        </div>

        <div class="settings-divider"></div>
        
        <div class="settings-field">
          <label class="settings-label">LaTeX 渲染选项 ({{ chatFontSize }}px)</label>
          <label class="latex-toggle">
            <input type="checkbox" v-model="defaultRenderLatex" @change="saveDefaultRenderLatex">
            <span class="latex-toggle-text">默认渲染 LaTeX 公式</span>
          </label>
        </div>

        <div class="settings-divider"></div>
        
        <div class="settings-field">
          <label class="settings-label">快捷键设置 ({{ chatFontSize }}px)</label>
          <label class="latex-toggle">
            <input type="checkbox" v-model="enableCtrlWClose" @change="saveCtrlWClose">
            <span class="latex-toggle-text">启用 Ctrl+W 快捷关闭聊天窗口</span>
          </label>
        </div>

        <div class="settings-divider"></div>
        
        <div class="settings-field">
          <label class="settings-label">预设伪装</label>
          <div style="display: flex; gap: 8px; flex-wrap: wrap;">
            <div
              v-for="preset in presetIcons" 
              :key="preset"
              style="width: 32px; height: 32px; border-radius: 6px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.1); cursor: pointer;"
              @click="setPresetIcon(preset)"
            >
              <img 
                :src="preset" 
                style="width: 100%; height: 100%; object-fit: contain; background: transparent; padding: 0;" 
              />
            </div>
          </div>
        </div>

        <div class="settings-field" style="margin-top: 12px; display: flex; gap: 8px;">
          <input type="file" accept="image/png, image/jpeg, image/x-icon, image/webp, image/svg+xml" @change="changeAppIcon" style="display:none" ref="appIconInput" />
          <button class="settings-upload-avatar-btn" @click="appIconInput?.click()">自定义图片...</button>
          <button class="settings-upload-avatar-btn" style="background-color: var(--border-color); color: var(--text-color);" @click="restoreDefaultIcon">恢复默认</button>
        </div>
      </div>
      
      <div class="settings-section network-section">
        <h3>网络信息</h3>
        <div class="network-overview">
          <div>
            <div class="network-label">主机名</div>
            <div class="network-value highlight">{{ selfInfo.hostname || 'Unknown' }}</div>
          </div>
          <div>
            <div class="network-value highlight">{{ selfInfo.tcpPort || '0' }}</div>
          </div>
        </div>
        
        <div class="network-label" style="margin-top: 12px;">可用网络接口</div>
        <div class="interfaces-grid">
          <div v-for="iface in selfInfo.interfaces" :key="iface.name" 
               class="interface-card" 
               :class="{ active: iface.ip === selfInfo.localIp }">
            <span class="iface-name">{{ iface.name }}</span>
            <span class="network-value" :class="{ highlight: iface.ip === selfInfo.localIp }">{{ iface.ip }}</span>
          </div>
        </div>
      </div>
      
      <div class="settings-section">
        <h3>关于 LanChat</h3>
        
        <div style="display: flex; gap: 16px; margin-bottom: 16px;">
          <img :src="globalAppIconUrl" style="width: 64px; height: 64px; border-radius: 8px; object-fit: cover; box-shadow: 0 4px 12px rgba(0,0,0,0.1);" />
          <div style="display: flex; flex-direction: column; justify-content: center;">
            <h2 style="margin: 0; font-size: 20px; color: var(--text-primary);">LanChat</h2>
            <div style="color: var(--text-secondary); font-size: 13px; opacity: 0.8;">v1.0.0 (Local Build)</div>
            <div style="color: var(--accent-color); font-size: 13px; margin-top: 4px; font-weight: 500;">开发者: zhangshiyan</div>
          </div>
        </div>
        
        <div style="font-size: 13px; color: var(--text-primary); line-height: 1.6; opacity: 0.9;">
          <p>基于极简、纯化设计理念的局域网零配置聊天与传输工具。无需公网服务器，自动发现网内节点，保障隐私安全与极速体验。</p>
          
          <div class="about-features" style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-top: 8px; background: rgba(0,0,0,0.06); padding: 12px; border-radius: 8px; border: 1px solid var(--border-color);">
            <div style="display: flex; align-items: center;"><Building2 :size="14" style="margin-right: 6px; color: var(--text-secondary);" /> 局域网大厅组播群聊</div>
            <div style="display: flex; align-items: center;"><Rocket :size="14" style="margin-right: 6px; color: var(--text-secondary);" /> 极速无服务器文件传输</div>
            <div style="display: flex; align-items: center;"><FileText :size="14" style="margin-right: 6px; color: var(--text-secondary);" /> LaTeX & Markdown 渲染</div>
            <div style="display: flex; align-items: center;"><Palette :size="14" style="margin-right: 6px; color: var(--text-secondary);" /> 深度个性化主题配色</div>
          </div>
          
          <div style="margin-top: 12px; font-size: 11px; display: flex; justify-content: space-between; color: var(--text-secondary); opacity: 0.8; margin-bottom: 20px;">
            <span>运行环境: Tauri v2 + Vue 3 + TypeScript</span>
            <span>© 2026 LanChat Project</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { Building2, Rocket, FileText, Palette } from 'lucide-vue-next';
import { useSettings } from '../composables/useSettings';
import { selfInfo, showToast, globalAppIconUrl } from '../store';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Image } from '@tauri-apps/api/image';

const presetFiles = import.meta.glob('../assets/presets/*.{png,jpg,jpeg,svg,ico,webp,gif}', { query: '?url', eager: true });
const presetIcons = Object.values(presetFiles).map((mod: any) => mod.default);

import { TrayIcon } from '@tauri-apps/api/tray';
import defaultLogoUrl from '../assets/logo.png';

const { 
  editUsername, editAvatarId, editAvatarBase64, 
  chatFontSize, globalFontSize, isDarkTheme, defaultRenderLatex,
  enableCtrlWClose,
  appAccentColor,
  updateProfile,
  selectAndUploadAvatar,
  saveFontSize,
  saveGlobalFontSize,
  saveDefaultRenderLatex,
  saveCtrlWClose,
  toggleTheme,
  setAccentColor
} = useSettings();

const appIconInput = ref<HTMLInputElement | null>(null);
const isUpdating = ref(false);
const saveSuccess = ref(false);

const isProfileModified = computed(() => {
  if (editUsername.value !== selfInfo.value.username) return true;
  if (editAvatarId.value !== selfInfo.value.avatarId) return true;
  if (editAvatarId.value === 0 && editAvatarBase64.value !== selfInfo.value.avatarBase64) return true;
  return false;
});

async function handleUpdateProfile() {
  if (isUpdating.value || !isProfileModified.value) return;
  isUpdating.value = true;
  try {
    await updateProfile();
    saveSuccess.value = true;
    setTimeout(() => { saveSuccess.value = false; }, 2000);
    if (appIconInput.value) {
      appIconInput.value.value = '';
    }
  } finally {
    isUpdating.value = false;
  }
}

async function restoreDefaultIcon() {
  try {
    const response = await fetch(defaultLogoUrl);
    const buffer = await response.arrayBuffer();
    const uint8Array = new Uint8Array(buffer);
    const image = await Image.fromBytes(uint8Array);
    
    const appWindow = getCurrentWindow();
    await appWindow.setIcon(image);
    
    try {
      const tray = await TrayIcon.getById('main-tray');
      if (tray) {
        await tray.setIcon(image);
      }
    } catch (err) {
      console.warn("Tray icon update failed", err);
    }
    
    globalAppIconUrl.value = defaultLogoUrl;
    showToast("应用图标已恢复", "success");
  } catch (e: any) {
    showToast("恢复图标失败: " + (e?.message || String(e)), "error");
    console.error(e);
  }
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

async function getRasterizedImageBytes(url: string, isSvg: boolean): Promise<Uint8Array> {
  if (isSvg) {
    return new Promise((resolve, reject) => {
      const img = new window.Image();
      img.onload = () => {
        const canvas = document.createElement('canvas');
        canvas.width = 256;
        canvas.height = 256;
        const ctx = canvas.getContext('2d');
        if (!ctx) {
          reject(new Error('Failed to get canvas context'));
          return;
        }
        ctx.drawImage(img, 0, 0, 256, 256);
        canvas.toBlob(async (blob) => {
          if (!blob) {
            reject(new Error('Failed to create blob'));
            return;
          }
          const buffer = await blob.arrayBuffer();
          resolve(new Uint8Array(buffer));
        }, 'image/png');
      };
      img.onerror = () => reject(new Error('Failed to load SVG for rasterization'));
      img.src = url;
    });
  } else {
    const response = await fetch(url);
    const buffer = await response.arrayBuffer();
    return new Uint8Array(buffer);
  }
}

async function setPresetIcon(url: string) {
  try {
    const isSvg = url.toLowerCase().includes('.svg');
    const uint8Array = await getRasterizedImageBytes(url, isSvg);
    const image = await Image.fromBytes(uint8Array);
    const appWindow = getCurrentWindow();
    await appWindow.setIcon(image);
    
    try {
      const tray = await TrayIcon.getById('main-tray');
      if (tray) {
        await tray.setIcon(image);
      }
    } catch (err) {
      console.warn("Tray icon update failed", err);
    }
    
    globalAppIconUrl.value = url;
    showToast("应用图标已伪装", "success");
  } catch (e: any) {
    showToast("图标修改失败: " + (e?.message || String(e)), "error");
    console.error(e);
  }
}

async function changeAppIcon(event: Event) {
  const target = event.target as HTMLInputElement;
  if (!target.files || target.files.length === 0) return;
  const file = target.files[0];
  
  try {
    const fileUrl = URL.createObjectURL(file);
    const isSvg = file.type === 'image/svg+xml' || file.name.toLowerCase().endsWith('.svg');
    const uint8Array = await getRasterizedImageBytes(fileUrl, isSvg);
    const image = await Image.fromBytes(uint8Array);
    const appWindow = getCurrentWindow();
    await appWindow.setIcon(image);
    
    try {
      const tray = await TrayIcon.getById('main-tray');
      if (tray) {
        await tray.setIcon(image);
      }
    } catch (err) {
      console.warn("Tray icon update failed", err);
    }
    
    // Create a temporary object URL for the UI display
    const blob = new Blob([uint8Array as any], { type: file.type });
    globalAppIconUrl.value = URL.createObjectURL(blob);
    
    showToast("应用图标已伪装", "success");
  } catch (e: any) {
    showToast("图标修改失败: " + (e?.message || String(e)), "error");
    console.error(e);
  } finally {
    // Reset input so the same file can be selected again
    target.value = '';
  }
}
</script>

<style scoped>
.chat-font-preview {
  display: flex;
  justify-content: flex-start;
}
.settings-section {
  color: var(--text-primary);
  background-color: var(--peers-bg);
  padding: 16px 20px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  margin-bottom: 16px;
}
.settings-section h3 {
  color: var(--text-primary);
  margin-top: 0;
  margin-bottom: 16px;
  font-size: 15px;
}
.settings-field {
  display: flex;
  align-items: center;
  margin-bottom: 0;
  gap: 12px;
  min-height: 28px;
}
.settings-field-vertical {
  display: flex;
  flex-direction: column;
  margin-bottom: 0;
  gap: 8px;
}
.settings-field-row {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 28px;
}
.settings-field-row > :last-child:not(.settings-label) {
  margin-left: auto;
}
.avatar-edit-area {
  justify-content: flex-start;
}
.settings-field > :last-child:not(.settings-label) {
  margin-left: auto;
}
.settings-label {
  color: var(--text-primary);
  flex-shrink: 0;
  display: flex;
  align-items: center;
}
.preview-bubble {
  background: var(--accent-color, #10b981);
  color: var(--accent-text-color, #ffffff);
  padding: 8px 14px;
  border-radius: 8px;
  border-top-left-radius: 4px;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
  display: inline-block;
  max-width: 100%;
  word-wrap: break-word;
}

/* Color Picker Styling */
.color-picker {
  display: flex;
  gap: 12px;
  margin-top: 4px;
}
.color-btn {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 2px solid transparent;
  background-color: var(--btn-color);
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
}
.color-btn:hover {
  transform: scale(1.1);
}
.color-btn.active {
  border-color: white;
  box-shadow: 0 0 0 2px var(--btn-color);
  transform: scale(1.1);
}

/* Custom Slider Styling */
.settings-slider {
  -webkit-appearance: none;
  width: 100%;
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  outline: none;
  border: none;
  margin: 10px 0;
}
.light .settings-slider {
  background: rgba(0, 0, 0, 0.1);
}
.settings-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--accent-color, #10b981);
  cursor: pointer;
  box-shadow: 0 2px 5px rgba(0,0,0,0.2);
  transition: transform 0.15s ease;
  border: 2px solid white;
  margin-top: -6px;
}
.settings-slider::-webkit-slider-thumb:hover {
  transform: scale(1.15);
}

/* Custom Latex Toggle Styling */
.latex-toggle {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: var(--window-bg, rgba(255, 255, 255, 0.05));
  border: 1px solid var(--border-color);
  height: 32px;
  padding: 0 12px;
  border-radius: 8px;
  font-size: 12.5px;
  color: var(--text-primary);
  transition: all 0.2s;
  cursor: pointer;
}
.latex-toggle:hover {
  background: var(--card-bg, rgba(255, 255, 255, 0.08));
}
.latex-toggle input[type="checkbox"] {
  accent-color: var(--accent-color);
  width: 14px;
  height: 14px;
  cursor: pointer;
}
.latex-toggle-text {
  font-size: 13.5px;
  color: var(--text-primary);
}

/* Theme Toggle Button */
.theme-toggle-btn {
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  height: 32px;
  padding: 0 12px;
  border-radius: 8px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: fit-content;
}

/* Explicit Save Button Styling */
.settings-save-btn:not(:disabled) {
  background-color: var(--accent-color, #10b981) !important;
  color: var(--accent-text-color, #ffffff) !important;
  border: none !important;
}

.settings-save-btn {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  padding: 0 16px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.settings-divider {
  height: 1px;
  background-color: var(--settings-divider-color);
  margin: 4px 0;
  width: 100%;
}

.settings-upload-avatar-btn {
  height: 32px !important;
  padding: 0 16px !important;
  font-size: 13px !important;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  background: var(--accent-color) !important;
  color: #fff !important;
  border: none !important;
  border-radius: 8px !important;
  font-weight: 500;
  transition: all 0.2s ease;
  cursor: pointer;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}
.settings-upload-avatar-btn:hover {
  filter: brightness(1.1);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  transform: translateY(-1px);
}

.settings-input {
  height: 32px;
  padding: 0 12px;
  font-size: 13px;
  box-sizing: border-box;
  background: var(--chat-bg);
  color: var(--text-primary);
  border: 1px solid var(--border-color) !important;
  border-radius: 8px;
  outline: none;
  transition: all 0.2s;
  text-align: left;
  width: 280px;
}
.settings-input:focus {
  border-color: var(--accent-color) !important;
  box-shadow: 0 0 0 2px var(--accent-glow);
}

/* Theme Toggle Button */
.theme-toggle-btn {
  background: var(--accent-color);
  border: none;
  color: #ffffff;
  height: 32px;
  padding: 0 16px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.theme-toggle-btn:hover {
  filter: brightness(1.1);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  transform: translateY(-1px);
}
</style>
