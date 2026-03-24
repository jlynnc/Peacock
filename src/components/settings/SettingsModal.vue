<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "@/utils/platform";

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
  const dir = await open({ directory: true, title: "选择默认下载目录" });
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
        <h3>设置</h3>
        <button class="close-btn" @click="emit('close')">✕</button>
      </div>
      <div class="modal-body">
        <div class="setting-group">
          <label class="setting-label">设备名称</label>
          <input
            v-model="editName"
            class="setting-input"
            placeholder="输入设备名称"
          />
          <p class="setting-hint">在其他设备上显示的名称</p>
        </div>

        <div class="setting-group">
          <label class="setting-label">默认下载目录</label>
          <div class="dir-picker">
            <input
              v-model="editDir"
              class="setting-input dir-input"
              placeholder="选择下载目录"
              readonly
            />
            <button class="pick-btn" @click="pickDownloadDir">浏览</button>
          </div>
          <p class="setting-hint">接收文件的默认保存位置</p>
        </div>

        <div class="setting-group">
          <label class="setting-label">自动接收</label>
          <div class="setting-row">
            <label class="toggle-label">
              <input
                type="checkbox"
                v-model="settingsStore.autoAcceptFiles"
                class="toggle-input"
              />
              <span class="toggle-text">自动接收小文件</span>
            </label>
          </div>
          <p class="setting-hint">
            小于 10MB 的文件自动接收到默认目录
          </p>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn-cancel" @click="emit('close')">取消</button>
        <button class="btn-save" @click="save">保存</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.modal-dialog {
  background: white;
  border-radius: 12px;
  width: 440px;
  max-height: 80vh;
  overflow-y: auto;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
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
}

.close-btn {
  border: none;
  background: none;
  font-size: 16px;
  cursor: pointer;
  color: var(--color-text-secondary);
  padding: 4px 8px;
  border-radius: 4px;
}

.close-btn:hover {
  background: rgba(0, 0, 0, 0.05);
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
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 13px;
  outline: none;
  transition: border-color 0.15s;
}

.setting-input:focus {
  border-color: var(--color-primary);
}

.setting-hint {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.dir-picker {
  display: flex;
  gap: 8px;
}

.dir-input {
  flex: 1;
  cursor: pointer;
  background: #fafafa;
}

.pick-btn {
  padding: 8px 16px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: white;
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
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: white;
  font-size: 13px;
  cursor: pointer;
}

.btn-cancel:hover {
  background: #f5f5f5;
}

.btn-save {
  padding: 8px 20px;
  border: none;
  border-radius: 6px;
  background: var(--color-primary);
  color: white;
  font-size: 13px;
  cursor: pointer;
}

.btn-save:hover {
  background: var(--color-primary-hover);
}
</style>
