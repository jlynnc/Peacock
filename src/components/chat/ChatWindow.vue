<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted, computed } from "vue";
import type { DeviceInfo } from "@/types/device";
import { useChatStore } from "@/stores/chat";
import { formatPlatform } from "@/utils/format";
import { sendPaths } from "@/utils/ipc";
import { isTauri } from "@/utils/platform";
import ChatBubble from "./ChatBubble.vue";
import ChatInput from "./ChatInput.vue";

const props = defineProps<{
  device: DeviceInfo;
}>();

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
    <div class="chat-header">
      <div class="chat-header-info">
        <span class="chat-device-name">{{ device.device_name }}</span>
        <span class="chat-device-meta">
          {{ formatPlatform(device.platform) }} · {{ device.ip_addr }}
        </span>
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
      />
    </div>
    <!-- Drag overlay -->
    <div v-if="isDragOver" class="drag-overlay">
      <div class="drag-hint">
        <span class="drag-icon">📂</span>
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
}

.chat-header {
  padding: 12px 20px;
  border-bottom: 1px solid var(--color-border);
  background: #ffffff;
  flex-shrink: 0;
}

.chat-header-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.chat-device-name {
  font-size: 15px;
  font-weight: 600;
}

.chat-device-meta {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.message-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.no-messages {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.drag-overlay {
  position: absolute;
  inset: 0;
  background: rgba(76, 175, 80, 0.08);
  border: 2px dashed var(--color-primary);
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
  color: var(--color-primary);
  font-size: 16px;
  font-weight: 500;
}

.drag-icon {
  font-size: 48px;
}
</style>
