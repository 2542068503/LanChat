import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { isEnabled, enable, disable } from '@tauri-apps/plugin-autostart';
import { selfInfo, showToast } from '../store';
import { useNetwork } from './useNetwork';

const editUsername = ref("");
const editAvatarId = ref(1);
const editAvatarBase64 = ref<string | null>(null);
const chatFontSize = ref(14);
const globalFontSize = ref(14);
const isDarkTheme = ref(false);
const defaultRenderLatex = ref(false);
const enableCtrlWClose = ref(true);
const appAccentColor = ref("#10b981");
const autostartEnabled = ref(false);
const silentStartup = ref(false);

export function useSettings() {
  const { fetchSelfInfo } = useNetwork();

  const initSettings = () => {
    editUsername.value = selfInfo.value.username;
    editAvatarId.value = selfInfo.value.avatarId;
    editAvatarBase64.value = selfInfo.value.avatarBase64 || null;
    
    // Load local settings
    const savedFontSize = localStorage.getItem("chatFontSize");
    if (savedFontSize) {
      chatFontSize.value = parseInt(savedFontSize);
      document.documentElement.style.setProperty('--chat-font-size', `${chatFontSize.value}px`);
    }

    const savedGlobalFontSize = localStorage.getItem("globalFontSize");
    if (savedGlobalFontSize) {
      globalFontSize.value = parseInt(savedGlobalFontSize);
      document.documentElement.style.setProperty('--global-font-size', `${globalFontSize.value}px`);
      document.documentElement.style.setProperty('--global-zoom', (globalFontSize.value / 14).toString());
    }

    const savedLatex = localStorage.getItem("defaultRenderLatex");
    if (savedLatex) {
      defaultRenderLatex.value = savedLatex === "true";
    }

    const savedCtrlW = localStorage.getItem("enableCtrlWClose");
    if (savedCtrlW) {
      enableCtrlWClose.value = savedCtrlW === "true";
    }
    
    const savedSilent = localStorage.getItem("silentStartup");
    if (savedSilent) {
      silentStartup.value = savedSilent === "true";
    }

    const savedTheme = localStorage.getItem("appTheme");
    if (savedTheme) {
      isDarkTheme.value = savedTheme === "dark";
      if (isDarkTheme.value) document.body.classList.add('dark-theme');
    }

    const savedAccent = localStorage.getItem("appAccentColor");
    if (savedAccent) {
      setAccentColor(savedAccent);
    } else {
      setAccentColor(appAccentColor.value);
    }

    // Load autostart status from OS
    loadAutostartStatus();
  };

  const loadAutostartStatus = async () => {
    try {
      autostartEnabled.value = await isEnabled();
    } catch (e) {
      console.warn("Failed to check autostart status", e);
      autostartEnabled.value = false;
    }
  };

  const toggleAutostart = async () => {
    try {
      if (autostartEnabled.value) {
        await disable();
        autostartEnabled.value = false;
        showToast("已关闭开机自启", "success");
      } else {
        await enable();
        autostartEnabled.value = true;
        showToast("已开启开机自启", "success");
      }
    } catch (e: any) {
      console.error("Failed to toggle autostart", e);
      showToast("设置失败: " + (e?.message || String(e)), "error");
    }
  };

  const setAccentColor = (color: string) => {
    appAccentColor.value = color;
    localStorage.setItem("appAccentColor", color);
    document.documentElement.style.setProperty('--accent-color', color);
    
    // Calculate contrast text color
    const hex = color.replace('#', '');
    const r = parseInt(hex.substr(0, 2), 16);
    const g = parseInt(hex.substr(2, 2), 16);
    const b = parseInt(hex.substr(4, 2), 16);
    const yiq = ((r * 299) + (g * 587) + (b * 114)) / 1000;
    const textColor = (yiq >= 145) ? '#1c1c1e' : '#ffffff';
    document.documentElement.style.setProperty('--accent-text-color', textColor);
  };

  const updateProfile = async () => {
    if (!editUsername.value.trim()) {
      showToast("名字不能为空", "error");
      return false;
    }
    
    try {
      await invoke("update_profile", {
        username: editUsername.value,
        avatarId: editAvatarId.value,
        avatarBase64: editAvatarId.value === 0 ? editAvatarBase64.value : null
      });
      await fetchSelfInfo();
      return true;
    } catch (e: any) {
      console.error("Failed to update profile", e);
      throw e;
    }
  };

  const selectAndUploadAvatar = async () => {
    try {
      const filePath = await invoke<string | null>("select_share_file");
      if (!filePath) return;
      
      const rawBase64 = await invoke<string>("read_file_base64", { filePath });
      const resizedBase64 = await resizeImageBase64("data:image/jpeg;base64," + rawBase64);
      
      editAvatarBase64.value = resizedBase64;
      editAvatarId.value = 0; 
    } catch (e: any) {
      console.error("Failed to upload avatar", e);
      throw e;
    }
  };

  const resizeImageBase64 = (dataUrl: string): Promise<string> => {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.src = dataUrl;
      img.onload = () => {
        const canvas = document.createElement("canvas");
        const ctx = canvas.getContext("2d");
        if (!ctx) {
          resolve(dataUrl);
          return;
        }
        canvas.width = 64;
        canvas.height = 64;
        
        const minDim = Math.min(img.width, img.height);
        const sx = (img.width - minDim) / 2;
        const sy = (img.height - minDim) / 2;
        
        ctx.drawImage(img, sx, sy, minDim, minDim, 0, 0, 64, 64);
        resolve(canvas.toDataURL("image/jpeg", 0.7));
      };
      img.onerror = (e) => reject(e);
    });
  };

  const saveFontSize = () => {
    localStorage.setItem("chatFontSize", chatFontSize.value.toString());
    document.documentElement.style.setProperty('--chat-font-size', `${chatFontSize.value}px`);
  };

  const saveGlobalFontSize = () => {
    localStorage.setItem("globalFontSize", globalFontSize.value.toString());
    document.documentElement.style.setProperty('--global-font-size', `${globalFontSize.value}px`);
    document.documentElement.style.setProperty('--global-zoom', (globalFontSize.value / 14).toString());
  };

  const saveDefaultRenderLatex = () => {
    localStorage.setItem("defaultRenderLatex", defaultRenderLatex.value.toString());
  };

  const saveCtrlWClose = () => {
    localStorage.setItem("enableCtrlWClose", enableCtrlWClose.value.toString());
  };

  const toggleTheme = () => {
    isDarkTheme.value = !isDarkTheme.value;
    localStorage.setItem("appTheme", isDarkTheme.value ? "dark" : "light");
    if (isDarkTheme.value) {
      document.body.classList.add('dark-theme');
    } else {
      document.body.classList.remove('dark-theme');
    }
  };
  
  const toggleSilentStartup = () => {
    silentStartup.value = !silentStartup.value;
    localStorage.setItem("silentStartup", silentStartup.value.toString());
  };

  return {
    editUsername,
    editAvatarId,
    editAvatarBase64,
    chatFontSize,
    globalFontSize,
    isDarkTheme,
    defaultRenderLatex,
    enableCtrlWClose,
    appAccentColor,
    autostartEnabled,
    silentStartup,
    initSettings,
    loadAutostartStatus,
    toggleAutostart,
    updateProfile,
    selectAndUploadAvatar,
    saveFontSize,
    saveGlobalFontSize,
    saveDefaultRenderLatex,
    saveCtrlWClose,
    toggleTheme,
    toggleSilentStartup,
    setAccentColor
  };
}
