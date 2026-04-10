<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useDeviceStore } from "@/stores/device";
import { useChatStore } from "@/stores/chat";
import { formatPlatform } from "@/utils/format";
import { isTauri } from "@/utils/platform";
import { useI18n } from "vue-i18n";
import { ChevronLeft, Send, Plus, Image, Camera, FileText, BookOpen, X } from "lucide-vue-next";
import ChatBubble from "@/components/chat/ChatBubble.vue";
import { useSwipeBack } from "@/composables/useSwipeBack";

useSwipeBack();

const route = useRoute();
const router = useRouter();
const deviceStore = useDeviceStore();
const chatStore = useChatStore();

const deviceId = computed(() => route.params.deviceId as string);
const device = computed(() => deviceStore.devices.get(deviceId.value) || null);
const messages = computed(() => chatStore.getMessages(deviceId.value));

const inputText = computed({
  get() {
    return chatStore.getConversation(deviceId.value).draft || "";
  },
  set(val: string) {
    chatStore.getConversation(deviceId.value).draft = val;
  },
});
const messageListRef = ref<HTMLElement | null>(null);
useI18n();
const isSendingFile = ref(false);
const showPlusPanel = ref(false);
const fileInputRef = ref<HTMLInputElement | null>(null);
const photoInputRef = ref<HTMLInputElement | null>(null);
const cameraInputRef = ref<HTMLInputElement | null>(null);

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
  const conv = chatStore.getConversation(deviceId.value);
  conv.draft = "";
  scrollToBottom();
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    handleSend();
  }
}

function togglePlusPanel() {
  showPlusPanel.value = !showPlusPanel.value;
}

function pickPhotos() {
  showPlusPanel.value = false;
  photoInputRef.value?.click();
}

function pickCamera() {
  showPlusPanel.value = false;
  cameraInputRef.value?.click();
}

function pickFiles() {
  showPlusPanel.value = false;
  fileInputRef.value?.click();
}

function pickSnippet() {
  showPlusPanel.value = false;
  // Navigate to snippet picker — for now go to snippets tab
  router.push({ name: "mobile-snippets" });
}

async function handleFileSelected(e: Event, _isPhoto: boolean) {
  const input = e.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) return;

  isSendingFile.value = true;
  try {
    if (isTauri()) {
      // On Tauri, we need to get the file path
      // For files selected via HTML input on iOS, we need to use a different approach
      // Write the file to a temp location and send from there
      const { invoke } = await import("@tauri-apps/api/core");

      for (const file of input.files) {
        const arrayBuffer = await file.arrayBuffer();
        const bytes = new Uint8Array(arrayBuffer);

        // Write to temp file and send
        const transferId = await invoke("send_file_bytes", {
          deviceId: deviceId.value,
          fileName: file.name,
          fileData: Array.from(bytes),
        });

        chatStore.addFileMessage(
          deviceId.value,
          transferId as string,
          file.name,
          file.size,
          "sent",
          "pending",
        );
      }
    }
  } catch (err) {
    console.error("Failed to send file:", err);
  } finally {
    isSendingFile.value = false;
    input.value = ""; // reset input
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
    <div ref="messageListRef" class="message-list" @click="showPlusPanel = false">
      <div v-if="messages.length === 0" class="no-messages">
        <p>{{ $t('chat.noMessages') }}</p>
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
        <input
          v-model="inputText"
          class="input-field"
          type="text"
          :placeholder="$t('chat.inputPlaceholderMobile')"
          @keydown="handleKeydown"
          @focus="showPlusPanel = false"
        />
        <button
          v-if="inputText.trim().length > 0"
          class="send-btn active"
          @click="handleSend"
        >
          <Send :size="16" />
        </button>
        <button
          v-else
          :class="['plus-btn', { active: showPlusPanel }]"
          @click="togglePlusPanel"
        >
          <Plus :size="22" v-if="!showPlusPanel" />
          <X :size="22" v-else />
        </button>
      </div>
    </div>

    <!-- Plus panel (WeChat style) -->
    <div v-if="showPlusPanel" class="plus-panel">
      <div class="panel-grid">
        <div class="panel-item" @click="pickPhotos">
          <div class="panel-icon">
            <Image :size="28" />
          </div>
          <span class="panel-label">{{ $t('chat.photos') || '照片' }}</span>
        </div>
        <div class="panel-item" @click="pickCamera">
          <div class="panel-icon">
            <Camera :size="28" />
          </div>
          <span class="panel-label">{{ $t('chat.camera') || '拍摄' }}</span>
        </div>
        <div class="panel-item" @click="pickSnippet">
          <div class="panel-icon">
            <BookOpen :size="28" />
          </div>
          <span class="panel-label">{{ $t('chat.snippet') || '片段' }}</span>
        </div>
        <div class="panel-item" @click="pickFiles">
          <div class="panel-icon">
            <FileText :size="28" />
          </div>
          <span class="panel-label">{{ $t('chat.file') || '文件' }}</span>
        </div>
      </div>
    </div>

    <!-- Hidden file inputs -->
    <input
      ref="photoInputRef"
      type="file"
      accept="image/*,video/*"
      multiple
      style="display:none"
      @change="handleFileSelected($event, true)"
    />
    <input
      ref="cameraInputRef"
      type="file"
      accept="image/*"
      capture="environment"
      style="display:none"
      @change="handleFileSelected($event, true)"
    />
    <input
      ref="fileInputRef"
      type="file"
      multiple
      style="display:none"
      @change="handleFileSelected($event, false)"
    />
  </div>
</template>

<style scoped>
.mobile-chat {
  display: flex;
  flex-direction: column;
  height: 100vh;
  height: 100dvh;
  background: var(--color-ios-bg);
}

.chat-nav {
  display: flex;
  align-items: center;
  padding: 8px 4px;
  padding-top: calc(8px + env(safe-area-inset-top, 0px));
  background: var(--color-ios-card);
  border-bottom: 0.5px solid var(--color-ios-border);
  flex-shrink: 0;
  min-height: 44px;
}

.back-btn {
  width: 44px;
  height: 44px;
  border: none;
  background: none;
  color: var(--color-primary);
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
  color: var(--color-ios-text);
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
  background: var(--color-ios-bg);
}

.no-messages {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-ios-text-secondary);
  font-size: 14px;
}

.input-bar {
  padding: 8px 12px;
  background: var(--color-ios-card);
  border-top: 0.5px solid var(--color-ios-border);
  flex-shrink: 0;
}

.input-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--color-ios-input-bg);
  border-radius: 20px;
  padding: 6px 6px 6px 14px;
}

.input-field {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font-size: 16px;
  color: var(--color-ios-text);
  padding: 6px 0;
  -webkit-appearance: none;
}

.input-field::placeholder {
  color: var(--color-ios-text-secondary);
}

.send-btn, .plus-btn {
  width: 34px;
  height: 34px;
  border: none;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  -webkit-tap-highlight-color: transparent;
  transition: background 0.15s;
}

.send-btn {
  background: var(--color-ios-border);
  color: #fff;
}

.send-btn.active {
  background: var(--color-primary);
}

.plus-btn {
  background: none;
  color: var(--color-ios-text-secondary);
}

.plus-btn.active {
  color: var(--color-primary);
}

/* Plus panel */
.plus-panel {
  background: var(--color-ios-card);
  border-top: 0.5px solid var(--color-ios-border);
  padding: 16px 24px;
  padding-bottom: calc(16px + env(safe-area-inset-bottom, 0px));
  flex-shrink: 0;
}

.panel-grid {
  display: flex;
  gap: 24px;
}

.panel-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  -webkit-tap-highlight-color: transparent;
}

.panel-icon {
  width: 56px;
  height: 56px;
  background: var(--color-ios-input-bg);
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-ios-text);
  transition: background 0.15s;
}

.panel-item:active .panel-icon {
  background: var(--color-ios-border);
}

.panel-label {
  font-size: 12px;
  color: var(--color-ios-text-secondary);
}
</style>
