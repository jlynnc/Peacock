<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useDeviceStore } from "@/stores/device";
import { useChatStore } from "@/stores/chat";
import { formatPlatform } from "@/utils/format";
import { isTauri } from "@/utils/platform";
import { ChevronLeft, Paperclip, FolderOpen, Send } from "lucide-vue-next";
import ChatBubble from "@/components/chat/ChatBubble.vue";

const route = useRoute();
const router = useRouter();
const deviceStore = useDeviceStore();
const chatStore = useChatStore();

const deviceId = computed(() => route.params.deviceId as string);
const device = computed(() => deviceStore.devices.get(deviceId.value) || null);
const messages = computed(() => chatStore.getMessages(deviceId.value));

const inputText = ref("");
const messageListRef = ref<HTMLElement | null>(null);
const isSendingFile = ref(false);

watch(
  () => messages.value.length,
  () => scrollToBottom(),
);

function scrollToBottom() {
  nextTick(() => {
    if (messageListRef.value) {
      messageListRef.value.scrollTop = messageListRef.value.scrollHeight;
    }
  });
}

async function handleSend() {
  const text = inputText.value.trim();
  if (!text) return;
  await chatStore.sendMessage(deviceId.value, text);
  inputText.value = "";
  scrollToBottom();
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    handleSend();
  }
}

async function handleFilePick() {
  if (!isTauri()) return;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({ multiple: true, title: "\u9009\u62E9\u8981\u53D1\u9001\u7684\u6587\u4EF6" });
  if (!selected) return;
  const files = Array.isArray(selected) ? selected : [selected];
  isSendingFile.value = true;
  try {
    const { sendFile } = await import("@/utils/ipc");
    for (const filePath of files) {
      const fileName = filePath.split(/[/\\]/).pop() || filePath;
      const transferId = await sendFile(deviceId.value, filePath);
      chatStore.addFileMessage(deviceId.value, transferId, fileName, 0, "sent", "pending");
    }
  } catch (e) {
    console.error("Failed to send file:", e);
  } finally {
    isSendingFile.value = false;
  }
}

async function handleFolderPick() {
  if (!isTauri()) return;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({ directory: true, title: "\u9009\u62E9\u8981\u53D1\u9001\u7684\u6587\u4EF6\u5939" });
  if (!selected) return;
  const folderPath = selected as string;
  isSendingFile.value = true;
  try {
    const { sendFolder } = await import("@/utils/ipc");
    const folderName = folderPath.split(/[/\\]/).pop() || folderPath;
    const transferId = await sendFolder(deviceId.value, folderPath);
    chatStore.addFileMessage(deviceId.value, transferId, folderName, 0, "sent", "pending");
  } catch (e) {
    console.error("Failed to send folder:", e);
  } finally {
    isSendingFile.value = false;
  }
}

function goBack() {
  router.back();
}

onMounted(() => {
  chatStore.markAsRead(deviceId.value);
  scrollToBottom();
});
</script>

<template>
  <div class="mobile-chat">
    <!-- Top nav bar -->
    <div class="chat-nav">
      <button class="back-btn" @click="goBack">
        <ChevronLeft :size="28" />
      </button>
      <div class="nav-center">
        <span class="nav-device-name">{{ device?.device_name || '...' }}</span>
        <span v-if="device" class="nav-status">
          <span class="status-dot"></span>
          {{ formatPlatform(device.platform) }}
        </span>
      </div>
      <div class="nav-spacer"></div>
    </div>

    <!-- Messages -->
    <div ref="messageListRef" class="message-list">
      <div v-if="messages.length === 0" class="no-messages">
        <p>暂无消息，发送第一条消息吧</p>
      </div>
      <ChatBubble
        v-for="msg in messages"
        :key="msg.transfer_id || msg.id"
        :message="msg"
        :device-name="device?.device_name || ''"
      />
    </div>

    <!-- Input bar -->
    <div class="input-bar">
      <div class="input-wrapper">
        <button class="tool-btn" @click="handleFilePick" :disabled="isSendingFile">
          <Paperclip :size="20" />
        </button>
        <button class="tool-btn" @click="handleFolderPick" :disabled="isSendingFile">
          <FolderOpen :size="20" />
        </button>
        <input
          v-model="inputText"
          class="input-field"
          type="text"
          placeholder="输入消息..."
          @keydown="handleKeydown"
        />
        <button
          :class="['send-btn', { active: inputText.trim().length > 0 }]"
          @click="handleSend"
        >
          <Send :size="16" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mobile-chat {
  display: flex;
  flex-direction: column;
  height: 100vh;
  height: 100dvh;
  background: #f2f2f7;
}

.chat-nav {
  display: flex;
  align-items: center;
  padding: 8px 4px;
  padding-top: calc(8px + env(safe-area-inset-top, 0px));
  background: #fff;
  border-bottom: 0.5px solid #d1d1d6;
  flex-shrink: 0;
  min-height: 44px;
}

.back-btn {
  width: 44px;
  height: 44px;
  border: none;
  background: none;
  color: #0d9488;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
  -webkit-tap-highlight-color: transparent;
}

.nav-center {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 0;
}

.nav-device-name {
  font-size: 16px;
  font-weight: 600;
  color: #000;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-status {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #34c759;
}

.status-dot {
  width: 6px;
  height: 6px;
  background: #34c759;
  border-radius: 50%;
}

.nav-spacer {
  width: 44px;
  flex-shrink: 0;
}

.message-list {
  flex: 1;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  background: #f2f2f7;
}

.no-messages {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #8e8e93;
  font-size: 14px;
}

.input-bar {
  padding: 8px 12px;
  padding-bottom: calc(8px + env(safe-area-inset-bottom, 0px));
  background: #fff;
  border-top: 0.5px solid #d1d1d6;
  flex-shrink: 0;
}

.input-wrapper {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #f2f2f7;
  border-radius: 20px;
  padding: 6px 8px;
}

.tool-btn {
  width: 34px;
  height: 34px;
  border: none;
  background: none;
  color: #8e8e93;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  -webkit-tap-highlight-color: transparent;
}

.tool-btn:active {
  background: rgba(0, 0, 0, 0.05);
}

.tool-btn:disabled {
  opacity: 0.3;
}

.input-field {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font-size: 16px;
  color: #000;
  padding: 6px 4px;
  -webkit-appearance: none;
}

.input-field::placeholder {
  color: #8e8e93;
}

.send-btn {
  width: 34px;
  height: 34px;
  border: none;
  background: #d1d1d6;
  color: #fff;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: background 0.15s;
  -webkit-tap-highlight-color: transparent;
}

.send-btn.active {
  background: #0d9488;
}
</style>
