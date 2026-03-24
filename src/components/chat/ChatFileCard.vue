<script setup lang="ts">
import { computed, ref } from "vue";
import type { ChatMessage } from "@/types/message";
import { formatFileSize, formatSpeed } from "@/utils/format";
import { useTransferStore } from "@/stores/transfer";
import { useSettingsStore } from "@/stores/settings";
import { useChatStore } from "@/stores/chat";
import { openFileLocation, deleteFile } from "@/utils/ipc";
import { open } from "@tauri-apps/plugin-dialog";
import { isTauri } from "@/utils/platform";

const props = defineProps<{
  message: ChatMessage;
}>();

const transferStore = useTransferStore();
const settingsStore = useSettingsStore();
const chatStore = useChatStore();
const deleted = ref(false);

const progress = computed(() => {
  if (!props.message.file_size || props.message.file_size === 0) return 0;
  return Math.round(
    ((props.message.transferred_bytes || 0) / props.message.file_size) * 100,
  );
});

const isPending = computed(
  () =>
    props.message.transfer_status === "pending" &&
    props.message.direction === "received",
);
const isActive = computed(
  () =>
    props.message.transfer_status === "active" ||
    props.message.transfer_status === "paused",
);
const isPaused = computed(() => props.message.transfer_status === "paused");
const isCompleted = computed(
  () => props.message.transfer_status === "completed",
);

const statusLabel = computed(() => {
  if (deleted.value) return "文件已删除";
  switch (props.message.transfer_status) {
    case "pending":
      return props.message.direction === "sent" ? "等待对方接收..." : "";
    case "active":
      return `${formatSpeed(props.message.speed_bps || 0)} - ${progress.value}%`;
    case "paused":
      return "已暂停";
    case "completed":
      return "";
    case "failed":
      return "传输失败";
    case "rejected":
      return props.message.direction === "sent" ? "对方已拒绝" : "已拒绝";
    default:
      return "";
  }
});

const fileIcon = computed(() => {
  const name = (props.message.file_name || "").toLowerCase();
  if (/\.(jpg|jpeg|png|gif|bmp|webp|svg)$/.test(name)) return "🖼️";
  if (/\.(mp4|avi|mkv|mov|wmv)$/.test(name)) return "🎬";
  if (/\.(mp3|wav|flac|aac|ogg)$/.test(name)) return "🎵";
  if (/\.(zip|rar|7z|tar|gz)$/.test(name)) return "📦";
  if (/\.(pdf)$/.test(name)) return "📕";
  if (/\.(doc|docx)$/.test(name)) return "📄";
  if (/\.(xls|xlsx)$/.test(name)) return "📊";
  if (/\.(ppt|pptx)$/.test(name)) return "📽️";
  if (/\.(exe|msi|dmg|app)$/.test(name)) return "⚙️";
  return "📁";
});

// Whether we have a valid file_path to use for open/delete
const hasFilePath = computed(
  () => !!props.message.file_path && props.message.file_path.length > 0,
);

async function accept() {
  if (!props.message.transfer_id) return;
  try {
    await transferStore.acceptOffer(props.message.transfer_id);
  } catch (e) {
    console.error("Failed to accept:", e);
  }
}

async function acceptToDir() {
  if (!props.message.transfer_id || !isTauri()) return;
  const dir = await open({ directory: true, title: "选择保存位置" });
  if (!dir) return;
  try {
    await transferStore.acceptOfferToDir(
      props.message.transfer_id,
      dir as string,
    );
  } catch (e) {
    console.error("Failed to accept:", e);
  }
}

async function reject() {
  if (!props.message.transfer_id) return;
  try {
    await transferStore.rejectOffer(props.message.transfer_id);
  } catch (e) {
    console.error("Failed to reject:", e);
  }
}

function pause() {
  if (!props.message.transfer_id) return;
  transferStore.pauseTransfer(props.message.transfer_id);
}

function resume() {
  if (!props.message.transfer_id) return;
  transferStore.resumeTransfer(props.message.transfer_id);
}

function cancel() {
  if (!props.message.transfer_id) return;
  transferStore.cancelTransfer(props.message.transfer_id);
}

async function handleOpenLocation() {
  if (!props.message.file_path) return;
  try {
    await openFileLocation(props.message.file_path);
  } catch (e) {
    console.error("Failed to open location:", e);
  }
}

async function handleDelete() {
  if (!props.message.file_path || !props.message.transfer_id) return;
  try {
    await deleteFile(props.message.file_path);
    deleted.value = true;
    // Update the message status to reflect deletion
    chatStore.updateFileStatus(props.message.transfer_id, "failed");
  } catch (e) {
    console.error("Failed to delete file:", e);
  }
}
</script>

<template>
  <div :class="['file-card', message.transfer_status]">
    <div class="file-card-header">
      <span class="file-icon">{{ fileIcon }}</span>
      <div class="file-info">
        <div class="file-name">{{ message.file_name }}</div>
        <div class="file-size">{{ formatFileSize(message.file_size || 0) }}</div>
      </div>
    </div>

    <!-- Progress bar for active/paused transfers -->
    <div v-if="isActive" class="file-progress">
      <div class="progress-bar">
        <div
          class="progress-fill"
          :style="{ width: progress + '%' }"
          :class="{ paused: isPaused }"
        ></div>
      </div>
      <div class="progress-info">
        <span>{{ statusLabel }}</span>
        <span>{{ formatFileSize(message.transferred_bytes || 0) }} / {{ formatFileSize(message.file_size || 0) }}</span>
      </div>
    </div>

    <!-- Completed: show links -->
    <div v-else-if="isCompleted && !deleted" class="file-completed">
      <span class="completed-label">传输完成</span>
      <div class="completed-links">
        <a v-if="hasFilePath" class="link-action" @click="handleOpenLocation">打开目录</a>
        <span v-if="hasFilePath && message.direction === 'received'" class="link-sep">|</span>
        <a v-if="hasFilePath && message.direction === 'received'" class="link-action delete" @click="handleDelete">删除文件</a>
      </div>
    </div>

    <!-- Deleted state -->
    <div v-else-if="deleted" class="file-status deleted">
      文件已删除
    </div>

    <!-- Status label for other states -->
    <div v-else-if="statusLabel" class="file-status" :class="message.transfer_status">
      {{ statusLabel }}
    </div>

    <!-- Save location hint for pending received files -->
    <div v-if="isPending" class="save-hint">
      保存到: {{ settingsStore.downloadDir || '默认下载目录' }}
    </div>

    <!-- Pending received: accept/reject -->
    <div v-if="isPending" class="file-actions">
      <button class="btn-file-accept" @click="accept">接收</button>
      <button class="btn-file-save-as" @click="acceptToDir">另存为...</button>
      <button class="btn-file-reject" @click="reject">拒绝</button>
    </div>

    <!-- Active: pause/cancel -->
    <div v-if="isActive && !isPaused" class="file-actions">
      <button class="btn-file-action" @click="pause">暂停</button>
      <button class="btn-file-action cancel" @click="cancel">取消</button>
    </div>

    <!-- Paused: resume/cancel -->
    <div v-if="isPaused" class="file-actions">
      <button class="btn-file-action" @click="resume">继续</button>
      <button class="btn-file-action cancel" @click="cancel">取消</button>
    </div>
  </div>
</template>

<style scoped>
.file-card {
  background: rgba(0, 0, 0, 0.03);
  border-radius: 8px;
  padding: 10px 12px;
  min-width: 240px;
  max-width: 320px;
}

.file-card-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.file-icon {
  font-size: 32px;
  flex-shrink: 0;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-text);
}

.file-size {
  font-size: 11px;
  color: var(--color-text-secondary);
  margin-top: 2px;
}

.file-progress {
  margin-top: 8px;
}

.progress-bar {
  height: 4px;
  background: rgba(0, 0, 0, 0.08);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: 2px;
  transition: width 0.2s;
}

.progress-fill.paused {
  background: #ff9800;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: var(--color-text-secondary);
  margin-top: 4px;
}

.file-completed {
  margin-top: 6px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.completed-label {
  font-size: 12px;
  color: var(--color-primary);
}

.completed-links {
  display: flex;
  align-items: center;
  gap: 6px;
}

.link-action {
  font-size: 12px;
  color: #1a73e8;
  cursor: pointer;
  text-decoration: none;
  transition: color 0.15s;
}

.link-action:hover {
  color: #1557b0;
  text-decoration: underline;
}

.link-action.delete {
  color: #999;
}

.link-action.delete:hover {
  color: #f44336;
}

.link-sep {
  font-size: 11px;
  color: #ccc;
}

.file-status {
  font-size: 12px;
  margin-top: 6px;
}

.file-status.completed {
  color: var(--color-primary);
}

.file-status.failed {
  color: #f44336;
}

.file-status.rejected {
  color: #999;
}

.file-status.pending {
  color: #ff9800;
}

.file-status.deleted {
  color: #999;
  font-style: italic;
}

.save-hint {
  font-size: 11px;
  color: var(--color-text-secondary);
  margin-top: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-actions {
  display: flex;
  gap: 6px;
  margin-top: 8px;
}

.btn-file-accept {
  flex: 1;
  padding: 5px 0;
  border: none;
  border-radius: 4px;
  background: var(--color-primary);
  color: white;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s;
}

.btn-file-accept:hover {
  background: var(--color-primary-hover);
}

.btn-file-save-as {
  flex: 1;
  padding: 5px 0;
  border: 1px solid var(--color-primary);
  border-radius: 4px;
  background: transparent;
  color: var(--color-primary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-file-save-as:hover {
  background: rgba(76, 175, 80, 0.08);
}

.btn-file-reject {
  padding: 5px 10px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-file-reject:hover {
  border-color: #f44336;
  color: #f44336;
}

.btn-file-action {
  flex: 1;
  padding: 5px 0;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-file-action:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.btn-file-action.cancel:hover {
  border-color: #f44336;
  color: #f44336;
}
</style>
