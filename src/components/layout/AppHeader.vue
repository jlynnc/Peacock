<script setup lang="ts">
import { useDeviceStore } from "@/stores/device";

const deviceStore = useDeviceStore();

const emit = defineEmits<{
  openSettings: [];
}>();
</script>

<template>
  <header class="app-header" data-tauri-drag-region>
    <div class="header-left">
      <span class="app-logo">🦚</span>
      <span class="app-title">Peacock</span>
    </div>
    <div class="header-center">
      <span v-if="deviceStore.selfInfo" class="self-name">
        {{ deviceStore.selfInfo.device_name }}
      </span>
    </div>
    <div class="header-right">
      <span class="online-count">
        {{ deviceStore.onlineCount }} 台设备在线
      </span>
      <button class="settings-btn" @click="emit('openSettings')" title="设置">⚙️</button>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  height: var(--header-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: #2e2e2e;
  color: #ffffff;
  -webkit-app-region: drag;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  -webkit-app-region: no-drag;
}

.app-logo {
  font-size: 20px;
}

.app-title {
  font-size: 15px;
  font-weight: 600;
}

.header-center {
  flex: 1;
  text-align: center;
}

.self-name {
  font-size: 13px;
  opacity: 0.8;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
  -webkit-app-region: no-drag;
}

.online-count {
  font-size: 12px;
  opacity: 0.7;
}

.settings-btn {
  border: none;
  background: none;
  font-size: 18px;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  opacity: 0.7;
  transition: opacity 0.15s;
}

.settings-btn:hover {
  opacity: 1;
}
</style>
