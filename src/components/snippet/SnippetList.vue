<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useSnippetStore } from "@/stores/snippet";
import { useChatStore } from "@/stores/chat";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "@/utils/platform";
import DevicePickerDialog from "@/components/common/DevicePickerDialog.vue";

const { t } = useI18n();

const store = useSnippetStore();
const chatStore = useChatStore();

// Drag state
const dragIndex = ref<number | null>(null);
const dragOverIndex = ref<number | null>(null);

// Context menu state
const showMenu = ref(false);
const menuPos = ref({ x: 0, y: 0 });
const menuSnippetId = ref<string | null>(null);
const showDevicePicker = ref(false);
const confirmDeleteId = ref<string | null>(null);

onMounted(() => {
  store.loadSnippets();
  document.addEventListener("click", closeMenu);
});

onUnmounted(() => {
  document.removeEventListener("click", closeMenu);
});

function closeMenu() {
  showMenu.value = false;
}

// When renamingId changes, auto-focus and select the input
watch(
  () => store.renamingId,
  async (id) => {
    if (!id) return;
    await nextTick();
    const el = document.querySelector(
      `[data-rename-id="${id}"]`,
    ) as HTMLInputElement | null;
    if (el) {
      el.focus();
      el.select();
    }
  },
);

function finishRename(id: string, value: string) {
  const title = value.trim() || t('snippet.newSnippet');
  store.saveSnippet(id, { title });
  store.renamingId = null;
}

// ── Right-click menu ──

function onContextMenu(e: MouseEvent, snippetId: string) {
  e.preventDefault();
  e.stopPropagation();
  store.selectedId = snippetId;
  menuSnippetId.value = snippetId;
  menuPos.value = { x: e.clientX, y: e.clientY };
  showMenu.value = true;
}

function menuRename() {
  if (menuSnippetId.value) {
    store.renamingId = menuSnippetId.value;
  }
  showMenu.value = false;
}

async function menuCopyContent() {
  const s = store.snippets.find((s) => s.id === menuSnippetId.value);
  if (s) {
    try {
      await writeText(s.content);
    } catch {
      await navigator.clipboard.writeText(s.content);
    }
  }
  showMenu.value = false;
}

function menuShare() {
  showMenu.value = false;
  showDevicePicker.value = true;
}

async function handleShareConfirm(deviceIds: string[]) {
  const s = store.snippets.find((s) => s.id === menuSnippetId.value);
  if (s) {
    for (const deviceId of deviceIds) {
      try {
        await store.shareToDevice(deviceId, s);
        const offerId = crypto.randomUUID();
        chatStore.addSnippetMessage(deviceId, offerId, s.title, s.content, s.tag, s.note, "sent");
      } catch (e) {
        console.error("Failed to share snippet:", e);
      }
    }
  }
  showDevicePicker.value = false;
}

async function menuPinTop() {
  if (!menuSnippetId.value) return;
  showMenu.value = false;
  // Move this snippet to sort_order = -1 (before all others)
  const items = [...store.filteredSnippets];
  const idx = items.findIndex((s) => s.id === menuSnippetId.value);
  if (idx <= 0) return; // already at top
  const [moved] = items.splice(idx, 1);
  items.unshift(moved);
  const ids = items.map((s) => s.id);
  if (isTauri()) {
    try {
      await invoke("reorder_snippets", { ids });
      await store.loadSnippets();
    } catch (e) {
      console.error("Failed to pin snippet:", e);
    }
  }
}

function menuDelete() {
  confirmDeleteId.value = menuSnippetId.value;
  showMenu.value = false;
}

async function confirmDelete() {
  if (confirmDeleteId.value) {
    await store.removeSnippet(confirmDeleteId.value);
  }
  confirmDeleteId.value = null;
}

// ── Custom Mouse Drag & Drop with ghost ──

const listBodyRef = ref<HTMLElement | null>(null);
let dragStartY = 0;
const isDragging = ref(false);
let ghostEl: HTMLElement | null = null;

function createGhost(sourceEl: HTMLElement, _x: number, y: number) {
  ghostEl = sourceEl.cloneNode(true) as HTMLElement;
  ghostEl.classList.add("drag-ghost");
  ghostEl.style.position = "fixed";
  ghostEl.style.width = sourceEl.offsetWidth + "px";
  ghostEl.style.left = sourceEl.getBoundingClientRect().left + "px";
  ghostEl.style.top = y - 20 + "px";
  ghostEl.style.pointerEvents = "none";
  ghostEl.style.zIndex = "1000";
  ghostEl.style.opacity = "0.85";
  ghostEl.style.boxShadow = "0 4px 12px rgba(0,0,0,0.15)";
  ghostEl.style.borderRadius = "8px";
  ghostEl.style.background = "var(--color-bg-surface)";
  ghostEl.style.border = "1px solid var(--color-primary)";
  ghostEl.style.transition = "none";
  document.body.appendChild(ghostEl);
}

function moveGhost(y: number) {
  if (ghostEl) {
    ghostEl.style.top = y - 20 + "px";
  }
}

function removeGhost() {
  if (ghostEl) {
    ghostEl.remove();
    ghostEl = null;
  }
}

function onMouseDown(index: number, e: MouseEvent) {
  // Only left button, not on rename input
  if (e.button !== 0) return;
  if ((e.target as HTMLElement).tagName === "INPUT") return;

  dragIndex.value = index;
  dragStartY = e.clientY;
  isDragging.value = false;

  // Find the snippet-item element
  let itemEl = e.currentTarget as HTMLElement;

  const onMouseMove = (ev: MouseEvent) => {
    if (!isDragging.value && Math.abs(ev.clientY - dragStartY) > 5) {
      isDragging.value = true;
      createGhost(itemEl, ev.clientX, ev.clientY);
    }
    if (!isDragging.value) return;

    moveGhost(ev.clientY);

    // Find which item we're hovering over
    const listBody = listBodyRef.value;
    if (!listBody) return;
    const items = listBody.querySelectorAll(".snippet-item");
    let hoverIdx: number | null = null;
    for (let i = 0; i < items.length; i++) {
      const rect = items[i].getBoundingClientRect();
      const midY = rect.top + rect.height / 2;
      if (ev.clientY < midY) {
        hoverIdx = i;
        break;
      }
      hoverIdx = i + 1;
    }
    // Clamp to valid range
    if (hoverIdx !== null && hoverIdx > store.filteredSnippets.length) {
      hoverIdx = store.filteredSnippets.length;
    }
    dragOverIndex.value = hoverIdx;
  };

  const onMouseUp = async () => {
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);
    removeGhost();

    if (isDragging.value && dragIndex.value !== null && dragOverIndex.value !== null && dragIndex.value !== dragOverIndex.value) {
      const items = [...store.filteredSnippets];
      const [moved] = items.splice(dragIndex.value, 1);
      // Adjust target index if dragging down
      const targetIdx = dragOverIndex.value > dragIndex.value ? dragOverIndex.value - 1 : dragOverIndex.value;
      items.splice(targetIdx, 0, moved);

      const ids = items.map((s) => s.id);
      if (isTauri()) {
        try {
          await invoke("reorder_snippets", { ids });
          await store.loadSnippets();
        } catch (e) {
          console.error("Failed to reorder snippets:", e);
        }
      }
    }

    dragIndex.value = null;
    dragOverIndex.value = null;
    isDragging.value = false;
  };

  document.addEventListener("mousemove", onMouseMove);
  document.addEventListener("mouseup", onMouseUp);
}

// ── Touch drag for mobile (long press to start) ──
let touchTimer: ReturnType<typeof setTimeout> | null = null;
let touchStartY = 0;

function onTouchStart(index: number, e: TouchEvent) {
  if ((e.target as HTMLElement).tagName === "INPUT") return;
  const touch = e.touches[0];
  touchStartY = touch.clientY;
  const itemEl = e.currentTarget as HTMLElement;

  // Long press to initiate drag
  touchTimer = setTimeout(() => {
    dragIndex.value = index;
    isDragging.value = true;
    createGhost(itemEl, touch.clientX, touch.clientY);
  }, 400);
}

function onTouchMove(e: TouchEvent) {
  const touch = e.touches[0];

  // Cancel long-press if moved too much before drag started
  if (touchTimer && Math.abs(touch.clientY - touchStartY) > 10) {
    clearTimeout(touchTimer);
    touchTimer = null;
  }

  if (!isDragging.value) return;
  e.preventDefault();
  moveGhost(touch.clientY);

  const listBody = listBodyRef.value;
  if (!listBody) return;
  const items = listBody.querySelectorAll(".snippet-item");
  let hoverIdx: number | null = null;
  for (let i = 0; i < items.length; i++) {
    const rect = items[i].getBoundingClientRect();
    const midY = rect.top + rect.height / 2;
    if (touch.clientY < midY) {
      hoverIdx = i;
      break;
    }
    hoverIdx = i + 1;
  }
  if (hoverIdx !== null && hoverIdx > store.filteredSnippets.length) {
    hoverIdx = store.filteredSnippets.length;
  }
  dragOverIndex.value = hoverIdx;
}

async function onTouchEnd() {
  if (touchTimer) {
    clearTimeout(touchTimer);
    touchTimer = null;
  }
  removeGhost();

  if (isDragging.value && dragIndex.value !== null && dragOverIndex.value !== null && dragIndex.value !== dragOverIndex.value) {
    const items = [...store.filteredSnippets];
    const [moved] = items.splice(dragIndex.value, 1);
    const targetIdx = dragOverIndex.value > dragIndex.value ? dragOverIndex.value - 1 : dragOverIndex.value;
    items.splice(targetIdx, 0, moved);

    const ids = items.map((s) => s.id);
    if (isTauri()) {
      try {
        await invoke("reorder_snippets", { ids });
        await store.loadSnippets();
      } catch (e) {
        console.error("Failed to reorder snippets:", e);
      }
    }
  }

  dragIndex.value = null;
  dragOverIndex.value = null;
  isDragging.value = false;
}

function formatTime(ts: number) {
  const d = new Date(ts * 1000);
  const m = d.getMonth() + 1;
  const day = d.getDate();
  const h = d.getHours().toString().padStart(2, "0");
  const min = d.getMinutes().toString().padStart(2, "0");
  return `${m}/${day} ${h}:${min}`;
}
</script>

<template>
  <div class="snippet-list">
    <div class="search-area">
      <input
        class="search-input"
        v-model="store.searchQuery"
        :placeholder="$t('snippet.searchPlaceholder')"
      />
    </div>

    <div class="list-body" ref="listBodyRef">
      <div
        v-for="(s, idx) in store.filteredSnippets"
        :key="s.id"
        :class="[
          'snippet-item',
          { active: store.selectedId === s.id },
          { 'drag-over': dragOverIndex === idx && dragIndex !== idx },
          { dragging: dragIndex === idx && isDragging },
        ]"
        @mousedown="onMouseDown(idx, $event)"
        @touchstart="onTouchStart(idx, $event)"
        @touchmove="onTouchMove"
        @touchend="onTouchEnd"
        @click="store.selectedId = s.id"
        @dblclick.stop="store.renamingId = s.id"
        @contextmenu="onContextMenu($event, s.id)"
      >
        <!-- Inline rename mode -->
        <input
          v-if="store.renamingId === s.id"
          class="rename-input"
          :data-rename-id="s.id"
          :value="s.title"
          @blur="finishRename(s.id, ($event.target as HTMLInputElement).value)"
          @keydown.enter="finishRename(s.id, ($event.target as HTMLInputElement).value)"
          @keydown.escape="store.renamingId = null"
          @click.stop
        />
        <div v-else class="snippet-title">{{ s.title }}</div>
        <div class="snippet-time">{{ formatTime(s.updated_at) }}</div>
      </div>

      <div v-if="store.filteredSnippets.length === 0" class="empty">
        {{ $t('snippet.noSnippets') }}
      </div>
    </div>

    <div class="list-footer">
      <button class="btn-new" @click="store.createNew()">{{ $t('snippet.newBtn') }}</button>
    </div>

    <!-- Right-click context menu -->
    <Teleport to="body">
      <div
        v-if="showMenu"
        class="context-menu"
        :style="{ left: menuPos.x + 'px', top: menuPos.y + 'px' }"
        @click.stop
      >
        <div class="context-item" @click="menuRename">✏️ {{ $t('snippet.rename') || '重命名' }}</div>
        <div class="context-item" @click="menuCopyContent">📋 {{ $t('snippet.copyContent') }}</div>
        <div class="context-item" @click="menuShare">📤 {{ $t('snippet.share') }}</div>
        <div class="context-item" @click="menuPinTop">📌 {{ $t('snippet.pinTop') || '置顶' }}</div>
        <div class="context-sep"></div>
        <div class="context-item context-danger" @click="menuDelete">🗑️ {{ $t('common.delete') }}</div>
      </div>
    </Teleport>

    <!-- Device picker for share -->
    <DevicePickerDialog
      v-if="showDevicePicker"
      @close="showDevicePicker = false"
      @confirm="handleShareConfirm"
    />

    <!-- Delete confirm dialog -->
    <Teleport to="body">
      <template v-if="confirmDeleteId">
        <div class="overlay" @click="confirmDeleteId = null"></div>
        <div class="confirm-dialog">
          <p>{{ $t('snippet.confirmDelete', { title: store.snippets.find(s => s.id === confirmDeleteId)?.title || '' }) }}</p>
          <div class="confirm-actions">
            <button class="btn" @click="confirmDeleteId = null">{{ $t('common.cancel') }}</button>
            <button class="btn btn-danger-solid" @click="confirmDelete">{{ $t('common.delete') }}</button>
          </div>
        </div>
      </template>
    </Teleport>
  </div>
</template>

<style scoped>
.snippet-list {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.search-area {
  padding: 10px 10px 6px;
  flex-shrink: 0;
}

.search-input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid var(--color-border-input);
  border-radius: 8px;
  font-size: 12px;
  background: var(--color-bg-surface);
  color: var(--color-text);
  outline: none;
  transition: border-color 0.15s;
}
.search-input::placeholder {
  color: var(--color-text-placeholder);
}
.search-input:focus {
  border-color: var(--color-primary);
}

.list-body {
  flex: 1;
  overflow-y: auto;
  padding: 4px 6px;
}

.snippet-item {
  padding: 10px;
  border-radius: 8px;
  cursor: pointer;
  margin: 0 4px;
  transition: background 0.15s;
  border: 2px solid transparent;
}
.snippet-item:hover {
  background: var(--color-bg-input);
}
.snippet-item.active {
  background: var(--color-primary-light);
}
.snippet-item.drag-over {
  border-top: 2px solid var(--color-primary);
}
.snippet-item.dragging {
  opacity: 0.4;
}

.snippet-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.rename-input {
  width: 100%;
  font-size: 13px;
  font-weight: 500;
  padding: 2px 4px;
  border: 1px solid var(--color-primary);
  border-radius: 4px;
  outline: none;
  background: var(--color-bg-surface);
  color: var(--color-text);
}

.snippet-time {
  font-size: 11px;
  color: var(--color-text-muted);
  margin-top: 2px;
}

.empty {
  text-align: center;
  color: var(--color-text-placeholder);
  font-size: 13px;
  padding: 40px 0;
}

.list-footer {
  padding: 8px 10px;
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
}

.btn-new {
  width: 100%;
  padding: 7px;
  background: var(--color-primary);
  color: #fff;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.2s;
}
.btn-new:hover {
  background: var(--color-primary-hover);
}

/* Context menu */
.context-menu {
  position: fixed;
  z-index: 100;
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  box-shadow: 0 4px 16px var(--color-shadow-md);
  padding: 4px;
  min-width: 140px;
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
.context-danger {
  color: var(--color-danger);
}
.context-sep {
  height: 1px;
  background: var(--color-border);
  margin: 4px 8px;
}

/* Delete confirm */
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 200;
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
  z-index: 201;
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
.btn {
  padding: 4px 12px;
  border-radius: 6px;
  border: none;
  background: var(--color-bg-input);
  color: var(--color-text-secondary);
  font-size: 12px;
  cursor: pointer;
}
.btn:hover {
  background: var(--color-border);
}
.btn-danger-solid {
  background: var(--color-danger);
  color: #fff;
}
.btn-danger-solid:hover {
  background: #dc2626;
}
</style>
