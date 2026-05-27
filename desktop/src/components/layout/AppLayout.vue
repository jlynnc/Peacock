<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { Bird, Minus, Square, X } from "lucide-vue-next";
import AppSidebar from "./AppSidebar.vue";
import ChatWindow from "@/components/chat/ChatWindow.vue";
import SettingsModal from "@/components/settings/SettingsModal.vue";
import SnippetPanel from "@/components/snippet/SnippetPanel.vue";
import RoomChatWindow from "@/components/room/RoomChatWindow.vue";
import DebugPanel from "@/components/debug/DebugPanel.vue";
import DevicePickerDialog from "@/components/common/DevicePickerDialog.vue";
import { useDeviceStore } from "@/stores/device";
import { useChatStore } from "@/stores/chat";
import { useRoomStore } from "@/stores/room";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauri } from "@/utils/platform";

const deviceStore = useDeviceStore();
const chatStore = useChatStore();
const roomStore = useRoomStore();
const showSettings = ref(false);
const activeView = ref("devices");

// Right-click "Send to Peacock" handling
const showSendPicker = ref(false);
const pendingSendPath = ref("");
let unlistenSendRequest: UnlistenFn | null = null;

onMounted(async () => {
  if (!isTauri()) return;
  unlistenSendRequest = await listen<string>("send-file-request", (event) => {
    pendingSendPath.value = event.payload;
    showSendPicker.value = true;
  });
});

onUnmounted(() => {
  unlistenSendRequest?.();
});

async function handleSendToDevices(deviceIds: string[]) {
  const filePath = pendingSendPath.value;
  showSendPicker.value = false;
  pendingSendPath.value = "";

  for (const deviceId of deviceIds) {
    try {
      await invoke("send_file", { deviceId, filePath });
      // Add file message to chat
      const fileName = filePath.split(/[/\\]/).pop() || filePath;
      chatStore.addFileMessage(deviceId, `send-${Date.now()}`, fileName, 0, "sent", "pending");
    } catch (e) {
      console.error("Failed to send file to device:", e);
    }
  }
}

function onTabChange(tab: string) {
  activeView.value = tab;
}

async function minimizeWindow() {
  if (!isTauri()) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  getCurrentWindow().minimize();
}

async function maximizeWindow() {
  if (!isTauri()) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  getCurrentWindow().toggleMaximize();
}

async function closeWindow() {
  if (!isTauri()) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  getCurrentWindow().close();
}
</script>

<template>
  <div class="app-layout">
    <SettingsModal v-if="showSettings" @close="showSettings = false" />
    <DevicePickerDialog
      v-if="showSendPicker"
      @close="showSendPicker = false; pendingSendPath = ''"
      @confirm="handleSendToDevices"
    />
    <div class="app-body">
      <AppSidebar
        @tab-change="onTabChange"
        @open-settings="showSettings = true"
      />
      <main class="app-main">
        <DebugPanel
          v-if="activeView === 'debug'"
          @minimize="minimizeWindow"
          @maximize="maximizeWindow"
          @close="closeWindow"
        />

        <SnippetPanel
          v-else-if="activeView === 'snippets'"
          @minimize="minimizeWindow"
          @maximize="maximizeWindow"
          @close="closeWindow"
        />

        <RoomChatWindow
          v-else-if="activeView === 'rooms' && roomStore.selectedRoom"
          :room="roomStore.selectedRoom"
          @minimize="minimizeWindow"
          @maximize="maximizeWindow"
          @close="closeWindow"
        />

        <ChatWindow
          v-else-if="deviceStore.selectedDevice"
          :device="deviceStore.selectedDevice"
          @minimize="minimizeWindow"
          @maximize="maximizeWindow"
          @close="closeWindow"
        />

        <div v-else class="empty-state">
          <!-- Minimal title bar for empty state -->
          <div class="empty-titlebar" data-tauri-drag-region>
            <div class="drag-spacer" data-tauri-drag-region></div>
            <button class="win-btn" @click="minimizeWindow"><Minus :size="14" :stroke-width="1.5" /></button>
            <button class="win-btn" @click="maximizeWindow"><Square :size="12" :stroke-width="1.5" /></button>
            <button class="win-btn win-close" @click="closeWindow"><X :size="14" :stroke-width="1.5" /></button>
          </div>
          <div class="empty-content">
            <div class="empty-icon">
              <Bird :size="48" />
            </div>
            <h2 class="empty-title">Peacock</h2>
            <p class="empty-subtitle">{{ $t('device.selectToChat') }}</p>
          </div>
        </div>
      </main>
    </div>
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--color-bg-surface);
  border-radius: 12px;
  overflow: hidden;
}

.app-body {
  display: flex;
  flex: 1;
  height: 100%;
  overflow: hidden;
}

.app-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-surface);
  overflow: hidden;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  user-select: none;
}

.empty-titlebar {
  display: flex;
  align-items: center;
  height: 56px;
  padding: 0 12px;
  flex-shrink: 0;
  -webkit-app-region: drag;
}

.drag-spacer {
  flex: 1;
  -webkit-app-region: drag;
}

.win-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: none;
  color: var(--color-text-muted);
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
  -webkit-app-region: no-drag;
}

.win-btn:hover {
  background: var(--color-bg-input);
  color: var(--color-text-secondary);
}

.win-close:hover {
  background: var(--color-danger-light);
  color: var(--color-danger);
}

.empty-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.empty-icon {
  color: var(--color-text-placeholder);
  opacity: 0.6;
  margin-bottom: 8px;
}

.empty-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text-placeholder);
  margin: 0;
}

.empty-subtitle {
  font-size: 13px;
  color: var(--color-text-placeholder);
  margin: 0;
}
</style>
