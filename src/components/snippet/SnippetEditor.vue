<script setup lang="ts">
import { ref, watch } from "vue";
import { useSnippetStore } from "@/stores/snippet";
import { useDeviceStore } from "@/stores/device";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

const store = useSnippetStore();
const deviceStore = useDeviceStore();

const title = ref("");
const content = ref("");
const note = ref("");
const copied = ref(false);
const showShareMenu = ref(false);
const confirmDelete = ref(false);

const saveStatus = ref<"saved" | "saving" | "idle">("idle");
let saveTimer: ReturnType<typeof setTimeout> | null = null;

// Sync local fields when selection changes
watch(
  () => store.selectedSnippet,
  (s) => {
    if (s) {
      title.value = s.title;
      content.value = s.content;
      note.value = s.note;
      saveStatus.value = "idle";
    }
  },
  { immediate: true },
);

function scheduleSave() {
  if (saveTimer) clearTimeout(saveTimer);
  saveStatus.value = "saving";
  saveTimer = setTimeout(async () => {
    if (!store.selectedId) return;
    await store.saveSnippet(store.selectedId, {
      title: title.value,
      content: content.value,
      note: note.value,
    });
    saveStatus.value = "saved";
  }, 600);
}

function onTitleInput() {
  scheduleSave();
}

function onContentInput() {
  scheduleSave();
}

function onNoteInput() {
  scheduleSave();
}

async function copyContent() {
  if (!content.value) return;
  try {
    await writeText(content.value);
    if (store.selectedId) store.incrementCopyCount(store.selectedId);
    copied.value = true;
    setTimeout(() => (copied.value = false), 1500);
  } catch (e) {
    console.error("Failed to copy:", e);
  }
}

async function shareToDevice(deviceId: string) {
  const s = store.selectedSnippet;
  if (!s) return;
  await store.shareToDevice(deviceId, s);
  showShareMenu.value = false;
}

async function doDelete() {
  if (!store.selectedId) return;
  await store.removeSnippet(store.selectedId);
  confirmDelete.value = false;
}

function formatDateTime(ts: number) {
  const d = new Date(ts * 1000);
  return d.toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}
</script>

<template>
  <div class="editor" v-if="store.selectedSnippet" @click="showShareMenu = false">
    <!-- Toolbar -->
    <div class="toolbar">
      <div class="toolbar-left">
        <span class="info-text">
          {{ formatDateTime(store.selectedSnippet.updated_at) }}
          ·
          <span v-if="saveStatus === 'saving'" class="save-status saving">保存中...</span>
          <span v-else-if="saveStatus === 'saved'" class="save-status saved">已保存</span>
        </span>
      </div>
      <div class="toolbar-right">
        <button
          :class="['btn', { 'btn-copied': copied }]"
          @click="copyContent"
        >
          {{ copied ? "已复制" : "复制内容" }}
        </button>
        <div class="share-wrap" @click.stop>
          <button class="btn" @click="showShareMenu = !showShareMenu">
            分享
          </button>
          <div class="share-menu" v-if="showShareMenu">
            <div
              v-for="[id, device] in deviceStore.devices"
              :key="id"
              class="share-device"
              @click="shareToDevice(id)"
            >
              <span>{{ device.device_name }}</span>
              <span class="share-ip">{{ device.ip_addr }}</span>
            </div>
            <div v-if="deviceStore.devices.size === 0" class="share-empty">
              暂无在线设备
            </div>
          </div>
        </div>
        <button class="btn btn-danger" @click="confirmDelete = true">
          删除
        </button>
      </div>
    </div>

    <!-- Title -->
    <div class="title-area">
      <input
        class="title-input"
        v-model="title"
        @input="onTitleInput"
        placeholder="标题"
      />
    </div>

    <!-- Content -->
    <div class="content-area">
      <textarea
        class="content-input"
        v-model="content"
        @input="onContentInput"
        placeholder="在此输入内容...&#10;&#10;例如 API Key、命令行、配置片段等"
      ></textarea>
    </div>

    <!-- Note -->
    <div class="note-area">
      <input
        class="note-input"
        v-model="note"
        @input="onNoteInput"
        placeholder="备注（可选）"
      />
    </div>

    <!-- Delete confirm overlay -->
    <div class="overlay" v-if="confirmDelete" @click="confirmDelete = false"></div>
    <div class="confirm-dialog" v-if="confirmDelete">
      <p>确定删除「{{ store.selectedSnippet.title }}」？</p>
      <div class="confirm-actions">
        <button class="btn" @click="confirmDelete = false">取消</button>
        <button class="btn btn-danger-solid" @click="doDelete">删除</button>
      </div>
    </div>
  </div>

  <!-- Empty state -->
  <div class="empty-editor" v-else>
    <div class="empty-icon">📋</div>
    <p>选择或新建一个片段</p>
  </div>
</template>

<style scoped>
.editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  align-items: center;
}

.info-text {
  font-size: 11px;
  color: var(--color-text-secondary);
}
.save-status {
  font-size: 11px;
}
.save-status.saving {
  color: var(--color-text-secondary);
}
.save-status.saved {
  color: var(--color-primary);
}

.toolbar-right {
  display: flex;
  gap: 6px;
  align-items: center;
}

.btn {
  padding: 4px 10px;
  border-radius: 4px;
  border: 1px solid var(--color-border);
  background: #fff;
  color: var(--color-text);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}
.btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}
.btn-copied {
  background: var(--color-primary);
  color: #fff;
  border-color: var(--color-primary);
}
.btn-danger:hover {
  border-color: #ff4d4f;
  color: #ff4d4f;
}
.btn-danger-solid {
  background: #ff4d4f;
  color: #fff;
  border-color: #ff4d4f;
}

.share-wrap {
  position: relative;
}

.share-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background: #fff;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  min-width: 180px;
  z-index: 20;
  padding: 4px;
}
.share-device {
  padding: 6px 10px;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  justify-content: space-between;
  font-size: 13px;
}
.share-device:hover {
  background: rgba(0, 0, 0, 0.04);
}
.share-ip {
  font-size: 11px;
  color: var(--color-text-secondary);
}
.share-empty {
  padding: 10px;
  text-align: center;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.title-area {
  padding: 16px 16px 0;
  flex-shrink: 0;
}

.title-input {
  width: 100%;
  font-size: 18px;
  font-weight: 600;
  border: none;
  outline: none;
  background: transparent;
  color: var(--color-text);
  padding: 0;
}
.title-input::placeholder {
  color: #ccc;
}

.content-area {
  flex: 1;
  padding: 12px 16px;
  overflow: hidden;
}

.content-input {
  width: 100%;
  height: 100%;
  font-family: "Cascadia Code", "Fira Code", "Consolas", monospace;
  font-size: 13px;
  line-height: 1.7;
  border: none;
  outline: none;
  background: transparent;
  color: var(--color-text);
  resize: none;
  padding: 0;
}
.content-input::placeholder {
  color: #ccc;
}

.note-area {
  padding: 0 16px 12px;
  flex-shrink: 0;
  border-top: 1px solid var(--color-border);
  padding-top: 8px;
}

.note-input {
  width: 100%;
  font-size: 12px;
  border: none;
  outline: none;
  background: transparent;
  color: var(--color-text-secondary);
  padding: 0;
}
.note-input::placeholder {
  color: #ccc;
}

.empty-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
}
.empty-icon {
  font-size: 48px;
  margin-bottom: 12px;
}
.empty-editor p {
  font-size: 14px;
}

.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.25);
  z-index: 50;
}
.confirm-dialog {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: #fff;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  z-index: 51;
  min-width: 260px;
}
.confirm-dialog p {
  font-size: 14px;
  margin-bottom: 14px;
}
.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
