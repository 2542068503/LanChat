<template>
  <div
    v-if="show && msg"
    class="custom-context-menu msg-context-menu"
    :style="{ top: y + 'px', left: x + 'px' }"
  >
    <div class="context-item" @click.stop="handleReply">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
        <path d="M10 9V5l-7 7 7 7v-4.1c5 0 8.5 1.6 11 5.1-1-5-4-10-11-11z"/>
      </svg>
      引用 (回复)
    </div>
    
    <div v-if="msg.contentType === 'text'" class="context-item" @click.stop="handleCopy">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
        <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
      </svg>
      {{ selectedText ? '复制选中内容' : '复制内容' }}
    </div>
  </div>
</template>

<script setup>
import { onMounted, onUnmounted } from 'vue';

const props = defineProps({
  show: Boolean,
  x: Number,
  y: Number,
  msg: Object,
  selectedText: String
});

const emit = defineEmits(['close', 'reply', 'copy']);

// Click outside to close
const closeMenu = (e) => {
  if (props.show) {
    emit('close');
  }
};

onMounted(() => {
  document.addEventListener('click', closeMenu);
  document.addEventListener('contextmenu', closeMenu);
});

onUnmounted(() => {
  document.removeEventListener('click', closeMenu);
  document.removeEventListener('contextmenu', closeMenu);
});

const handleReply = () => {
  emit('reply', props.msg);
  emit('close');
};

const handleCopy = () => {
  emit('copy', props.msg, props.selectedText);
  emit('close');
};
</script>

<style scoped>
.custom-context-menu {
  position: fixed;
  background-color: var(--card-bg, #121215);
  border: 1px solid var(--border-color, #2e2e33);
  border-radius: 8px;
  padding: 6px 4px;
  min-width: 130px;
  z-index: 1000;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
}
.context-item {
  padding: 8px 14px;
  font-size: 13px;
  cursor: pointer;
  color: var(--text-primary, #e4e4e7);
  transition: all 0.15s;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
}
.context-item:hover {
  background-color: rgba(125, 125, 125, 0.15);
}
</style>
