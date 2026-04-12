<script setup lang="ts">
import { computed } from "vue";
import type { DevicePlatform } from "@/types/device";

const props = withDefaults(
  defineProps<{
    deviceId: string;
    platform: DevicePlatform;
    online?: boolean;
    restricted?: boolean;
  }>(),
  { online: true, restricted: false }
);

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

const bgColor = computed(() => {
  const colors: Record<string, string> = {
    windows: "#e8f0fe",
    macos: "#e8f0fe",
    ios: "#f3e8ff",
    android: "#f3e8ff",
    linux: "#e0f2f1",
  };
  return colors[props.platform] || "#f5f5f5";
});
</script>

<template>
  <div class="device-avatar" :style="{ background: bgColor }">
    <span class="avatar-icon">{{ platformIcon }}</span>
    <span v-if="online" :class="['online-dot', { restricted }]"></span>
  </div>
</template>

<style scoped>
.device-avatar {
  position: relative;
  width: 38px;
  height: 38px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.avatar-icon {
  font-size: 18px;
}

.online-dot {
  position: absolute;
  bottom: -1px;
  right: -1px;
  width: 10px;
  height: 10px;
  background: #22c55e;
  border: 2px solid #fafafa;
  border-radius: 50%;
}

.online-dot.restricted {
  background: #f97316;
}
</style>
