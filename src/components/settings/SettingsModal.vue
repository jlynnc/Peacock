<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/stores/settings";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "@/utils/platform";
import { setLocale, getLocale } from "@/i18n";
import { X } from "lucide-vue-next";

const { t } = useI18n();
const currentLocale = ref(getLocale());

const emit = defineEmits<{
  close: [];
}>();

const settingsStore = useSettingsStore();
const editName = ref(settingsStore.deviceName);
const editDir = ref(settingsStore.downloadDir);

onMounted(async () => {
  if (isTauri()) {
    try {
      const dir = await invoke<string>("get_download_dir");
      editDir.value = dir;
      settingsStore.downloadDir = dir;
    } catch (e) {
      console.error("Failed to get download dir:", e);
    }
  }
});

async function pickDownloadDir() {
  if (!isTauri()) return;
  const dir = await open({ directory: true, title: t('settings.downloadDir') });
  if (dir) {
    editDir.value = dir as string;
  }
}

async function save() {
  if (editName.value.trim() && editName.value !== settingsStore.deviceName) {
    await settingsStore.setDeviceName(editName.value.trim());
  }
  if (editDir.value && editDir.value !== settingsStore.downloadDir) {
    await settingsStore.setDownloadDir(editDir.value);
  }
  emit("close");
}

function handleOverlayClick(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains("modal-overlay")) {
    emit("close");
  }
}
</script>

<template>
  <div class="modal-overlay" @click="handleOverlayClick">
    <div class="modal-dialog">
      <div class="modal-header">
        <h3>{{ $t('settings.title') }}</h3>
        <button class="close-btn" @click="emit('close')">
          <X :size="18" />
        </button>
      </div>
      <div class="modal-body">
        <div class="setting-group">
          <label class="setting-label">{{ $t('settings.deviceName') }}</label>
          <input
            v-model="editName"
            class="setting-input"
            :placeholder="$t('settings.deviceNamePlaceholder')"
          />
          <p class="setting-hint">{{ $t('settings.deviceNameHint') }}</p>
        </div>

        <div class="setting-group">
          <label class="setting-label">{{ $t('settings.downloadDir') }}</label>
          <div class="dir-picker">
            <input
              v-model="editDir"
              class="setting-input dir-input"
              :placeholder="$t('settings.downloadDirPlaceholder')"
              readonly
            />
            <button class="pick-btn" @click="pickDownloadDir">{{ $t('settings.browse') }}</button>
          </div>
          <p class="setting-hint">{{ $t('settings.downloadDirHint') }}</p>
        </div>

        <div class="setting-group">
          <label class="setting-label">{{ $t('settings.autoAccept') }}</label>
          <div class="setting-row">
            <label class="toggle-label">
              <input
                type="checkbox"
                v-model="settingsStore.autoAcceptFiles"
                class="toggle-input"
              />
              <span class="toggle-text">{{ $t('settings.autoAcceptToggle') }}</span>
            </label>
          </div>
          <p class="setting-hint">
            {{ $t('settings.autoAcceptHint') }}
          </p>
        </div>

        <div class="setting-group">
          <label class="setting-label">{{ $t('settings.autoStart') }}</label>
          <div class="setting-row">
            <label class="toggle-label">
              <input
                type="checkbox"
                :checked="settingsStore.autoStart"
                @change="settingsStore.setAutoStart(($event.target as HTMLInputElement).checked)"
                class="toggle-input"
              />
              <span class="toggle-text">{{ $t('settings.autoStartToggle') }}</span>
            </label>
          </div>
          <p class="setting-hint">
            {{ $t('settings.autoStartHint') }}
          </p>
        </div>

        <div class="setting-group">
          <label class="setting-label">{{ $t('settings.maxConcurrent') }}</label>
          <div class="setting-row">
            <select
              class="setting-select"
              :value="settingsStore.maxConcurrent"
              @change="settingsStore.setMaxConcurrent(Number(($event.target as HTMLSelectElement).value))"
            >
              <option v-for="n in [1, 3, 5, 10, 20, 50]" :key="n" :value="n">{{ n }}</option>
            </select>
          </div>
          <p class="setting-hint">{{ $t('settings.maxConcurrentHint') }}</p>
        </div>

        <div class="setting-group">
          <label class="setting-label">{{ $t('settings.contextMenu') }}</label>
          <div class="setting-row">
            <label class="toggle-label">
              <input
                type="checkbox"
                :checked="settingsStore.contextMenu"
                @change="settingsStore.setContextMenu(($event.target as HTMLInputElement).checked)"
                class="toggle-input"
              />
              <span class="toggle-text">{{ $t('settings.contextMenuToggle') }}</span>
            </label>
          </div>
          <p class="setting-hint">
            {{ $t('settings.contextMenuHint') }}
          </p>
        </div>

        <div class="setting-group">
          <label class="setting-label">{{ $t('settings.darkTheme') }}</label>
          <div class="setting-row">
            <select
              class="setting-select"
              :value="settingsStore.theme"
              @change="settingsStore.setTheme(($event.target as HTMLSelectElement).value as 'system' | 'light' | 'dark')"
            >
              <option value="system">{{ $t('settings.language') === '语言' ? '跟随系统' : 'Follow System' }}</option>
              <option value="light">{{ $t('settings.language') === '语言' ? '亮色' : 'Light' }}</option>
              <option value="dark">{{ $t('settings.language') === '语言' ? '暗色' : 'Dark' }}</option>
            </select>
          </div>
        </div>

        <div class="setting-group">
          <label class="setting-label">{{ $t('settings.language') }}</label>
          <div class="setting-row">
            <select
              class="setting-select"
              v-model="currentLocale"
              @change="setLocale(currentLocale)"
            >
              <option value="zh-CN">简体中文</option>
              <option value="en">English</option>
            </select>
          </div>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn-cancel" @click="emit('close')">{{ $t('settings.cancel') }}</button>
        <button class="btn-save" @click="save">{{ $t('settings.save') }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translate(-50%, -46%);
  }
  to {
    opacity: 1;
    transform: translate(-50%, -50%);
  }
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  animation: fadeIn 0.2s ease;
}

.modal-dialog {
  background: var(--color-bg-surface);
  border-radius: 14px;
  width: 440px;
  max-height: 80vh;
  overflow-y: auto;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  animation: slideUp 0.25s ease;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
}

.modal-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}

.close-btn {
  border: none;
  background: none;
  cursor: pointer;
  color: var(--color-text-muted);
  padding: 4px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.close-btn:hover {
  background: var(--color-bg-input);
  color: var(--color-text-secondary);
}

.modal-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.setting-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
}

.setting-input {
  padding: 8px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 8px;
  font-size: 13px;
  outline: none;
  background: var(--color-bg-input);
  color: var(--color-text);
  transition: border-color 0.15s;
}
.setting-input::placeholder {
  color: var(--color-text-placeholder);
}
.setting-input:focus {
  border-color: var(--color-primary);
}

.setting-hint {
  font-size: 12px;
  color: var(--color-text-muted);
}

.dir-picker {
  display: flex;
  gap: 8px;
}

.dir-input {
  flex: 1;
  cursor: pointer;
}

.pick-btn {
  padding: 8px 16px;
  border: 1px solid var(--color-border-input);
  border-radius: 8px;
  background: var(--color-bg-input);
  color: var(--color-text-secondary);
  font-size: 13px;
  cursor: pointer;
  flex-shrink: 0;
  transition: all 0.15s;
}

.pick-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.setting-row {
  display: flex;
  align-items: center;
}

.toggle-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.toggle-input {
  width: 16px;
  height: 16px;
  accent-color: var(--color-primary);
}

.toggle-text {
  font-size: 13px;
  color: var(--color-text);
}

.setting-select {
  padding: 8px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 8px;
  font-size: 13px;
  outline: none;
  background: var(--color-bg-input);
  color: var(--color-text);
  cursor: pointer;
  min-width: 160px;
  transition: border-color 0.15s;
}
.setting-select:focus {
  border-color: var(--color-primary);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid var(--color-border);
}

.btn-cancel {
  padding: 8px 20px;
  border: none;
  border-radius: 8px;
  background: var(--color-bg-input);
  color: var(--color-text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s;
}

.btn-cancel:hover {
  background: var(--color-border);
}

.btn-save {
  padding: 8px 20px;
  border: none;
  border-radius: 8px;
  background: var(--color-primary);
  color: #fff;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-save:hover {
  background: var(--color-primary-hover);
}
</style>
