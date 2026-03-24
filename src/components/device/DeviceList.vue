<script setup lang="ts">
import { useDeviceStore } from "@/stores/device";
import { useChatStore } from "@/stores/chat";
import DeviceItem from "./DeviceItem.vue";

const deviceStore = useDeviceStore();
const chatStore = useChatStore();

function handleSelect(deviceId: string) {
  deviceStore.selectDevice(deviceId);
  chatStore.markAsRead(deviceId);
}
</script>

<template>
  <div class="device-list">
    <div v-if="deviceStore.onlineDevices.length === 0" class="no-devices">
      <div class="scanning-icon">📡</div>
      <p>正在搜索设备...</p>
      <p class="hint">请确保其他设备在同一局域网</p>
    </div>
    <DeviceItem
      v-for="device in deviceStore.onlineDevices"
      :key="device.device_id"
      :device="device"
      :selected="deviceStore.selectedDeviceId === device.device_id"
      :unread="chatStore.getUnreadCount(device.device_id)"
      @click="handleSelect(device.device_id)"
    />
  </div>
</template>

<style scoped>
.device-list {
  padding: 4px 0;
}

.no-devices {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--color-text-secondary);
  text-align: center;
  gap: 8px;
}

.scanning-icon {
  font-size: 32px;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.4;
  }
}

.no-devices p {
  font-size: 13px;
}

.hint {
  font-size: 12px !important;
  opacity: 0.6;
}
</style>
