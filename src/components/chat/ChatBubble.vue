<script setup lang="ts">
import { ref, computed } from "vue";
import type { ChatMessage } from "@/types/message";
import { formatTime } from "@/utils/format";
import { useSnippetStore } from "@/stores/snippet";
import { useDeviceStore } from "@/stores/device";
import ChatFileCard from "./ChatFileCard.vue";
import ChatSnippetCard from "./ChatSnippetCard.vue";

const props = defineProps<{
  message: ChatMessage;
  deviceName?: string;
}>();

const avatarEmoji = computed(() => "💻");
const snippetStore = useSnippetStore();
const deviceStore = useDeviceStore();

// ── Right-click context menu (global singleton) ──
const showContextMenu = ref(false);
const menuX = ref(0);
const menuY = ref(0);

// Close any previously open context menu
function closeAllMenus() {
  document.dispatchEvent(new CustomEvent("peacock-close-context-menu"));
}

function onRightClick(e: MouseEvent) {
  if (props.message.msg_type !== "text") return;
  e.preventDefault();

  // Close any other open menu first
  closeAllMenus();

  // Select all text in the bubble
  const bubbleEl = (e.currentTarget as HTMLElement).querySelector(".bubble-content");
  if (bubbleEl) {
    const range = document.createRange();
    range.selectNodeContents(bubbleEl);
    const sel = window.getSelection();
    sel?.removeAllRanges();
    sel?.addRange(range);
  }

  menuX.value = e.clientX;
  menuY.value = e.clientY;
  showContextMenu.value = true;

  // Auto-hide on click elsewhere or another menu opening
  const hide = () => {
    showContextMenu.value = false;
    document.removeEventListener("click", hide);
    document.removeEventListener("peacock-close-context-menu", hide);
  };
  setTimeout(() => {
    document.addEventListener("click", hide);
    document.addEventListener("peacock-close-context-menu", hide);
  }, 0);
}

async function copyText() {
  showContextMenu.value = false;
  const text = props.message.content || "";
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    const ta = document.createElement("textarea");
    ta.value = text;
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    document.body.removeChild(ta);
  }
}

async function saveToSnippet() {
  showContextMenu.value = false;
  const text = props.message.content || "";
  if (!text) return;

  // Create a new snippet with the message content
  await snippetStore.createNew();
  if (snippetStore.selectedId) {
    await snippetStore.saveSnippet(snippetStore.selectedId, {
      title: text.substring(0, 30) + (text.length > 30 ? "..." : ""),
      content: text,
    });
  }

  // Switch to snippets tab
  deviceStore.sidebarTab = "snippets";
  deviceStore.selectedDeviceId = null;
}

/** Open links in default browser */
async function handleLinkClick(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (target.tagName === "A" && target.classList.contains("msg-link")) {
    e.preventDefault();
    const href = target.getAttribute("href");
    if (href) {
      try {
        const { open } = await import("@tauri-apps/plugin-shell");
        await open(href);
      } catch {
        window.open(href, "_blank");
      }
    }
  }
}

/** Convert URLs in text to clickable links */
const linkedContent = computed(() => {
  if (!props.message.content) return "";
  const urlRegex = /(https?:\/\/[^\s<>"'，。！？）】]+)/gi;
  return props.message.content.replace(urlRegex, (url: string) => {
    const escaped = url.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    return `<a href="${escaped}" target="_blank" rel="noopener" class="msg-link">${escaped}</a>`;
  });
});
</script>

<template>
  <div :class="['bubble-row', message.direction]">
    <!-- Received avatar -->
    <div v-if="message.direction === 'received'" class="avatar received-avatar">
      {{ avatarEmoji }}
    </div>

    <div class="bubble-wrapper" :class="message.direction">
      <!-- Sender name OUTSIDE bubble -->
      <div v-if="message.direction === 'received' && deviceName" class="sender-name">
        {{ deviceName }}
      </div>

      <!-- Text message bubble -->
      <div
        v-if="message.msg_type === 'text'"
        :class="['bubble', message.direction]"
        @contextmenu="onRightClick"
      >
        <div class="bubble-content" v-html="linkedContent" @click="handleLinkClick"></div>
      </div>

      <!-- File transfer card (NO bubble wrapper) -->
      <ChatFileCard v-else-if="message.msg_type === 'file'" :message="message" />

      <!-- Snippet share card (NO bubble wrapper) -->
      <ChatSnippetCard
        v-else-if="message.msg_type === 'snippet'"
        :message="message"
      />

      <div class="bubble-meta">
        <span :class="['bubble-time', message.direction]">{{ formatTime(message.timestamp) }}</span>
        <span v-if="message.direction === 'sent' && message.msg_type === 'text'" class="bubble-status">
          <span v-if="message.status === 'sending'" class="status-sending">...</span>
          <span v-else-if="message.status === 'sent'" class="status-sent"></span>
          <span v-else-if="message.status === 'failed'" class="status-failed"> {{ $t('chat.sendFailed') }}</span>
        </span>
      </div>
    </div>

    <!-- Sent avatar -->
    <div v-if="message.direction === 'sent'" class="avatar sent-avatar">
      {{ $t('chat.me') }}
    </div>

    <!-- Custom context menu -->
    <Teleport to="body">
      <div
        v-if="showContextMenu"
        class="context-menu"
        :style="{ left: menuX + 'px', top: menuY + 'px' }"
      >
        <div class="context-menu-item" @click="copyText">{{ $t('snippet.copy') || '复制' }}</div>
        <div class="context-menu-item" @click="saveToSnippet">{{ $t('chat.saveToSnippet') || '保存到片段' }}</div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.bubble-row {
  display: flex;
  width: 100%;
  align-items: flex-start;
}

.bubble-row.sent {
  justify-content: flex-end;
}

.bubble-row.received {
  justify-content: flex-start;
}

.avatar {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 2px;
}

.received-avatar {
  background: var(--color-bg-surface);
  font-size: 14px;
  margin-right: 8px;
}

.sent-avatar {
  background: linear-gradient(135deg, #0d9488, #14b8a6);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  margin-left: 8px;
}

.bubble-wrapper {
  max-width: 60%;
  display: flex;
  flex-direction: column;
}

.bubble-wrapper.sent {
  align-items: flex-end;
}

.bubble-wrapper.received {
  align-items: flex-start;
}

.sender-name {
  font-size: 11px;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-bottom: 3px;
  padding-left: 4px;
}

.bubble {
  position: relative;
  padding: 10px 14px;
  border-radius: 12px;
  word-wrap: break-word;
  white-space: pre-wrap;
  user-select: text;
  -webkit-user-select: text;
}

/* Sent bubble with arrow */
.bubble.sent {
  background: var(--color-primary-gradient);
  border: 1px solid var(--color-primary-border);
  border-top-right-radius: 4px;
  color: var(--color-text-bubble-sent);
}

.bubble.sent::after {
  content: "";
  position: absolute;
  top: 8px;
  right: -6px;
  width: 0;
  height: 0;
  border-left: 6px solid var(--color-primary-border);
  border-top: 5px solid transparent;
  border-bottom: 5px solid transparent;
}

/* Received bubble with arrow */
.bubble.received {
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border);
  border-top-left-radius: 4px;
}

.bubble.received::after {
  content: "";
  position: absolute;
  top: 8px;
  left: -6px;
  width: 0;
  height: 0;
  border-right: 6px solid var(--color-border);
  border-top: 5px solid transparent;
  border-bottom: 5px solid transparent;
}

.bubble :deep(.msg-link) {
  color: #2563eb;
  text-decoration: underline;
  cursor: pointer;
  word-break: break-all;
}
.bubble.sent :deep(.msg-link) {
  color: #93c5fd;
}

.bubble-content {
  font-size: 13px;
  line-height: 1.55;
}

.bubble-meta {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
  margin-top: 3px;
  padding-right: 2px;
}

.bubble-time {
  font-size: 10px;
  color: var(--color-text-muted);
  text-align: right;
}

.bubble-time.sent {
  color: var(--color-text-muted);
}

.bubble-status {
  font-size: 11px;
}

.status-sending {
  opacity: 0.6;
}

.status-sent {
  color: var(--color-primary);
}

.status-failed {
  color: var(--color-danger, #ef4444);
  font-size: 11px;
}

/* Context menu */
.context-menu {
  position: fixed;
  z-index: 9999;
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  box-shadow: 0 4px 16px var(--color-shadow-md);
  padding: 4px 0;
  min-width: 120px;
}

.context-menu-item {
  padding: 8px 16px;
  font-size: 13px;
  color: var(--color-text);
  cursor: pointer;
  transition: background 0.1s;
}

.context-menu-item:hover {
  background: var(--color-primary-light);
}
</style>
