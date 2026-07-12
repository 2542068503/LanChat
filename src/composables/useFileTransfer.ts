import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { 
  activePeerId, chats, hashProgress, downloadProgress, 
  fileSendConfirm, selfInfo, showToast, saveChatsToLocalStorage
} from '../store';
import { useChat } from './useChat';

export function useFileTransfer() {
  const { appendMessage } = useChat();

  const cancelHash = () => {
    invoke("emit_cancel_hash").catch(console.error); // We'll implement this in rust, or we can just emit event from frontend
    // Better, we can just emit the event directly from frontend using tauri emit
    import('@tauri-apps/api/event').then(({ emit }) => emit("cancel-hash"));
    hashProgress.value.show = false;
  };

  const skipHash = () => {
    import('@tauri-apps/api/event').then(({ emit }) => emit("skip-hash"));
  };
  
  const setupFileListeners = async () => {
    await listen<any>("hash-progress", (event) => {
      if (hashProgress.value.show) {
        hashProgress.value.bytesProcessed = event.payload.bytesProcessed;
        hashProgress.value.totalBytes = event.payload.totalBytes;
        
        const now = Date.now();
        const elapsedSeconds = (now - hashProgress.value.startTime) / 1000;
        if (elapsedSeconds > 0) {
          const bytesPerSec = event.payload.bytesProcessed / elapsedSeconds;
          hashProgress.value.speed = formatSpeed(bytesPerSec);
          
          if (bytesPerSec > 0) {
            const remainingBytes = event.payload.totalBytes - event.payload.bytesProcessed;
            const remainingSecs = Math.ceil(remainingBytes / bytesPerSec);
            hashProgress.value.eta = formatTime(remainingSecs);
          }
        }
      }
    });

    await listen<any>("download-progress", (event) => {
      const { fileId, fileName, bytesDownloaded, totalBytes, status } = event.payload;
      
      // Update message object directly
      for (const peerId in chats.value) {
        const msgs = chats.value[peerId];
        const idx = msgs.findIndex((m: any) => m.fileInfo?.fileId === fileId);
        if (idx !== -1) {
          const msg = msgs[idx];
          const newMsg = { ...msg };
          newMsg.downloadBytesProcessed = bytesDownloaded;
          newMsg.downloadTotalBytes = totalBytes;
          
          if (!newMsg.downloadStartTime) newMsg.downloadStartTime = Date.now();
          const elapsedSeconds = (Date.now() - newMsg.downloadStartTime) / 1000;
          if (elapsedSeconds > 0) {
            const bytesPerSec = bytesDownloaded / elapsedSeconds;
            newMsg.downloadSpeed = formatSpeed(bytesPerSec);
            if (bytesPerSec > 0) {
              const remainingSecs = Math.ceil((totalBytes - bytesDownloaded) / bytesPerSec);
              newMsg.downloadEta = formatTime(remainingSecs);
            }
          }
          msgs[idx] = newMsg;
          break;
        }
      }
      
      if (!downloadProgress.value.show) {
        downloadProgress.value = {
          show: false, // Don't show global progress anymore
          filePath: fileId, // Use fileId as key
          fileName: fileName,
          bytesProcessed: bytesDownloaded,
          totalBytes: totalBytes,
          speed: "计算中...",
          startTime: Date.now(),
          isHashing: false,
          eta: "计算中..."
        };
      } else {
        downloadProgress.value.bytesProcessed = bytesDownloaded;
        downloadProgress.value.totalBytes = totalBytes;
        
        const now = Date.now();
        const elapsedSeconds = (now - downloadProgress.value.startTime) / 1000;
        if (elapsedSeconds > 0) {
          const bytesPerSec = bytesDownloaded / elapsedSeconds;
          downloadProgress.value.speed = formatSpeed(bytesPerSec);
          
          if (bytesPerSec > 0) {
            const remainingBytes = totalBytes - bytesDownloaded;
            const remainingSecs = Math.ceil(remainingBytes / bytesPerSec);
            downloadProgress.value.eta = formatTime(remainingSecs);
          }
        }
      }
      
      if (bytesDownloaded >= totalBytes || status === "completed") {
        setTimeout(() => {
          downloadProgress.value.show = false;
        }, 1000);
      }
    });

    await listen<any>("download-success", (event) => {
      const { fileId, savePath } = event.payload;
      downloadProgress.value.show = false;
      
      let fileName = "文件";
      
      // Find the message in chats and update its localPath
      for (const peerId in chats.value) {
        const msgs = chats.value[peerId];
        const idx = msgs.findIndex((m: any) => m.fileInfo?.fileId === fileId);
        if (idx !== -1) {
          const currentMsg = msgs[idx];
          msgs[idx] = {
            ...currentMsg,
            isDownloading: false,
            localPath: savePath
          };
          fileName = currentMsg.fileInfo?.name || "文件";
        }
      }
      
      saveChatsToLocalStorage();
      
      if (!savePath.includes("LanChat_Thumbnails")) {
        showToast(`${fileName} 下载成功！\n保存至: ${savePath}`, "success");
      }
    });

    await listen<any>("download-error", (event) => {
      const { fileId, error } = event.payload;
      downloadProgress.value.show = false;
      showToast(`下载失败: ${error}`, "error");
      for (const peerId in chats.value) {
        const msgs = chats.value[peerId];
        const idx = msgs.findIndex((m: any) => m.fileInfo?.fileId === fileId);
        if (idx !== -1) {
          msgs[idx] = {
            ...msgs[idx],
            isDownloading: false
          };
          break;
        }
      }
    });

    await listen<any>("message-received", (event) => {
      setTimeout(() => {
        autoDownloadImage(event.payload, event.payload.senderId);
      }, 50);
    });

    await listen<any>("group-message-received", (event) => {
      setTimeout(() => {
        autoDownloadImage(event.payload, "lobby");
      }, 50);
    });
  };

  const selectAndShareFile = async (providedFilePath?: string) => {
    if (!activePeerId.value) return;
    const targetPeerId = activePeerId.value;

    try {
      const filePath = providedFilePath || await invoke<string | null>("select_share_file");
      if (!filePath) return;

      const fileName = filePath.split(/[/\\]/).pop() || "未知文件";

      hashProgress.value = {
        show: true,
        filePath,
        fileName,
        bytesProcessed: 0,
        totalBytes: 0,
        speed: "计算中...",
        startTime: Date.now(),
        isHashing: true,
        eta: "计算中..."
      };

      let sha256 = "";
      try {
        sha256 = await invoke("calculate_file_hash", { filePath });
      } catch (e: any) {
        if (e === "Cancelled") {
          console.log("Hashing cancelled");
          hashProgress.value.show = false;
          return;
        }
        throw e;
      }
      
      if (sha256 === "SKIPPED") {
        sha256 = ""; // Empty hash when skipped
      }

      const fileInfo: any = await invoke("share_file_with_hash", { filePath, sha256 });

      hashProgress.value.show = false;
      
      const isImage = fileName.match(/\.(jpg|jpeg|png|gif|webp)$/i);
      let previewUrl = null;
      if (isImage) {
        previewUrl = convertFileSrc(filePath);
      }

      fileSendConfirm.value = {
        show: true,
        filePath,
        fileInfo,
        targetPeerId,
        imagePreviewUrl: previewUrl
      };
      
    } catch (e: any) {
      hashProgress.value.show = false;
      console.error("Failed to share file:", e);
      throw e; 
    }
  };

  const sendClipboardImage = async (base64Data: string) => {
    if (!activePeerId.value) return;
    const targetPeerId = activePeerId.value;

    try {
      const filePath = await invoke<string>("save_clipboard_image", { base64Data });
      if (!filePath) return;

      const fileName = filePath.split(/[/\\]/).pop() || "clipboard.png";

      hashProgress.value = {
        show: true,
        filePath,
        fileName,
        bytesProcessed: 0,
        totalBytes: 0,
        speed: "计算中...",
        startTime: Date.now(),
        isHashing: true,
        eta: "计算中..."
      };

      let sha256 = "";
      try {
        sha256 = await invoke("calculate_file_hash", { filePath });
      } catch (e: any) {
        if (e === "Cancelled") {
          hashProgress.value.show = false;
          return;
        }
        throw e;
      }
      
      if (sha256 === "SKIPPED") {
        sha256 = "";
      }

      const fileInfo: any = await invoke("share_file_with_hash", { filePath, sha256 });
      hashProgress.value.show = false;
      
      const previewUrl = convertFileSrc(filePath);

      fileSendConfirm.value = {
        show: true,
        filePath,
        fileInfo,
        targetPeerId,
        imagePreviewUrl: previewUrl
      };
      
    } catch (e: any) {
      hashProgress.value.show = false;
      console.error("Failed to share clipboard image:", e);
      throw e;
    }
  };

  
  const sendConfirmedFile = async () => {
    const { targetPeerId, filePath, fileInfo } = fileSendConfirm.value;
    if (!targetPeerId || !filePath || !fileInfo) return;
    
    fileSendConfirm.value.show = false;

    try {
      if (targetPeerId === "lobby") {
        const msg: any = await invoke("send_group_message", {
          contentType: "file",
          content: filePath,
          fileInfo,
          renderLatex: false,
          quoteMsgId: null,
          quoteSender: null,
          quoteContent: null
        });
        msg.localPath = filePath;
        appendMessage("lobby", msg);
      } else {
        const msg: any = await invoke("send_message", {
          peerId: targetPeerId,
          contentType: "file",
          content: filePath,
          fileInfo,
          renderLatex: false,
          quoteMsgId: null,
          quoteSender: null,
          quoteContent: null
        });
        msg.localPath = filePath;
        appendMessage(targetPeerId, msg);
      }
    } catch (e) {
      console.error("Failed to send confirmed file:", e);
    }
  };

  const autoDownloadImage = async (msg: any, customPeerId?: string) => {
    if (msg.contentType === 'file' && msg.fileInfo) {
      const isImage = msg.fileInfo.name.match(/\.(jpg|jpeg|png|gif|webp)$/i);
      const isSmall = msg.fileInfo.size < 20 * 1024 * 1024;
      if (isImage && isSmall && !msg.localPath && !msg.isDownloading) {
        const peerId = customPeerId || (msg.senderId === selfInfo.value?.id ? msg.peerId : (msg.peerId === 'lobby' ? 'lobby' : msg.senderId));
        
        // Mark as downloading in a reactive way
        if (chats.value[peerId]) {
          const idx = chats.value[peerId].findIndex((m: any) => m.messageId === msg.messageId);
          if (idx !== -1) {
            chats.value[peerId][idx] = {
              ...chats.value[peerId][idx],
              isDownloading: true
            };
          }
        }

        try {
          const { appCacheDir, join } = await import('@tauri-apps/api/path');
          const cacheDir = await appCacheDir();
          const savePath = await join(cacheDir, "LanChat_Thumbnails", `${msg.fileInfo.fileId}_${msg.fileInfo.name}`);
          
          await invoke("download_file", {
            peerId: msg.senderId,
            fileId: msg.fileInfo.fileId,
            fileName: msg.fileInfo.name,
            fileSize: msg.fileInfo.size,
            savePath: savePath
          });
          
          // 不在此处设置 localPath — 文件可能尚未写入完成
          // download-success 事件会在文件实际写入后设置 localPath
        } catch (e) {
          console.error("Auto download failed", e);
          if (chats.value[peerId]) {
            const idx = chats.value[peerId].findIndex((m: any) => m.messageId === msg.messageId);
            if (idx !== -1) {
              chats.value[peerId][idx] = {
                ...chats.value[peerId][idx],
                isDownloading: false
              };
            }
          }
        }
      }
    }
  };

  const downloadFile = async (msg: any) => {
    if (!msg.fileInfo) return;
    try {
      const savePath = await invoke<string | null>("select_save_path", { defaultName: msg.fileInfo.name });
      if (!savePath) return; // User cancelled
      
      if (msg.localPath) {
        if (msg.localPath === savePath) {
          showToast("文件已保存至该位置！", "info");
          return;
        }
        // File already on disk (either sent by us or auto-downloaded), just copy it (Save As)
        await invoke("save_as_file", { source: msg.localPath, dest: savePath });
        showToast("文件已另存为至:\n" + savePath, "success");
        return;
      }
      
      msg.isDownloading = true;
      msg.downloadBytesProcessed = 0;
      msg.downloadTotalBytes = msg.fileInfo.size || 0;
      msg.downloadSpeed = "计算中...";
      msg.downloadEta = "计算中...";
      msg.downloadStartTime = Date.now();
      downloadProgress.value.show = false; // reset
      
      await invoke("download_file", {
        peerId: msg.senderId,
        fileId: msg.fileInfo.fileId,
        fileName: msg.fileInfo.name,
        fileSize: msg.fileInfo.size,
        savePath: savePath
      });
      // Do NOT set localPath here. Wait for download-success event.
    } catch (e: any) {
      console.error("Failed to download file:", e);
      msg.isDownloading = false;
    }
  };

  const openFile = async (path: string) => {
    try {
      const { openPath } = await import('@tauri-apps/plugin-opener');
      await openPath(path);
    } catch (e: any) {
      console.error("Failed to open file:", e);
      showToast("无法打开文件: " + (e.message || e), "error");
    }
  };
  
  const openImagePreview = async (path: string) => {
    try {
      const urlPath = `http://localhost.tauri/` + path.replace(/\\/g, '/');
      const webview = new WebviewWindow(`preview-${Date.now()}`, {
        url: `preview.html?img=${encodeURIComponent(urlPath)}`,
        title: '图片预览',
        width: 800,
        height: 600,
        center: true
      });
      webview.once('tauri://error', function (e) {
        console.error('Error creating webview:', e);
      });
    } catch (e) {
      console.error("Failed to open preview:", e);
    }
  };

  const formatSpeed = (bytesPerSec: number) => {
    if (bytesPerSec > 1024 * 1024) return (bytesPerSec / (1024 * 1024)).toFixed(2) + ' MB/s';
    if (bytesPerSec > 1024) return (bytesPerSec / 1024).toFixed(2) + ' KB/s';
    return bytesPerSec.toFixed(0) + ' B/s';
  };

  const formatTime = (seconds: number) => {
    if (seconds < 60) return `${Math.ceil(seconds)}秒`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}分 ${Math.ceil(seconds % 60)}秒`;
    return `${Math.floor(seconds / 3600)}小时`;
  };

  return {
    setupFileListeners,
    selectAndShareFile,
    sendConfirmedFile,
    sendClipboardImage,
    autoDownloadImage,
    downloadFile,
    openFile,
    openImagePreview,
    cancelHash,
    skipHash
  };
}
