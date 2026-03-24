<script setup lang="ts">
import { computed } from "vue";
import type { DevicePlatform } from "@/types/device";

const props = defineProps<{
  deviceId: string;
  platform: DevicePlatform;
}>();

const platformIcon = computed(() => {
  const icons: Record<string, string> = {
    windows: "💻",
    macos: "🖥️",
    linux: "🐧",
    android: "📱",
    ios: "📱",
    web: "🌐",
  };
  return icons[props.platform] || "💻";
});

// Generate a consistent color from device_id
const bgColor = computed(() => {
  let hash = 0;
  for (let i = 0; i < props.deviceId.length; i++) {
    hash = props.deviceId.charCodeAt(i) + ((hash << 5) - hash);
  }
  const hue = Math.abs(hash % 360);
  return `hsl(${hue}, 55%, 65%)`;
});
</script>

<template>
  <div class="device-avatar" :style="{ background: bgColor }">
    <span class="avatar-icon">{{ platformIcon }}</span>
  </div>
</template>

<style scoped>
.device-avatar {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.avatar-icon {
  font-size: 20px;
}
</style>
