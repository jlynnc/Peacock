<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { Paperclip, FolderOpen, Send } from "lucide-vue-next";
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
    <div class="input-wrapper">
      <button class="tool-btn" title="发送文件" @click="handleFilePick" :disabled="isSendingFile">
        <Paperclip :size="18" />
      </button>
      <button class="tool-btn" title="发送文件夹" @click="handleFolderPick" :disabled="isSendingFile">
        <FolderOpen :size="18" />
      </button>
      <textarea
        ref="textareaRef"
        v-model="inputText"
        class="input-textarea"
        rows="1"
        placeholder="输入消息，Enter 发送，Shift+Enter 换行"
        @keydown="handleKeydown"
      ></textarea>
      <button
        :class="['send-btn', { active: inputText.trim().length > 0 }]"
        @click="handleSend"
      >
        <Send :size="16" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.chat-input-area {
  padding: 8px 16px 12px;
  flex-shrink: 0;
}

.input-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  background: #f7f8f9;
  border: 1px solid #eee;
  border-radius: 12px;
  padding: 8px 12px;
  transition: border-color 0.15s;
}

.input-wrapper:focus-within {
  border-color: #0d9488;
}

.tool-btn {
  width: 30px;
  height: 30px;
  border: none;
  background: none;
  color: #bbb;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.15s;
}

.tool-btn:hover {
  color: #0d9488;
  background: rgba(13, 148, 136, 0.06);
}

.tool-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.input-textarea {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font-size: 13px;
  font-family: inherit;
  color: #333;
  resize: none;
  min-height: 20px;
  max-height: 80px;
  line-height: 1.5;
}

.input-textarea::placeholder {
  color: #ccc;
}

.send-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: #e0e0e0;
  color: #999;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.15s;
}

.send-btn.active {
  background: #0d9488;
  color: #fff;
}

.send-btn.active:hover {
  background: #0f766e;
}
</style>
