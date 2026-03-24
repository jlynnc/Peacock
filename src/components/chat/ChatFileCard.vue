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
import { Download } from "lucide-vue-next";

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
      <div class="file-icon-box">
        <span class="file-icon">{{ fileIcon }}</span>
      </div>
      <div class="file-info">
        <div class="file-name">{{ message.file_name }}</div>
        <div class="file-meta">{{ formatFileSize(message.file_size || 0) }} · {{ (message.file_name || '').split('.').pop()?.toUpperCase() || '文件' }}</div>
      </div>
      <button
        v-if="isCompleted && hasFilePath && !deleted"
        class="download-btn"
        title="打开目录"
        @click="handleOpenLocation"
      >
        <Download :size="14" />
      </button>
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
        <span class="progress-text">{{ statusLabel }}</span>
        <span class="progress-text">{{ formatFileSize(message.transferred_bytes || 0) }} / {{ formatFileSize(message.file_size || 0) }}</span>
      </div>
    </div>

    <!-- Completed: show links -->
    <div v-else-if="isCompleted && !deleted" class="file-completed">
      <span class="completed-label">传输完成</span>
      <span class="completed-sep">·</span>
      <a v-if="hasFilePath" class="link-action" @click="handleOpenLocation">打开目录</a>
      <template v-if="hasFilePath && message.direction === 'received'">
        <span class="completed-sep">·</span>
        <a class="link-action delete" @click="handleDelete">删除</a>
      </template>
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
      <button class="btn-accept" @click="accept">接收</button>
      <button class="btn-save-as" @click="acceptToDir">另存为...</button>
      <button class="btn-reject" @click="reject">拒绝</button>
    </div>

    <!-- Active: pause/cancel -->
    <div v-if="isActive && !isPaused" class="file-actions">
      <button class="btn-action" @click="pause">暂停</button>
      <button class="btn-action btn-cancel" @click="cancel">取消</button>
    </div>

    <!-- Paused: resume/cancel -->
    <div v-if="isPaused" class="file-actions">
      <button class="btn-action" @click="resume">继续</button>
      <button class="btn-action btn-cancel" @click="cancel">取消</button>
    </div>
  </div>
</template>

<style scoped>
.file-card {
  background: #fff;
  border: 1px solid #f0f0f0;
  border-radius: 10px;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 280px;
  max-width: 380px;
}

.file-card-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.file-icon-box {
  width: 42px;
  height: 42px;
  border-radius: 8px;
  background: #f0fdfa;
  color: #0d9488;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.file-icon {
  font-size: 18px;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-size: 13px;
  font-weight: 500;
  color: #0d9488;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-meta {
  font-size: 11px;
  color: #aaa;
  margin-top: 2px;
}

.download-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: none;
  background: #f0fdfa;
  color: #0d9488;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
  transition: background 0.15s;
}

.download-btn:hover {
  background: #ccfbf1;
}

.file-progress {
  margin-top: 0;
}

.progress-bar {
  height: 3px;
  background: #eee;
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #0d9488, #14b8a6);
  border-radius: 2px;
  transition: width 0.2s;
}

.progress-fill.paused {
  background: #ff9800;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-top: 4px;
}

.progress-text {
  font-size: 10px;
  color: #aaa;
}

.file-completed {
  display: flex;
  align-items: center;
  gap: 6px;
}

.completed-label {
  font-size: 12px;
  color: #0d9488;
  font-weight: 500;
}

.completed-sep {
  font-size: 11px;
  color: #ddd;
}

.link-action {
  font-size: 12px;
  color: #0d9488;
  cursor: pointer;
  text-decoration: none;
  transition: color 0.15s;
}

.link-action:hover {
  text-decoration: underline;
}

.link-action.delete {
  color: #888;
}

.link-action.delete:hover {
  color: #ef4444;
}

.file-status {
  font-size: 12px;
}

.file-status.completed {
  color: #0d9488;
}

.file-status.failed {
  color: #ef4444;
}

.file-status.rejected {
  color: #888;
}

.file-status.pending {
  color: #aaa;
}

.file-status.deleted {
  color: #888;
  font-style: italic;
}

.save-hint {
  font-size: 11px;
  color: #aaa;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-actions {
  display: flex;
  gap: 8px;
  margin-top: 2px;
}

.btn-accept {
  flex: 1;
  padding: 4px 12px;
  border: none;
  border-radius: 6px;
  background: #0d9488;
  color: #fff;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s;
}

.btn-accept:hover {
  background: #0f766e;
}

.btn-save-as {
  flex: 1;
  padding: 4px 12px;
  border: 1px solid #0d9488;
  border-radius: 6px;
  background: transparent;
  color: #0d9488;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-save-as:hover {
  background: rgba(13, 148, 136, 0.06);
}

.btn-reject {
  padding: 4px 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: #ef4444;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-reject:hover {
  text-decoration: underline;
}

.btn-action {
  flex: 1;
  padding: 4px 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: #888;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-action:hover {
  color: #0d9488;
}

.btn-action.btn-cancel:hover {
  color: #ef4444;
}
</style>
