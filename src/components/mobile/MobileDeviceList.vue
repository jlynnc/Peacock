<script setup lang="ts">
import { ref, computed } from "vue";
import { useRouter } from "vue-router";
import { useDeviceStore } from "@/stores/device";
import { useChatStore } from "@/stores/chat";
import { formatPlatform, formatTime } from "@/utils/format";
import { Wifi } from "lucide-vue-next";
import DeviceAvatar from "@/components/device/DeviceAvatar.vue";

const router = useRouter();
const deviceStore = useDeviceStore();
const chatStore = useChatStore();
const searchQuery = ref("");

const filteredDevices = computed(() => {
  const devices = deviceStore.onlineDevices;
  if (!searchQuery.value) return devices;
  const q = searchQuery.value.toLowerCase();
  return devices.filter(
    (d) =>
      d.device_name.toLowerCase().includes(q) ||
      d.ip_addr.includes(q),
  );
});

function getLastMessage(deviceId: string): string {
  const msgs = chatStore.getMessages(deviceId);
  if (msgs.length === 0) return "";
  const last = msgs[msgs.length - 1];
  return last.content;
}

function getLastMessageTime(deviceId: string): string {
  const msgs = chatStore.getMessages(deviceId);
  if (msgs.length === 0) return "";
  const last = msgs[msgs.length - 1];
  return formatTime(last.timestamp);
}

function openChat(deviceId: string) {
  deviceStore.selectDevice(deviceId);
  chatStore.markAsRead(deviceId);
  router.push({ name: "mobile-chat", params: { deviceId } });
}
</script>

<template>
  <div class="mobile-device-list">
    <div class="page-header">
      <h1 class="page-title">{{'设备'}}</h1>
    </div>

    <div class="search-bar">
      <input
        v-model="searchQuery"
        type="text"
        class="search-input"
        placeholder="搜索设备..."
      />
    </div>

    <div v-if="filteredDevices.length === 0" class="empty-state">
      <div class="scanning-icon">
        <Wifi :size="32" color="#c7c7cc" />
      </div>
      <p class="empty-text">正在搜索设备...</p>
      <p class="empty-hint">请确保其他设备在同一局域网</p>
    </div>

    <div v-else class="device-items">
      <div
        v-for="device in filteredDevices"
        :key="device.device_id"
        class="device-row"
        @click="openChat(device.device_id)"
      >
        <div class="device-avatar-wrap">
          <DeviceAvatar
            :device-id="device.device_id"
            :platform="device.platform"
          />
          <span class="online-dot"></span>
        </div>
        <div class="device-info">
          <div class="device-top-row">
            <span class="device-name">{{ device.device_name }}</span>
            <span class="device-time">{{ getLastMessageTime(device.device_id) }}</span>
          </div>
          <div class="device-bottom-row">
            <span class="device-preview">
              {{ getLastMessage(device.device_id) || formatPlatform(device.platform) + ' \u00B7 ' + device.ip_addr }}
            </span>
            <span
              v-if="chatStore.getUnreadCount(device.device_id) > 0"
              class="unread-badge"
            >
              {{ chatStore.getUnreadCount(device.device_id) > 99 ? '99+' : chatStore.getUnreadCount(device.device_id) }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mobile-device-list {
  min-height: 100%;
  background: #f2f2f7;
}

.page-header {
  padding: 16px 16px 0;
  padding-top: calc(16px + env(safe-area-inset-top, 0px));
}

.page-title {
  font-size: 30px;
  font-weight: 800;
  color: #000;
  margin: 0;
  letter-spacing: -0.5px;
}

.search-bar {
  padding: 10px 16px 6px;
}

.search-input {
  width: 100%;
  padding: 10px 14px;
  border: none;
  border-radius: 12px;
  font-size: 16px;
  background: #e5e5ea;
  color: #000;
  outline: none;
  -webkit-appearance: none;
}

.search-input::placeholder {
  color: #8e8e93;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 60px 20px;
  gap: 8px;
}

.scanning-icon {
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.empty-text {
  font-size: 15px;
  color: #8e8e93;
}

.empty-hint {
  font-size: 13px;
  color: #c7c7cc;
}

.device-items {
  padding: 6px 16px;
}

.device-row {
  display: flex;
  align-items: center;
  gap: 13px;
  padding: 12px 14px;
  background: #fff;
  border-radius: 14px;
  margin-bottom: 8px;
  cursor: pointer;
  -webkit-tap-highlight-color: transparent;
  transition: background 0.15s;
}

.device-row:active {
  background: #f0f0f0;
}

.device-avatar-wrap {
  position: relative;
  flex-shrink: 0;
}

.device-avatar-wrap :deep(.device-avatar),
.device-avatar-wrap :deep(> div),
.device-avatar-wrap :deep(> span) {
  width: 48px !important;
  height: 48px !important;
  border-radius: 14px !important;
  font-size: 18px !important;
}

.online-dot {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 12px;
  height: 12px;
  background: #34c759;
  border: 2.5px solid #fff;
  border-radius: 50%;
}

.device-info {
  flex: 1;
  min-width: 0;
}

.device-top-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.device-name {
  font-size: 16px;
  font-weight: 600;
  color: #000;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.device-time {
  font-size: 13px;
  color: #8e8e93;
  flex-shrink: 0;
  margin-left: 8px;
}

.device-bottom-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.device-preview {
  font-size: 14px;
  color: #8e8e93;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.unread-badge {
  min-width: 20px;
  height: 20px;
  background: #0d9488;
  color: #fff;
  font-size: 12px;
  font-weight: 600;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 6px;
  flex-shrink: 0;
  margin-left: 8px;
}
</style>
