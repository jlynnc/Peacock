<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "@/utils/platform";
import { X } from "lucide-vue-next";

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
        <button class="close-btn" @click="emit('close')">
          <X :size="18" />
        </button>
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
  background: #fff;
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
  border-bottom: 1px solid #f0f0f0;
}

.modal-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: #1a1a1a;
}

.close-btn {
  border: none;
  background: none;
  cursor: pointer;
  color: #bbb;
  padding: 4px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.close-btn:hover {
  background: #f5f5f5;
  color: #666;
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
  color: #1a1a1a;
}

.setting-input {
  padding: 8px 12px;
  border: 1px solid #eee;
  border-radius: 8px;
  font-size: 13px;
  outline: none;
  background: #f7f8f9;
  color: #1a1a1a;
  transition: border-color 0.15s;
}
.setting-input::placeholder {
  color: #ccc;
}
.setting-input:focus {
  border-color: #0d9488;
}

.setting-hint {
  font-size: 12px;
  color: #aaa;
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
  border: 1px solid #eee;
  border-radius: 8px;
  background: #f7f8f9;
  color: #666;
  font-size: 13px;
  cursor: pointer;
  flex-shrink: 0;
  transition: all 0.15s;
}

.pick-btn:hover {
  border-color: #0d9488;
  color: #0d9488;
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
  accent-color: #0d9488;
}

.toggle-text {
  font-size: 13px;
  color: #1a1a1a;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid #f0f0f0;
}

.btn-cancel {
  padding: 8px 20px;
  border: none;
  border-radius: 8px;
  background: #f5f5f5;
  color: #666;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s;
}

.btn-cancel:hover {
  background: #eee;
}

.btn-save {
  padding: 8px 20px;
  border: none;
  border-radius: 8px;
  background: #0d9488;
  color: #fff;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-save:hover {
  background: #0f766e;
}
</style>
