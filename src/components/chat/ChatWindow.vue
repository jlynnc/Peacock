<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted, computed } from "vue";
import type { DeviceInfo } from "@/types/device";
import { useChatStore } from "@/stores/chat";
import { formatPlatform } from "@/utils/format";
import { sendPaths } from "@/utils/ipc";
import { isTauri } from "@/utils/platform";
import { Search, Minus, Square, X, FolderOpen } from "lucide-vue-next";
import ChatBubble from "./ChatBubble.vue";
import ChatInput from "./ChatInput.vue";

const props = defineProps<{
  device: DeviceInfo;
}>();

const emit = defineEmits<{
  minimize: [];
  maximize: [];
  close: [];
}>();

const searchQuery = ref("");

const chatStore = useChatStore();
const messageListRef = ref<HTMLElement | null>(null);
const isDragOver = ref(false);

const messages = computed(() => chatStore.getMessages(props.device.device_id));

let unlistenDragDrop: (() => void) | null = null;

watch(
  () => props.device.device_id,
  () => {
    chatStore.markAsRead(props.device.device_id);
    scrollToBottom();
  },
);

watch(
  () => messages.value.length,
  () => {
    scrollToBottom();
  },
);

function scrollToBottom() {
  nextTick(() => {
    if (messageListRef.value) {
      messageListRef.value.scrollTop = messageListRef.value.scrollHeight;
    }
  });
}

async function handleSend(text: string) {
  await chatStore.sendMessage(props.device.device_id, text);
  scrollToBottom();
}

async function setupDragDrop() {
  if (!isTauri()) return;
  try {
    const { getCurrentWebview } = await import("@tauri-apps/api/webview");
    const webview = getCurrentWebview();
    unlistenDragDrop = await webview.onDragDropEvent(async (event) => {
      if (event.payload.type === "over") {
        isDragOver.value = true;
      } else if (event.payload.type === "leave") {
        isDragOver.value = false;
      } else if (event.payload.type === "drop") {
        isDragOver.value = false;
        const paths = event.payload.paths;
        if (paths && paths.length > 0) {
          await handleDropPaths(paths);
        }
      }
    });
  } catch (e) {
    console.error("Failed to setup drag-drop:", e);
  }
}

async function handleDropPaths(paths: string[]) {
  if (!props.device.device_id) return;
  try {
    const transferIds = await sendPaths(props.device.device_id, paths);
    // Add file messages to chat for each transfer
    for (let i = 0; i < transferIds.length; i++) {
      const path = paths[i] || "";
      const name = path.split(/[/\\]/).pop() || path;
      chatStore.addFileMessage(
        props.device.device_id,
        transferIds[i],
        name,
        0,
        "sent",
        "pending",
      );
    }
  } catch (e) {
    console.error("Failed to send dropped files:", e);
  }
}

onMounted(() => {
  chatStore.markAsRead(props.device.device_id);
  scrollToBottom();
  setupDragDrop();
});

onUnmounted(() => {
  if (unlistenDragDrop) {
    unlistenDragDrop();
    unlistenDragDrop = null;
  }
});
</script>

<template>
  <div class="chat-window" :class="{ 'drag-over': isDragOver }">
    <!-- Unified title bar: device info + search + window controls -->
    <div class="unified-titlebar" data-tauri-drag-region>
      <div class="titlebar-left">
        <span class="chat-device-name">{{ device.device_name }}</span>
        <span class="status-badge">
          <span class="status-dot"></span>
          {{ formatPlatform(device.platform) }}
        </span>
      </div>
      <div class="titlebar-center" data-tauri-drag-region>
        <div class="search-box">
          <Search :size="16" class="search-icon" />
          <input
            v-model="searchQuery"
            class="search-input"
            type="text"
            placeholder="搜索消息..."
          />
        </div>
      </div>
      <div class="titlebar-right">
        <button class="win-btn" @click="emit('minimize')"><Minus :size="14" :stroke-width="1.5" /></button>
        <button class="win-btn" @click="emit('maximize')"><Square :size="12" :stroke-width="1.5" /></button>
        <button class="win-btn win-close" @click="emit('close')"><X :size="14" :stroke-width="1.5" /></button>
      </div>
    </div>
    <div ref="messageListRef" class="message-list">
      <div v-if="messages.length === 0" class="no-messages">
        <p>暂无消息，发送第一条消息吧</p>
      </div>
      <ChatBubble
        v-for="msg in messages"
        :key="msg.transfer_id || msg.id"
        :message="msg"
        :device-name="device.device_name"
      />
    </div>
    <!-- Drag overlay -->
    <div v-if="isDragOver" class="drag-overlay">
      <div class="drag-hint">
        <FolderOpen :size="48" />
        <span>释放以发送文件</span>
      </div>
    </div>
    <ChatInput @send="handleSend" :device-id="device.device_id" />
  </div>
</template>

<style scoped>
.chat-window {
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
  background: #fff;
}

/* Unified title bar */
.unified-titlebar {
  display: flex;
  align-items: center;
  height: 56px;
  padding: 0 12px 0 20px;
  border-bottom: 1px solid #f0f0f0;
  flex-shrink: 0;
  -webkit-app-region: drag;
  gap: 16px;
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.chat-device-name {
  font-size: 15px;
  font-weight: 600;
  color: #1a1a1a;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 10px;
  background: #ecfdf5;
  color: #059669;
  border-radius: 10px;
  font-size: 12px;
}

.status-dot {
  width: 6px;
  height: 6px;
  background: #059669;
  border-radius: 50%;
}

.titlebar-center {
  flex: 1;
  display: flex;
  justify-content: center;
  -webkit-app-region: drag;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  background: #f7f8f9;
  border: 1px solid #eee;
  border-radius: 10px;
  padding: 7px 14px;
  width: 240px;
  max-width: 300px;
  transition: border-color 0.15s;
  -webkit-app-region: no-drag;
}

.search-box:focus-within {
  border-color: #0d9488;
}

.search-icon {
  color: #bbb;
  flex-shrink: 0;
  transition: color 0.15s;
}

.search-box:focus-within .search-icon {
  color: #0d9488;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font-size: 13px;
  font-family: inherit;
  color: #333;
  line-height: 1.5;
}

.search-input::placeholder {
  color: #ccc;
}

.titlebar-right {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.win-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: none;
  color: #bbb;
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.win-btn:hover {
  background: #f0f0f0;
  color: #666;
}

.win-close:hover {
  background: #fee2e2;
  color: #ef4444;
}

.message-list {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  background: #fafbfc;
}

.no-messages {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #bbb;
  font-size: 13px;
}

.drag-overlay {
  position: absolute;
  inset: 0;
  background: rgba(13, 148, 136, 0.04);
  border: 2px dashed #0d9488;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  pointer-events: none;
}

.drag-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: #0d9488;
  font-size: 16px;
  font-weight: 500;
}
</style>
