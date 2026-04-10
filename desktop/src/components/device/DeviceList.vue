<script setup lang="ts">
import { useDeviceStore } from "@/stores/device";
import { useChatStore } from "@/stores/chat";
import { Wifi } from "lucide-vue-next";
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
      <div class="scanning-icon">
        <Wifi :size="24" color="#bbb" />
      </div>
      <p class="scanning-text">{{ $t('device.searching') }}</p>
      <p class="hint">{{ $t('device.searchHint') }}</p>
    </div>
    <div v-else class="device-list-container">
      <DeviceItem
        v-for="device in deviceStore.onlineDevices"
        :key="device.device_id"
        :device="device"
        :selected="deviceStore.selectedDeviceId === device.device_id"
        :unread="chatStore.getUnreadCount(device.device_id)"
        @click="handleSelect(device.device_id)"
      />
    </div>
  </div>
</template>

<style scoped>
.device-list {
  padding: 4px 0;
}

.device-list-container {
  padding: 0 8px;
}

.no-devices {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  text-align: center;
  gap: 8px;
}

.scanning-icon {
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.3;
  }
}

.scanning-text {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin: 0;
}

.hint {
  font-size: 12px;
  color: var(--color-text-placeholder);
  margin: 0;
}
</style>
