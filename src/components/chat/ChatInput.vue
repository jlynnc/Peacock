<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { sendFile, sendFolder } from "@/utils/ipc";
import { useDeviceStore } from "@/stores/device";
import { useChatStore } from "@/stores/chat";
import { isTauri } from "@/utils/platform";

const emit = defineEmits<{
  send: [text: string];
}>();

const deviceStore = useDeviceStore();
const chatStore = useChatStore();
const inputText = ref("");
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const isSendingFile = ref(false);

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    handleSend();
  }
}

function handleSend() {
  const text = inputText.value.trim();
  if (!text) return;
  emit("send", text);
  inputText.value = "";
  textareaRef.value?.focus();
}

async function handleFilePick() {
  if (!isTauri() || !deviceStore.selectedDeviceId) return;

  const selected = await open({
    multiple: true,
    title: "选择要发送的文件",
  });

  if (!selected) return;

  const files = Array.isArray(selected) ? selected : [selected];
  isSendingFile.value = true;

  try {
    for (const filePath of files) {
      const fileName = filePath.split(/[/\\]/).pop() || filePath;
      const transferId = await sendFile(deviceStore.selectedDeviceId, filePath);
      chatStore.addFileMessage(
        deviceStore.selectedDeviceId,
        transferId,
        fileName,
        0,
        "sent",
        "pending",
      );
    }
  } catch (e) {
    console.error("Failed to send file:", e);
  } finally {
    isSendingFile.value = false;
  }
}

async function handleFolderPick() {
  if (!isTauri() || !deviceStore.selectedDeviceId) return;

  const selected = await open({
    directory: true,
    title: "选择要发送的文件夹",
  });

  if (!selected) return;

  const folderPath = selected as string;
  isSendingFile.value = true;

  try {
    const folderName = folderPath.split(/[/\\]/).pop() || folderPath;
    const transferId = await sendFolder(deviceStore.selectedDeviceId, folderPath);
    chatStore.addFileMessage(
      deviceStore.selectedDeviceId,
      transferId,
      folderName,
      0,
      "sent",
      "pending",
    );
  } catch (e) {
    console.error("Failed to send folder:", e);
  } finally {
    isSendingFile.value = false;
  }
}

function handleDrop(e: DragEvent) {
  e.preventDefault();
}

function handleDragOver(e: DragEvent) {
  e.preventDefault();
}
</script>

<template>
  <div class="chat-input-area" @drop="handleDrop" @dragover="handleDragOver">
    <div class="input-toolbar">
      <button class="toolbar-btn" title="发送文件" @click="handleFilePick" :disabled="isSendingFile">
        📎
      </button>
      <button class="toolbar-btn" title="发送文件夹" @click="handleFolderPick" :disabled="isSendingFile">
        📂
      </button>
    </div>
    <div class="input-body">
      <textarea
        ref="textareaRef"
        v-model="inputText"
        class="input-textarea"
        placeholder="输入消息，Enter 发送，Shift+Enter 换行，可拖拽文件到窗口"
        @keydown="handleKeydown"
      ></textarea>
    </div>
    <div class="input-footer">
      <button
        :class="['send-btn', { active: inputText.trim().length > 0 }]"
        @click="handleSend"
      >
        发送
      </button>
    </div>
  </div>
</template>

<style scoped>
.chat-input-area {
  border-top: 1px solid var(--color-border);
  background: #ffffff;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.input-toolbar {
  display: flex;
  gap: 4px;
  padding: 6px 12px 0;
}

.toolbar-btn {
  border: none;
  background: none;
  font-size: 18px;
  padding: 4px 6px;
  cursor: pointer;
  border-radius: 4px;
  opacity: 0.7;
  transition: all 0.15s;
}

.toolbar-btn:hover {
  opacity: 1;
  background: rgba(0, 0, 0, 0.05);
}

.toolbar-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.input-body {
  padding: 4px 12px;
}

.input-textarea {
  width: 100%;
  height: 80px;
  border: none;
  outline: none;
  resize: none;
  font-size: 14px;
  font-family: inherit;
  line-height: 1.5;
  color: var(--color-text);
  background: transparent;
}

.input-textarea::placeholder {
  color: #bbb;
}

.input-footer {
  display: flex;
  justify-content: flex-end;
  padding: 0 12px 8px;
}

.send-btn {
  padding: 6px 24px;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  background: #e0e0e0;
  color: #999;
  transition: all 0.15s;
}

.send-btn.active {
  background: var(--color-primary);
  color: white;
}

.send-btn.active:hover {
  background: var(--color-primary-hover);
}
</style>
