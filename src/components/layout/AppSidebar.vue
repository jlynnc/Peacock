<script setup lang="ts">
import { ref, watch } from "vue";
import { Settings } from "lucide-vue-next";
import DeviceList from "@/components/device/DeviceList.vue";
import SnippetList from "@/components/snippet/SnippetList.vue";
import { useDeviceStore } from "@/stores/device";

const deviceStore = useDeviceStore();

type TabType = "devices" | "snippets";
const activeTab = ref<TabType>("devices");
const searchQuery = ref("");

const emit = defineEmits<{
  "tab-change": [tab: string];
  "open-settings": [];
}>();

watch(activeTab, (tab) => {
  emit("tab-change", tab);
});

function getSelfInitial(): string {
  const name = deviceStore.selfInfo?.device_name;
  if (!name) return "P";
  return name.charAt(0).toUpperCase();
}
</script>

<template>
  <aside class="sidebar">
    <!-- Branding -->
    <div class="sidebar-branding" data-tauri-drag-region>
      <div class="brand-logo">P</div>
      <span class="brand-name">Peacock</span>
    </div>

    <!-- Tabs -->
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

    <!-- Search -->
    <div class="sidebar-search">
      <input
        v-model="searchQuery"
        type="text"
        class="search-input"
        placeholder="搜索..."
      />
    </div>

    <!-- Content -->
    <div class="sidebar-content">
      <DeviceList v-if="activeTab === 'devices'" />
      <SnippetList v-else-if="activeTab === 'snippets'" />
    </div>

    <!-- Footer -->
    <div class="sidebar-footer">
      <div class="self-avatar">{{ getSelfInitial() }}</div>
      <span class="self-name">{{
        deviceStore.selfInfo?.device_name || "未连接"
      }}</span>
      <button
        class="settings-btn"
        title="设置"
        @click="emit('open-settings')"
      >
        <Settings :size="16" />
      </button>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width, 240px);
  background: var(--color-bg-sidebar);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow: hidden;
}

/* Branding */
.sidebar-branding {
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 10px;
  -webkit-app-region: drag;
}

.brand-logo {
  width: 32px;
  height: 32px;
  background: linear-gradient(135deg, #0d9488, #14b8a6);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 16px;
  font-weight: 700;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.brand-name {
  font-size: 16px;
  font-weight: 700;
  color: var(--color-text);
}

/* Tabs */
.sidebar-tabs {
  padding: 0 12px;
  display: flex;
  gap: 2px;
  margin-bottom: 8px;
  flex-shrink: 0;
}

.tab-btn {
  flex: 1;
  padding: 7px 0;
  text-align: center;
  font-size: 13px;
  color: #999;
  border-radius: 6px;
  border: none;
  background: none;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tab-btn:hover {
  color: #666;
  background: #f0f0f0;
}

.tab-btn.active {
  color: var(--color-primary);
  font-weight: 600;
  background: var(--color-primary-light);
}

/* Search */
.sidebar-search {
  padding: 0 12px 8px;
  flex-shrink: 0;
}

.search-input {
  width: 100%;
  padding: 7px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 8px;
  font-size: 12px;
  background: var(--color-bg-surface);
  color: var(--color-text);
  outline: none;
  transition: border-color 0.15s ease;
  box-sizing: border-box;
}

.search-input::placeholder {
  color: var(--color-text-placeholder);
}

.search-input:focus {
  border-color: var(--color-primary);
}

/* Content */
.sidebar-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

/* Footer */
.sidebar-footer {
  padding: 10px 12px;
  border-top: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.self-avatar {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  background: linear-gradient(135deg, #0d9488, #14b8a6);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  flex-shrink: 0;
}

.self-name {
  font-size: 12px;
  color: #666;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.settings-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: none;
  color: #bbb;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.settings-btn:hover {
  background: #f0f0f0;
  color: #666;
}
</style>
