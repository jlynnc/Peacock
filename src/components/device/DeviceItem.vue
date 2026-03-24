<script setup lang="ts">
import type { DeviceInfo } from "@/types/device";
import { formatPlatform } from "@/utils/format";
import DeviceAvatar from "./DeviceAvatar.vue";

defineProps<{
  device: DeviceInfo;
  selected: boolean;
  unread: number;
}>();
</script>

<template>
  <div :class="['device-item', { selected }]">
    <DeviceAvatar :device-id="device.device_id" :platform="device.platform" />
    <div class="device-info">
      <div class="device-name">{{ device.device_name }}</div>
      <div class="device-meta">
        {{ formatPlatform(device.platform) }} · {{ device.ip_addr }}
      </div>
    </div>
    <div class="device-status">
      <span v-if="unread > 0" class="unread-badge">{{ unread > 99 ? '99+' : unread }}</span>
      <span class="online-dot"></span>
    </div>
  </div>
</template>

<style scoped>
.device-item {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  cursor: pointer;
  transition: background 0.15s;
  gap: 12px;
}

.device-item:hover {
  background: rgba(0, 0, 0, 0.04);
}

.device-item.selected {
  background: rgba(0, 0, 0, 0.08);
}

.device-info {
  flex: 1;
  min-width: 0;
}

.device-name {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.device-meta {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.device-status {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.online-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-online);
}

.unread-badge {
  background: #f44336;
  color: white;
  font-size: 11px;
  min-width: 18px;
  height: 18px;
  border-radius: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 5px;
  font-weight: 500;
}
</style>
