<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Settings } from "lucide-vue-next";
import DeviceList from "@/components/device/DeviceList.vue";
import SnippetList from "@/components/snippet/SnippetList.vue";
import RoomList from "@/components/room/RoomList.vue";
import { useDeviceStore } from "@/stores/device";
import { useSettingsStore } from "@/stores/settings";

const { t } = useI18n();

const deviceStore = useDeviceStore();
const settingsStore = useSettingsStore();

// sidebarTab is in the store so it can be changed from anywhere (e.g. "save to snippet")
const searchQuery = ref("");

const emit = defineEmits<{
  "tab-change": [tab: string];
  "open-settings": [];
}>();

watch(() => deviceStore.sidebarTab, (tab) => {
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
        :class="['tab-btn', { active: deviceStore.sidebarTab === 'devices' }]"
        @click="deviceStore.sidebarTab = 'devices'"
      >
        {{ $t('tabs.devices') }}
      </button>
      <button
        :class="['tab-btn', { active: deviceStore.sidebarTab === 'snippets' }]"
        @click="deviceStore.sidebarTab = 'snippets'"
      >
        {{ $t('tabs.snippets') }}
      </button>
      <button
        :class="['tab-btn', { active: deviceStore.sidebarTab === 'rooms' }]"
        @click="deviceStore.sidebarTab = 'rooms'"
      >
        群聊
      </button>
      <button
        v-if="settingsStore.debugMode"
        :class="['tab-btn', { active: deviceStore.sidebarTab === 'debug' }]"
        @click="deviceStore.sidebarTab = 'debug'"
      >
        🔧
      </button>
    </div>

    <!-- Search (only for devices tab; snippets has its own) -->
    <div v-if="deviceStore.sidebarTab === 'devices'" class="sidebar-search">
      <input
        v-model="searchQuery"
        type="text"
        class="search-input"
        :placeholder="$t('search.placeholder')"
      />
    </div>

    <!-- Content -->
    <div class="sidebar-content">
      <DeviceList v-if="deviceStore.sidebarTab === 'devices'" />
      <SnippetList v-else-if="deviceStore.sidebarTab === 'snippets'" />
      <RoomList v-else-if="deviceStore.sidebarTab === 'rooms'" />
    </div>

    <!-- Footer -->
    <div class="sidebar-footer">
      <div class="self-avatar">{{ getSelfInitial() }}</div>
      <span class="self-name">{{
        deviceStore.selfInfo?.device_name || t('device.notConnected')
      }}</span>
      <button
        class="settings-btn"
        :title="$t('tabs.settings')"
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
  color: var(--color-text-secondary);
  border-radius: 6px;
  border: none;
  background: none;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tab-btn:hover {
  color: var(--color-text-secondary);
  background: var(--color-bg-input);
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
  color: var(--color-text-secondary);
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
  color: var(--color-text-muted);
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.settings-btn:hover {
  background: var(--color-bg-input);
  color: var(--color-text-secondary);
}
</style>
