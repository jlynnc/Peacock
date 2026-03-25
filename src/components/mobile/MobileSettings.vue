<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { isTauri } from "@/utils/platform";
import {
  Smartphone,
  FolderDown,
  FileCheck,
  Moon,
  Globe,
  Info,
  ChevronRight,
} from "lucide-vue-next";

const settingsStore = useSettingsStore();
const editName = ref(settingsStore.deviceName);
const isEditingName = ref(false);

onMounted(async () => {
  await settingsStore.loadSettings();
  editName.value = settingsStore.deviceName;
});

async function saveName() {
  const name = editName.value.trim();
  if (name && name !== settingsStore.deviceName) {
    await settingsStore.setDeviceName(name);
  }
  isEditingName.value = false;
}

async function pickDownloadDir() {
  if (!isTauri()) return;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const dir = await open({ directory: true, title: "\u9009\u62E9\u9ED8\u8BA4\u4E0B\u8F7D\u76EE\u5F55" });
  if (dir) {
    await settingsStore.setDownloadDir(dir as string);
  }
}

function toggleAutoAccept() {
  settingsStore.setAutoAccept(!settingsStore.autoAcceptFiles);
}

function toggleTheme() {
  const newTheme = settingsStore.theme === "light" ? "dark" : "light";
  settingsStore.setTheme(newTheme);
}
</script>

<template>
  <div class="mobile-settings">
    <div class="page-header">
      <h1 class="page-title">设置</h1>
    </div>

    <div class="settings-content">
      <!-- Device section -->
      <div class="settings-group">
        <div class="settings-row" @click="isEditingName = true">
          <div class="row-icon"><Smartphone :size="18" color="#0d9488" /></div>
          <div class="row-body">
            <span class="row-label">设备名称</span>
            <div class="row-right">
              <input
                v-if="isEditingName"
                v-model="editName"
                class="inline-input"
                @blur="saveName"
                @keydown.enter="saveName"
                autofocus
                @click.stop
              />
              <span v-else class="row-value">{{ settingsStore.deviceName }}</span>
              <ChevronRight v-if="!isEditingName" :size="16" color="#c7c7cc" />
            </div>
          </div>
        </div>

        <div class="settings-row" @click="pickDownloadDir">
          <div class="row-icon"><FolderDown :size="18" color="#0d9488" /></div>
          <div class="row-body">
            <span class="row-label">默认下载目录</span>
            <div class="row-right">
              <span class="row-value row-value-path">{{ settingsStore.downloadDir || '未设置' }}</span>
              <ChevronRight :size="16" color="#c7c7cc" />
            </div>
          </div>
        </div>
      </div>

      <!-- Preferences section -->
      <div class="settings-group">
        <div class="settings-row" @click="toggleAutoAccept">
          <div class="row-icon"><FileCheck :size="18" color="#0d9488" /></div>
          <div class="row-body">
            <span class="row-label">自动接收文件</span>
            <div class="row-right">
              <div :class="['toggle-switch', { on: settingsStore.autoAcceptFiles }]">
                <div class="toggle-thumb"></div>
              </div>
            </div>
          </div>
        </div>

        <div class="settings-row" @click="toggleTheme">
          <div class="row-icon"><Moon :size="18" color="#0d9488" /></div>
          <div class="row-body">
            <span class="row-label">暗色主题</span>
            <div class="row-right">
              <div :class="['toggle-switch', { on: settingsStore.theme === 'dark' }]">
                <div class="toggle-thumb"></div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Info section -->
      <div class="settings-group">
        <div class="settings-row">
          <div class="row-icon"><Globe :size="18" color="#0d9488" /></div>
          <div class="row-body">
            <span class="row-label">语言</span>
            <div class="row-right">
              <span class="row-value">简体中文</span>
              <ChevronRight :size="16" color="#c7c7cc" />
            </div>
          </div>
        </div>

        <div class="settings-row">
          <div class="row-icon"><Info :size="18" color="#0d9488" /></div>
          <div class="row-body">
            <span class="row-label">关于 Peacock</span>
            <div class="row-right">
              <span class="row-value">v0.1.0</span>
              <ChevronRight :size="16" color="#c7c7cc" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mobile-settings {
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

.settings-content {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.settings-group {
  background: #fff;
  border-radius: 14px;
  overflow: hidden;
}

.settings-row {
  display: flex;
  align-items: center;
  padding: 0 14px;
  min-height: 48px;
  cursor: pointer;
  -webkit-tap-highlight-color: transparent;
}

.settings-row:active {
  background: #f2f2f7;
}

.settings-row + .settings-row {
  border-top: 0.5px solid #e5e5ea;
}

.row-icon {
  width: 32px;
  height: 32px;
  background: rgba(13, 148, 136, 0.08);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-right: 12px;
}

.row-body {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 48px;
  min-width: 0;
}

.row-label {
  font-size: 16px;
  color: #000;
  flex-shrink: 0;
}

.row-right {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.row-value {
  font-size: 15px;
  color: #8e8e93;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-value-path {
  max-width: 140px;
  direction: rtl;
  text-align: right;
}

.inline-input {
  font-size: 15px;
  color: #000;
  border: none;
  outline: none;
  background: transparent;
  text-align: right;
  padding: 4px 0;
  width: 140px;
  -webkit-appearance: none;
}

.toggle-switch {
  width: 50px;
  height: 30px;
  border-radius: 15px;
  background: #e5e5ea;
  position: relative;
  transition: background 0.25s;
  flex-shrink: 0;
}

.toggle-switch.on {
  background: #0d9488;
}

.toggle-thumb {
  width: 26px;
  height: 26px;
  border-radius: 13px;
  background: #fff;
  position: absolute;
  top: 2px;
  left: 2px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
  transition: transform 0.25s;
}

.toggle-switch.on .toggle-thumb {
  transform: translateX(20px);
}
</style>
