<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import { Paperclip, FolderOpen } from "lucide-vue-next";
import { sendFile, sendFolder } from "@/utils/ipc";
import { useDeviceStore } from "@/stores/device";
import { useChatStore } from "@/stores/chat";
import { isTauri } from "@/utils/platform";

const { t } = useI18n();

const emit = defineEmits<{
  send: [text: string];
}>();

const deviceStore = useDeviceStore();
const chatStore = useChatStore();
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const isSendingFile = ref(false);

// Use draft from conversation store so text survives tab switches
const inputText = computed({
  get() {
    if (!deviceStore.selectedDeviceId) return "";
    const conv = chatStore.getConversation(deviceStore.selectedDeviceId);
    return conv.draft || "";
  },
  set(val: string) {
    if (!deviceStore.selectedDeviceId) return;
    const conv = chatStore.getConversation(deviceStore.selectedDeviceId);
    conv.draft = val;
  },
});

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
  // Clear draft
  if (deviceStore.selectedDeviceId) {
    const conv = chatStore.getConversation(deviceStore.selectedDeviceId);
    conv.draft = "";
  }
  textareaRef.value?.focus();
}

async function handleFilePick() {
  if (!isTauri() || !deviceStore.selectedDeviceId) return;

  const selected = await open({
    multiple: true,
    title: t('transfer.selectFile'),
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
    title: t('transfer.selectFolder'),
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
    <!-- Toolbar row -->
    <div class="toolbar-row">
      <button class="tool-btn" :title="$t('transfer.sendFile')" @click="handleFilePick" :disabled="isSendingFile">
        <Paperclip :size="16" />
      </button>
      <button class="tool-btn" :title="$t('transfer.sendFolder')" @click="handleFolderPick" :disabled="isSendingFile">
        <FolderOpen :size="16" />
      </button>
    </div>

    <!-- Input area -->
    <div class="input-area">
      <textarea
        ref="textareaRef"
        v-model="inputText"
        class="input-textarea"
        rows="4"
        :placeholder="$t('chat.inputPlaceholder')"
        @keydown="handleKeydown"
      ></textarea>
    </div>

    <!-- Send button row -->
    <div class="send-row">
      <button
        :class="['send-btn', { active: inputText.trim().length > 0 }]"
        @click="handleSend"
      >
        {{ $t('chat.sendBtn') || '发送(S)' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.chat-input-area {
  flex-shrink: 0;
  border-top: 1px solid var(--color-border);
  background: var(--color-bg-surface);
}

/* Toolbar */
.toolbar-row {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 6px 12px 0;
}

.tool-btn {
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
  flex-shrink: 0;
  transition: all 0.15s;
}

.tool-btn:hover {
  color: var(--color-primary);
  background: var(--color-primary-light);
}

.tool-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

/* Input */
.input-area {
  padding: 4px 12px;
}

.input-textarea {
  width: 100%;
  border: none;
  outline: none;
  background: transparent;
  font-size: 13px;
  font-family: inherit;
  color: var(--color-text);
  resize: none;
  line-height: 1.5;
  min-height: 72px;
  max-height: 120px;
}

.input-textarea::placeholder {
  color: var(--color-text-placeholder);
}

/* Send button */
.send-row {
  display: flex;
  justify-content: flex-end;
  padding: 0 12px 8px;
}

.send-btn {
  padding: 4px 16px;
  border: none;
  background: var(--color-border);
  color: var(--color-text-muted);
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.send-btn.active {
  background: var(--color-primary);
  color: #fff;
}

.send-btn.active:hover {
  background: var(--color-primary-hover);
}
</style>
