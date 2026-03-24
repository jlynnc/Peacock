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
  <div :class="['device-item', { active: selected }]">
    <DeviceAvatar :device-id="device.device_id" :platform="device.platform" />
    <div class="device-info">
      <div class="device-name">{{ device.device_name }}</div>
      <div class="device-meta">
        {{ formatPlatform(device.platform) }} · {{ device.ip_addr }}
      </div>
    </div>
    <span v-if="unread > 0" class="unread-badge">{{ unread > 99 ? '99+' : unread }}</span>
  </div>
</template>

<style scoped>
.device-item {
  display: flex;
  align-items: center;
  padding: 10px;
  border-radius: 10px;
  margin-bottom: 2px;
  cursor: pointer;
  gap: 10px;
  transition: all 0.15s;
}

.device-item:hover {
  background: #f5f5f5;
}

.device-item.active {
  background: rgba(13, 148, 136, 0.06);
}

.device-info {
  flex: 1;
  min-width: 0;
}

.device-name {
  font-size: 13px;
  font-weight: 500;
  color: #1a1a1a;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.device-meta {
  font-size: 11px;
  color: #aaa;
  margin-top: 1px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.unread-badge {
  min-width: 18px;
  height: 18px;
  background: #0d9488;
  color: #fff;
  font-size: 10px;
  font-weight: 600;
  border-radius: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 5px;
  flex-shrink: 0;
}
</style>
