<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import { useSnippetStore } from "@/stores/snippet";
import { useChatStore } from "@/stores/chat";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { ClipboardList } from "lucide-vue-next";
import DevicePickerDialog from "@/components/common/DevicePickerDialog.vue";
import { isMobile } from "@/utils/platform";

const store = useSnippetStore();
const chatStore = useChatStore();

// Mobile: floating "Mark" button when text is selected
const showFloatingMark = ref(false);
const floatingMarkPos = ref({ x: 0, y: 0 });

function onSelectionChange() {
  if (!isMobile()) return;
  const sel = window.getSelection();
  if (sel && sel.toString().length > 0 && contentEditable.value?.contains(sel.anchorNode)) {
    const range = sel.getRangeAt(0);
    const rect = range.getBoundingClientRect();
    floatingMarkPos.value = { x: rect.left + rect.width / 2, y: rect.top - 40 };
    showFloatingMark.value = true;
  } else {
    showFloatingMark.value = false;
  }
}

function floatingMarkQuickCopy() {
  markAsQuickCopy();
  showFloatingMark.value = false;
}

onMounted(() => {
  document.addEventListener("selectionchange", onSelectionChange);
});

onUnmounted(() => {
  document.removeEventListener("selectionchange", onSelectionChange);
});
const contentEditable = ref<HTMLDivElement | null>(null);
const showContentMenu = ref(false);
const contentMenuPos = ref({ x: 0, y: 0 });

const title = ref("");
const content = ref("");
const note = ref("");
const copied = ref(false);
const showDevicePicker = ref(false);
const confirmDelete = ref(false);

const saveStatus = ref<"saved" | "saving" | "idle">("idle");
let saveTimer: ReturnType<typeof setTimeout> | null = null;

// Render content with [[...]] markers as inline chips
function renderContent(text: string): string {
  if (!text) return "";
  const escaped = text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
  return escaped
    .replace(/\[\[(.*?)\]\]/g, (_match, inner) => {
      return `<span class="qc-chip" contenteditable="false" data-qc="${inner.replace(/"/g, '&quot;')}">${inner}</span>`;
    })
    .replace(/\n/g, "<br>");
}

// Extract plain text with [[...]] markers back from contenteditable HTML
function extractContent(el: HTMLDivElement): string {
  let result = "";
  for (const node of el.childNodes) {
    if (node.nodeType === Node.TEXT_NODE) {
      result += node.textContent || "";
    } else if (node.nodeName === "BR") {
      result += "\n";
    } else if (node instanceof HTMLElement) {
      if (node.classList.contains("qc-chip")) {
        result += `[[${node.dataset.qc || node.textContent}]]`;
      } else {
        // Nested div (line breaks in contenteditable)
        if (node.nodeName === "DIV") {
          if (result.length > 0 && !result.endsWith("\n")) result += "\n";
          result += node.textContent || "";
        } else {
          result += node.textContent || "";
        }
      }
    }
  }
  return result;
}

let isRendering = false;

// Sync local fields when selection changes
watch(
  () => store.selectedSnippet,
  (s) => {
    if (s) {
      title.value = s.title;
      content.value = s.content;
      note.value = s.note;
      saveStatus.value = "idle";
      // Render into contenteditable
      if (contentEditable.value) {
        isRendering = true;
        contentEditable.value.innerHTML = renderContent(s.content);
        isRendering = false;
      }
    }
  },
  { immediate: true },
);

// Also render when contentEditable ref becomes available
watch(contentEditable, (el) => {
  if (el && content.value) {
    isRendering = true;
    el.innerHTML = renderContent(content.value);
    isRendering = false;
  }
});

function scheduleSave() {
  if (saveTimer) clearTimeout(saveTimer);
  saveStatus.value = "saving";
  saveTimer = setTimeout(async () => {
    if (!store.selectedId) return;
    await store.saveSnippet(store.selectedId, {
      content: content.value,
      note: note.value,
    });
    saveStatus.value = "saved";
  }, 600);
}

function onContentEditableInput() {
  if (isRendering) return;
  const el = contentEditable.value;
  if (!el) return;
  content.value = extractContent(el);
  scheduleSave();
}

// Force plain text paste — strip all formatting except our qc-chip markers
function onPaste(e: ClipboardEvent) {
  e.preventDefault();
  const text = e.clipboardData?.getData("text/plain") || "";
  // Insert plain text at cursor position
  const sel = window.getSelection();
  if (!sel || sel.rangeCount === 0) return;
  const range = sel.getRangeAt(0);
  range.deleteContents();
  const textNode = document.createTextNode(text);
  range.insertNode(textNode);
  // Move cursor to end of inserted text
  range.setStartAfter(textNode);
  range.collapse(true);
  sel.removeAllRanges();
  sel.addRange(range);
  // Trigger save
  onContentEditableInput();
}

function onNoteInput() {
  scheduleSave();
}

const contextMenuType = ref<"text" | "chip">("text");
const contextChipEl = ref<HTMLElement | null>(null);

// Handle click on chip to copy
async function onContentMouseDown(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (target.classList.contains("qc-chip")) {
    e.preventDefault();
    e.stopPropagation();
    const text = target.dataset.qc || target.textContent || "";
    try {
      await writeText(text);
    } catch {
      // Fallback to navigator.clipboard
      try {
        await navigator.clipboard.writeText(text);
      } catch (err) {
        console.error("Failed to copy chip:", err);
        return;
      }
    }
    target.classList.add("qc-copied");
    setTimeout(() => target.classList.remove("qc-copied"), 500);
  }
}

function onContentContextMenu(e: MouseEvent) {
  const target = e.target as HTMLElement;

  // Right-clicked on a chip → show chip menu (copy + unmark)
  if (target.classList.contains("qc-chip")) {
    e.preventDefault();
    contextMenuType.value = "chip";
    contextChipEl.value = target;
    contentMenuPos.value = { x: e.clientX, y: e.clientY };
    showContentMenu.value = true;
    return;
  }

  // Right-clicked on selected text → show text menu (copy + mark)
  const sel = window.getSelection();
  if (sel && sel.toString().length > 0) {
    e.preventDefault();
    contextMenuType.value = "text";
    contextChipEl.value = null;
    contentMenuPos.value = { x: e.clientX, y: e.clientY };
    showContentMenu.value = true;
  }
}

async function copySelection() {
  if (contextMenuType.value === "chip" && contextChipEl.value) {
    const text = contextChipEl.value.dataset.qc || contextChipEl.value.textContent || "";
    await writeText(text);
    contextChipEl.value.classList.add("qc-copied");
    setTimeout(() => contextChipEl.value?.classList.remove("qc-copied"), 500);
  } else {
    const sel = window.getSelection();
    const text = sel?.toString() || "";
    if (text) await writeText(text);
  }
  showContentMenu.value = false;
}

function unmarkQuickCopy() {
  const chip = contextChipEl.value;
  if (!chip || !chip.parentNode) {
    showContentMenu.value = false;
    return;
  }
  // Replace chip with plain text
  const textNode = document.createTextNode(chip.dataset.qc || chip.textContent || "");
  chip.parentNode.replaceChild(textNode, chip);

  // Update content from DOM
  const el = contentEditable.value;
  if (el) {
    content.value = extractContent(el);
    scheduleSave();
  }
  showContentMenu.value = false;
}

function markAsQuickCopy() {
  const sel = window.getSelection();
  if (!sel || sel.rangeCount === 0) {
    showContentMenu.value = false;
    return;
  }
  const selectedText = sel.toString();
  if (!selectedText) {
    showContentMenu.value = false;
    return;
  }

  // Replace selected text with a chip span
  const range = sel.getRangeAt(0);
  range.deleteContents();
  const chip = document.createElement("span");
  chip.className = "qc-chip";
  chip.contentEditable = "false";
  chip.dataset.qc = selectedText;
  chip.textContent = selectedText;
  range.insertNode(chip);

  // Update content from DOM
  const el = contentEditable.value;
  if (el) {
    content.value = extractContent(el);
    scheduleSave();
  }

  sel.removeAllRanges();
  showContentMenu.value = false;
}

async function copyContent() {
  // If user has selected text, copy only the selection
  const selection = window.getSelection();
  const selectedText = selection?.toString();

  const textToCopy = selectedText && selectedText.length > 0 ? selectedText : content.value;
  if (!textToCopy) return;

  try {
    await writeText(textToCopy);
    if (store.selectedId && !selectedText) {
      store.incrementCopyCount(store.selectedId);
    }
    copied.value = true;
    setTimeout(() => (copied.value = false), 1500);
  } catch (e) {
    console.error("Failed to copy:", e);
  }
}

async function handleShareConfirm(deviceIds: string[]) {
  const s = store.selectedSnippet;
  if (s) {
    for (const deviceId of deviceIds) {
      try {
        await store.shareToDevice(deviceId, s);
        // Add sent snippet message to chat
        const offerId = crypto.randomUUID();
        chatStore.addSnippetMessage(
          deviceId,
          offerId,
          s.title,
          s.content,
          s.tag,
          s.note,
          "sent",
        );
      } catch (e) {
        console.error("Failed to share snippet to device:", e);
      }
    }
  }
  showDevicePicker.value = false;
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
  <div class="editor" v-if="store.selectedSnippet">
    <!-- Toolbar -->
    <div class="toolbar">
      <div class="toolbar-left">
        <span class="info-text">
          {{ formatDateTime(store.selectedSnippet.updated_at) }}
          ·
          <span v-if="saveStatus === 'saving'" class="save-status saving">{{ $t('snippet.saving') }}</span>
          <span v-else-if="saveStatus === 'saved'" class="save-status saved">{{ $t('snippet.saved') }}</span>
        </span>
      </div>
      <div class="toolbar-right">
        <button
          :class="['btn', { 'btn-copied': copied }]"
          @click="copyContent"
        >
          {{ copied ? $t('snippet.copied') : $t('snippet.copyContent') }}
        </button>
        <button class="btn" @click="showDevicePicker = true">
          {{ $t('snippet.share') }}
        </button>
        <button class="btn btn-danger" @click="confirmDelete = true">
          {{ $t('common.delete') }}
        </button>
      </div>
    </div>

    <!-- Content with inline quick copy chips -->
    <div class="content-area">
      <div
        ref="contentEditable"
        class="content-input"
        contenteditable="true"
        @input="onContentEditableInput"
        @paste="onPaste"
        @mousedown="onContentMouseDown"
        @contextmenu="onContentContextMenu"
        :data-placeholder="$t('snippet.contentPlaceholder')"
      ></div>

      <!-- Right-click context menu -->
      <div
        v-if="showContentMenu"
        class="content-context-menu"
        :style="{ left: contentMenuPos.x + 'px', top: contentMenuPos.y + 'px' }"
      >
        <!-- Copy is always available -->
        <div class="context-item" @click="copySelection">📋 {{ $t('snippet.copyContent') }}</div>
        <!-- Text selected: mark as quick copy -->
        <div v-if="contextMenuType === 'text'" class="context-item" @click="markAsQuickCopy">📌 {{ $t('snippet.markQuickCopy') }}</div>
        <!-- Chip right-clicked: unmark -->
        <div v-if="contextMenuType === 'chip'" class="context-item" @click="unmarkQuickCopy">✖ {{ $t('snippet.unmarkQuickCopy') || '取消标记' }}</div>
      </div>
      <div v-if="showContentMenu" class="context-overlay" @click="showContentMenu = false"></div>

      <!-- Mobile: floating "Mark" button on text selection -->
      <div
        v-if="showFloatingMark"
        class="floating-mark-btn"
        :style="{ left: floatingMarkPos.x + 'px', top: floatingMarkPos.y + 'px' }"
        @pointerdown.stop.prevent="floatingMarkQuickCopy"
      >
        📌
      </div>
    </div>

    <!-- Note -->
    <div class="note-area">
      <input
        class="note-input"
        v-model="note"
        @input="onNoteInput"
        :placeholder="$t('snippet.notePlaceholder')"
      />
    </div>

    <!-- Device Picker Dialog for sharing -->
    <DevicePickerDialog
      v-if="showDevicePicker"
      @close="showDevicePicker = false"
      @confirm="handleShareConfirm"
    />

    <!-- Delete confirm overlay -->
    <div class="overlay" v-if="confirmDelete" @click="confirmDelete = false"></div>
    <div class="confirm-dialog" v-if="confirmDelete">
      <p>{{ $t('snippet.confirmDelete', { title: store.selectedSnippet.title }) }}</p>
      <div class="confirm-actions">
        <button class="btn" @click="confirmDelete = false">{{ $t('common.cancel') }}</button>
        <button class="btn btn-danger-solid" @click="doDelete">{{ $t('common.delete') }}</button>
      </div>
    </div>
  </div>

  <!-- Empty state -->
  <div class="empty-editor" v-else>
    <ClipboardList class="empty-icon" :size="48" />
    <p>{{ $t('snippet.selectOrCreate') }}</p>
  </div>
</template>

<style scoped>
.editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
  background: var(--color-bg-surface);
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  background: var(--color-bg-sidebar);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  align-items: center;
}

.info-text {
  font-size: 11px;
  color: var(--color-text-muted);
}
.save-status {
  font-size: 11px;
}
.save-status.saving {
  color: var(--color-text-muted);
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
  border-radius: 6px;
  border: none;
  background: var(--color-bg-input);
  color: var(--color-text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}
.btn:hover {
  background: var(--color-border);
  color: var(--color-text-secondary);
}
.btn-copied {
  background: var(--color-primary);
  color: #fff;
}
.btn-copied:hover {
  background: var(--color-primary);
  color: #fff;
}
.btn-danger:hover {
  color: var(--color-danger);
}
.btn-danger-solid {
  background: var(--color-danger);
  color: #fff;
}
.btn-danger-solid:hover {
  background: #dc2626;
  color: #fff;
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
  padding: 0;
  white-space: pre-wrap;
  word-break: break-word;
  overflow-y: auto;
}
.content-input:empty::before {
  content: attr(data-placeholder);
  color: var(--color-text-placeholder);
  pointer-events: none;
}

/* Inline quick copy chip */
.content-input :deep(.qc-chip) {
  display: inline;
  background: var(--color-primary-light);
  border: 1px solid var(--color-primary-border);
  border-radius: 4px;
  padding: 1px 6px;
  color: var(--color-primary);
  cursor: pointer;
  font-size: 13px;
  transition: all 0.15s;
  user-select: none;
}
.content-input :deep(.qc-chip:hover) {
  background: var(--color-primary);
  color: #fff;
  border-color: var(--color-primary);
}
.content-input :deep(.qc-chip.qc-copied) {
  animation: chip-flash 0.4s ease;
}

@keyframes chip-flash {
  0% { transform: scale(1); opacity: 1; }
  20% { transform: scale(0.9); opacity: 0.5; }
  50% { transform: scale(1.05); opacity: 1; background: #fff; color: var(--color-primary); }
  100% { transform: scale(1); opacity: 1; }
}

/* Content right-click context menu */
.context-overlay {
  position: fixed;
  inset: 0;
  z-index: 49;
}
.content-context-menu {
  position: fixed;
  z-index: 50;
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  box-shadow: 0 4px 16px var(--color-shadow-md);
  padding: 4px;
  min-width: 160px;
}
.context-item {
  padding: 6px 12px;
  font-size: 12px;
  color: var(--color-text);
  cursor: pointer;
  border-radius: 6px;
  transition: background 0.1s;
}
.context-item:hover {
  background: var(--color-bg-input);
}

/* Mobile floating mark button */
.floating-mark-btn {
  position: fixed;
  transform: translateX(-50%);
  background: var(--color-primary);
  color: #fff;
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 14px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.2);
  z-index: 100;
  cursor: pointer;
  -webkit-tap-highlight-color: transparent;
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
  color: var(--color-text-placeholder);
}

.empty-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-placeholder);
  background: var(--color-bg-surface);
}
.empty-icon {
  color: var(--color-text-placeholder);
  margin-bottom: 12px;
}
.empty-editor p {
  font-size: 14px;
  color: var(--color-text-placeholder);
}

.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 50;
}
.confirm-dialog {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: var(--color-bg-surface);
  border-radius: 10px;
  padding: 20px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  z-index: 51;
  min-width: 260px;
}
.confirm-dialog p {
  font-size: 14px;
  color: var(--color-text);
  margin-bottom: 14px;
}
.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
