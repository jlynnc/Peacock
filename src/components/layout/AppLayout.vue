<script setup lang="ts">
import { ref } from "vue";
import AppSidebar from "./AppSidebar.vue";
import AppHeader from "./AppHeader.vue";
import ChatWindow from "@/components/chat/ChatWindow.vue";
import SettingsModal from "@/components/settings/SettingsModal.vue";
import SnippetPanel from "@/components/snippet/SnippetPanel.vue";
import { useDeviceStore } from "@/stores/device";

const deviceStore = useDeviceStore();
const showSettings = ref(false);
const activeView = ref("devices");

function onTabChange(tab: string) {
  activeView.value = tab;
}
</script>

<template>
  <div class="app-layout">
    <AppHeader @open-settings="showSettings = true" />
    <SettingsModal v-if="showSettings" @close="showSettings = false" />
    <div class="app-body">
      <AppSidebar @tab-change="onTabChange" />
      <main class="app-main">
        <!-- Snippets: full panel with own list + editor -->
        <SnippetPanel v-if="activeView === 'snippets'" />

        <!-- Chat view -->
        <ChatWindow
          v-else-if="deviceStore.selectedDevice"
          :device="deviceStore.selectedDevice"
        />

        <!-- Empty state -->
        <div v-else class="empty-state">
          <div class="empty-icon">🦚</div>
          <h2>Peacock</h2>
          <p>选择左侧设备开始聊天</p>
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
}

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.app-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-chat);
  overflow: hidden;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  gap: 8px;
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 12px;
}

.empty-state h2 {
  font-size: 20px;
  font-weight: 500;
  color: var(--color-text);
}

.empty-state p {
  font-size: 14px;
}
</style>
