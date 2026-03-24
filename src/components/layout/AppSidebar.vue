<script setup lang="ts">
import { ref, watch } from "vue";
import DeviceList from "@/components/device/DeviceList.vue";
import SnippetList from "@/components/snippet/SnippetList.vue";

type TabType = "devices" | "snippets";
const activeTab = ref<TabType>("devices");

const emit = defineEmits<{
  "tab-change": [tab: string];
}>();

watch(activeTab, (tab) => {
  emit("tab-change", tab);
});
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-tabs">
      <button
        :class="['tab-btn', { active: activeTab === 'devices' }]"
        @click="activeTab = 'devices'"
      >
        设备
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'snippets' }]"
        @click="activeTab = 'snippets'"
      >
        片段
      </button>
    </div>
    <div class="sidebar-content">
      <DeviceList v-if="activeTab === 'devices'" />
      <SnippetList v-else-if="activeTab === 'snippets'" />
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  background: var(--color-bg-sidebar);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow: hidden;
}

.sidebar-tabs {
  display: flex;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.tab-btn {
  flex: 1;
  padding: 10px 0;
  border: none;
  background: none;
  font-size: 13px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}

.tab-btn:hover {
  color: var(--color-text);
  background: rgba(0, 0, 0, 0.03);
}

.tab-btn.active {
  color: var(--color-primary);
  font-weight: 500;
}

.tab-btn.active::after {
  content: "";
  position: absolute;
  bottom: 0;
  left: 20%;
  width: 60%;
  height: 2px;
  background: var(--color-primary);
  border-radius: 1px;
}

.sidebar-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-secondary);
  font-size: 13px;
}
</style>
