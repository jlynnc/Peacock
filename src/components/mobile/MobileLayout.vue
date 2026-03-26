<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { Smartphone, FileText, Settings } from "lucide-vue-next";

const route = useRoute();

const showTabBar = computed(() => {
  const name = route.name as string;
  return (
    name === "mobile-devices" ||
    name === "mobile-snippets" ||
    name === "mobile-settings"
  );
});

const tabs = [
  { name: "mobile-devices", path: "/devices", labelKey: "tabs.devices", icon: Smartphone },
  { name: "mobile-snippets", path: "/snippets", labelKey: "tabs.snippets", icon: FileText },
  { name: "mobile-settings", path: "/settings", labelKey: "tabs.settings", icon: Settings },
];
</script>

<template>
  <div class="mobile-layout">
    <div class="mobile-content">
      <router-view />
    </div>
    <nav v-if="showTabBar" class="tab-bar">
      <router-link
        v-for="tab in tabs"
        :key="tab.name"
        :to="tab.path"
        :class="['tab-item', { active: route.name === tab.name }]"
      >
        <component :is="tab.icon" :size="22" />
        <span class="tab-label">{{ $t(tab.labelKey) }}</span>
      </router-link>
    </nav>
  </div>
</template>

<style scoped>
.mobile-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--color-ios-bg);
  font-family: -apple-system, system-ui, "PingFang SC", "Hiragino Sans GB",
    "Microsoft YaHei", sans-serif;
}

.mobile-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  -webkit-overflow-scrolling: touch;
}

.tab-bar {
  display: flex;
  align-items: center;
  justify-content: space-around;
  background: var(--color-ios-card);
  border-top: 0.5px solid var(--color-ios-border);
  padding-bottom: env(safe-area-inset-bottom, 0px);
  height: calc(50px + env(safe-area-inset-bottom, 0px));
  flex-shrink: 0;
}

.tab-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  flex: 1;
  padding: 6px 0 2px;
  color: var(--color-ios-text-secondary);
  text-decoration: none;
  transition: color 0.2s;
  -webkit-tap-highlight-color: transparent;
}

.tab-item.active {
  color: var(--color-primary);
}

.tab-label {
  font-size: 10px;
  font-weight: 500;
  line-height: 1;
}
</style>
