import { nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { 
  selfInfo, chats, activePeerId,
  unreadCounts, currentTab, saveChatsToLocalStorage, getSenderProfile 
} from '../store';
import type { Message } from '../types';
import { useSettings } from './useSettings';
import { sendNotification } from '@tauri-apps/plugin-notification';

export function useChat() {
  const loadChatsFromLocalStorage = () => {
    const data = localStorage.getItem("chatHistory");
    if (data) {
      try {
        chats.value = JSON.parse(data);
      } catch (e) {
        console.error("Failed to parse chatHistory", e);
      }
    }
  };

  const appendMessage = (peerId: string, msg: Message) => {
    if (!chats.value[peerId]) {
      chats.value[peerId] = [];
    }
    
    // Check for duplicates
    if (!chats.value[peerId].some(m => m.messageId === msg.messageId)) {
      chats.value[peerId].push(msg);
      
      // Keep only last 1000 messages in memory
      if (chats.value[peerId].length > 1000) {
        chats.value[peerId].shift();
      }
      
      saveChatsToLocalStorage();
      
      nextTick(() => {
        scrollToBottom();
      });
      return true;
    }
    return false;
  };

  const scrollToBottom = () => {
    const container = document.querySelector('.msg-history');
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  };

  const setupChatListeners = async () => {
    await listen<Message>("message-received", (event) => {
      const msg = event.payload;
      // Ensure lobby messages go to lobby
      if (msg.senderId !== selfInfo.value.id) {
        const isNew = appendMessage(msg.senderId, msg);
        if (isNew && (currentTab.value !== 'chat' || activePeerId.value !== msg.senderId || !document.hasFocus())) {
          unreadCounts.value[msg.senderId] = (unreadCounts.value[msg.senderId] || 0) + 1;
          
          const { enableSystemNotification } = useSettings();
          if (enableSystemNotification.value) {
            const profile = getSenderProfile(msg.senderId, msg);
            let bodyText = msg.contentType === 'text' ? msg.content : `[${msg.contentType}]`;
            if (msg.contentType === 'file') bodyText = '[文件]';
            else if (msg.contentType === 'image') bodyText = '[图片]';
            
            try {
              sendNotification({
                title: `${profile.username} 发来新消息`,
                body: bodyText
              });
            } catch (e) {
              console.warn("Failed to send notification", e);
            }
          }
        }
      }
    });

    await listen<Message>("group-message-received", (event) => {
      const msg = event.payload;
      if (msg.senderId !== selfInfo.value.id) {
        const isNew = appendMessage("lobby", msg);
        if (isNew && (currentTab.value !== 'chat' || activePeerId.value !== 'lobby' || !document.hasFocus())) {
          unreadCounts.value['lobby'] = (unreadCounts.value['lobby'] || 0) + 1;
          
          const { enableSystemNotification } = useSettings();
          if (enableSystemNotification.value) {
            const profile = getSenderProfile(msg.senderId, msg);
            let bodyText = msg.contentType === 'text' ? msg.content : `[${msg.contentType}]`;
            if (msg.contentType === 'file') bodyText = '[文件]';
            else if (msg.contentType === 'image') bodyText = '[图片]';
            
            try {
              sendNotification({
                title: `局域网大厅 - ${profile.username}`,
                body: bodyText
              });
            } catch (e) {
              console.warn("Failed to send notification", e);
            }
          }
        }
      }
    });
  };

  const sendMessage = async (
    content: string, 
    contentType: string, 
    isLatex: boolean,
    replyTo: any
  ) => {
    if (!content.trim()) return false;
    if (!activePeerId.value) return false;

    const targetPeerId = activePeerId.value;
    const quoteMsgId = replyTo ? replyTo.messageId : null;
    const quoteSender = replyTo ? replyTo.senderName : null;
    const quoteContent = replyTo ? replyTo.content : null;

    try {
      if (targetPeerId === "lobby") {
        const msg = await invoke<Message>("send_group_message", {
          contentType: contentType,
          content: content,
          fileInfo: null,
          renderLatex: isLatex,
          quoteMsgId,
          quoteSender,
          quoteContent
        });
        msg.renderLatex = isLatex;
        appendMessage("lobby", msg);
      } else {
        const msg = await invoke<Message>("send_message", {
          peerId: targetPeerId,
          contentType: contentType,
          content: content,
          fileInfo: null,
          renderLatex: isLatex,
          quoteMsgId,
          quoteSender,
          quoteContent
        });
        msg.renderLatex = isLatex;
        appendMessage(targetPeerId, msg);
      }
      return true;
    } catch (e: any) {
      console.error("Failed to send message", e);
      return false;
    }
  };

  return {
    appendMessage,
    scrollToBottom,
    setupChatListeners,
    sendMessage,
    loadChatsFromLocalStorage
  };
}
