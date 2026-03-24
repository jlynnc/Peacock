<script setup lang="ts">
import { ref } from "vue";
import { Bird, Minus, Square, X } from "lucide-vue-next";
import AppSidebar from "./AppSidebar.vue";
import ChatWindow from "@/components/chat/ChatWindow.vue";
import SettingsModal from "@/components/settings/SettingsModal.vue";
import SnippetPanel from "@/components/snippet/SnippetPanel.vue";
import { useDeviceStore } from "@/stores/device";
import { isTauri } from "@/utils/platform";

const deviceStore = useDeviceStore();
const showSettings = ref(false);
const activeView = ref("devices");

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
    <div class="app-body">
      <AppSidebar
        @tab-change="onTabChange"
        @open-settings="showSettings = true"
      />
      <main class="app-main">
        <SnippetPanel
          v-if="activeView === 'snippets'"
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
            <p class="empty-subtitle">选择左侧设备开始聊天</p>
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
  color: #bbb;
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
  -webkit-app-region: no-drag;
}

.win-btn:hover {
  background: #f0f0f0;
  color: #666;
}

.win-close:hover {
  background: #fee2e2;
  color: #ef4444;
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
  color: #ddd;
  opacity: 0.6;
  margin-bottom: 8px;
}

.empty-title {
  font-size: 20px;
  font-weight: 600;
  color: #ccc;
  margin: 0;
}

.empty-subtitle {
  font-size: 13px;
  color: #ddd;
  margin: 0;
}
</style>
