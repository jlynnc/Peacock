<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useSnippetStore } from "@/stores/snippet";
import { useChatStore } from "@/stores/chat";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "@/utils/platform";
import { Plus, FileText, ChevronRight } from "lucide-vue-next";
import DevicePickerDialog from "@/components/common/DevicePickerDialog.vue";

const { t } = useI18n();
const router = useRouter();
const store = useSnippetStore();
const chatStore = useChatStore();
const searchQuery = ref("");

const filteredSnippets = computed(() => {
  if (!searchQuery.value) return store.snippets;
  const q = searchQuery.value.toLowerCase();
  return store.snippets.filter(
    (s) =>
      s.title.toLowerCase().includes(q) ||
      s.content.toLowerCase().includes(q) ||
      s.note.toLowerCase().includes(q),
  );
});

function formatTime(ts: number) {
  const d = new Date(ts * 1000);
  const m = d.getMonth() + 1;
  const day = d.getDate();
  const h = d.getHours().toString().padStart(2, "0");
  const min = d.getMinutes().toString().padStart(2, "0");
  return `${m}/${day} ${h}:${min}`;
}

function openSnippet(id: string) {
  store.selectedId = id;
  router.push({ name: "mobile-snippet-edit", params: { id } });
}

async function createNew() {
  await store.createNew();
  if (store.selectedId) {
    router.push({ name: "mobile-snippet-edit", params: { id: store.selectedId } });
  }
}

// ── Context menu ──
const showMenu = ref(false);
const menuPos = ref({ x: 0, y: 0 });
const menuSnippetId = ref<string | null>(null);
const showDevicePicker = ref(false);
const confirmDeleteId = ref<string | null>(null);
const renamingId = ref<string | null>(null);

function closeMenu() {
  showMenu.value = false;
}

// Auto-focus rename input
watch(renamingId, async (id) => {
  if (!id) return;
  await nextTick();
  const el = document.querySelector(`[data-rename-id="${id}"]`) as HTMLInputElement | null;
  if (el) { el.focus(); el.select(); }
});

function finishRename(id: string, value: string) {
  const title = value.trim() || t('snippet.newSnippet');
  store.saveSnippet(id, { title });
  renamingId.value = null;
}

function menuRename() {
  if (menuSnippetId.value) renamingId.value = menuSnippetId.value;
  showMenu.value = false;
}

async function menuCopyContent() {
  const s = store.snippets.find((s) => s.id === menuSnippetId.value);
  if (s) {
    try { await writeText(s.content); } catch { await navigator.clipboard.writeText(s.content); }
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
      } catch (e) { console.error("Failed to share snippet:", e); }
    }
  }
  showDevicePicker.value = false;
}

async function menuPinTop() {
  if (!menuSnippetId.value) return;
  showMenu.value = false;
  const items = [...filteredSnippets.value];
  const idx = items.findIndex((s) => s.id === menuSnippetId.value);
  if (idx <= 0) return;
  const [moved] = items.splice(idx, 1);
  items.unshift(moved);
  const ids = items.map((s) => s.id);
  if (isTauri()) {
    try { await invoke("reorder_snippets", { ids }); await store.loadSnippets(); }
    catch (e) { console.error("Failed to pin snippet:", e); }
  }
}

function menuDelete() {
  confirmDeleteId.value = menuSnippetId.value;
  showMenu.value = false;
}

async function confirmDelete() {
  if (confirmDeleteId.value) await store.removeSnippet(confirmDeleteId.value);
  confirmDeleteId.value = null;
}

// ── Touch: two-phase for iOS ──
// Phase 1 (0-500ms): movement cancels long-press, allows native scroll
// Phase 2 (after 500ms): long-press activated → drag or menu
const listRef = ref<HTMLElement | null>(null);
const dragIndex = ref<number | null>(null);
const dragOverIndex = ref<number | null>(null);
const isDragging = ref(false);
let longPressActivated = false;
let touchTimer: ReturnType<typeof setTimeout> | null = null;
let touchStartX = 0;
let touchStartY = 0;
let touchItemEl: HTMLElement | null = null;
let ghostEl: HTMLElement | null = null;

function createGhost(sourceEl: HTMLElement, y: number) {
  ghostEl = sourceEl.cloneNode(true) as HTMLElement;
  ghostEl.style.cssText = `
    position:fixed; width:${sourceEl.offsetWidth}px;
    left:${sourceEl.getBoundingClientRect().left}px; top:${y - 20}px;
    pointer-events:none; z-index:1000; opacity:0.85;
    box-shadow:0 4px 12px rgba(0,0,0,0.15); border-radius:14px;
    background:var(--color-ios-card); border:2px solid var(--color-primary);
    transition:none;
  `;
  document.body.appendChild(ghostEl);
}

function moveGhost(y: number) { if (ghostEl) ghostEl.style.top = y - 20 + "px"; }
function removeGhost() { if (ghostEl) { ghostEl.remove(); ghostEl = null; } }

// Phase 1 listener: detect early scroll movement
function handleEarlyMove(e: TouchEvent) {
  const touch = e.touches[0];
  if (Math.abs(touch.clientX - touchStartX) > 10 || Math.abs(touch.clientY - touchStartY) > 10) {
    if (touchTimer) { clearTimeout(touchTimer); touchTimer = null; }
    document.removeEventListener("touchmove", handleEarlyMove);
  }
}

// Phase 2 listener: drag reorder after long-press
function handleDragMove(e: TouchEvent) {
  if (!longPressActivated) return;
  const touch = e.touches[0];
  e.preventDefault();

  if (!isDragging.value && touchItemEl) {
    isDragging.value = true;
    createGhost(touchItemEl, touch.clientY);
  }
  if (!isDragging.value) return;
  moveGhost(touch.clientY);

  const list = listRef.value;
  if (!list) return;
  const items = list.querySelectorAll(".snippet-row");
  let hoverIdx: number | null = null;
  for (let i = 0; i < items.length; i++) {
    const rect = items[i].getBoundingClientRect();
    if (touch.clientY < rect.top + rect.height / 2) { hoverIdx = i; break; }
    hoverIdx = i + 1;
  }
  if (hoverIdx !== null && hoverIdx > filteredSnippets.value.length)
    hoverIdx = filteredSnippets.value.length;
  dragOverIndex.value = hoverIdx;
}

function onTouchStart(index: number, snippetId: string, e: TouchEvent) {
  if ((e.target as HTMLElement).tagName === "INPUT") return;
  const touch = e.touches[0];
  touchStartX = touch.clientX;
  touchStartY = touch.clientY;
  touchItemEl = e.currentTarget as HTMLElement;
  dragIndex.value = index;
  longPressActivated = false;

  // Phase 1: passive listener to detect scroll
  document.addEventListener("touchmove", handleEarlyMove, { passive: true });

  // After 500ms → long-press activated
  touchTimer = setTimeout(() => {
    touchTimer = null;
    longPressActivated = true;
    document.removeEventListener("touchmove", handleEarlyMove);
    document.addEventListener("touchmove", handleDragMove, { passive: false });
    // Visual feedback
    if (touchItemEl) touchItemEl.style.opacity = "0.5";
    store.selectedId = snippetId;
    menuSnippetId.value = snippetId;
  }, 500);
}

async function onTouchEnd() {
  document.removeEventListener("touchmove", handleEarlyMove);
  document.removeEventListener("touchmove", handleDragMove);
  if (touchTimer) { clearTimeout(touchTimer); touchTimer = null; }
  if (touchItemEl) touchItemEl.style.opacity = "";

  if (longPressActivated && !isDragging.value) {
    // Long press without drag → show context menu
    menuPos.value = { x: touchStartX, y: touchStartY };
    showMenu.value = true;
  }

  if (isDragging.value && dragIndex.value !== null && dragOverIndex.value !== null && dragIndex.value !== dragOverIndex.value) {
    const items = [...filteredSnippets.value];
    const [moved] = items.splice(dragIndex.value, 1);
    const targetIdx = dragOverIndex.value > dragIndex.value ? dragOverIndex.value - 1 : dragOverIndex.value;
    items.splice(targetIdx, 0, moved);
    const ids = items.map((s) => s.id);
    if (isTauri()) {
      try { await invoke("reorder_snippets", { ids }); await store.loadSnippets(); }
      catch (e) { console.error("Failed to reorder:", e); }
    }
  }

  removeGhost();
  dragIndex.value = null;
  dragOverIndex.value = null;
  isDragging.value = false;
  longPressActivated = false;
  touchItemEl = null;
}

onMounted(() => {
  store.loadSnippets();
});

onUnmounted(() => {
  document.removeEventListener("touchmove", handleEarlyMove);
  document.removeEventListener("touchmove", handleDragMove);
});
</script>

<template>
  <div class="mobile-snippet-list">
    <div class="page-header">
      <h1 class="page-title">{{ $t('tabs.snippets') }}</h1>
      <button class="add-btn" @click="createNew">
        <Plus :size="22" color="#fff" />
      </button>
    </div>

    <div class="search-bar">
      <input
        v-model="searchQuery"
        type="text"
        class="search-input"
        :placeholder="$t('snippet.searchPlaceholder')"
      />
    </div>

    <div v-if="filteredSnippets.length === 0" class="empty-state">
      <FileText :size="40" color="#c7c7cc" />
      <p class="empty-text">{{ $t('snippet.noSnippets') }}</p>
      <p class="empty-hint">{{ $t('snippet.newHintMobile') }}</p>
    </div>

    <div v-else class="snippet-items" ref="listRef">
      <div
        v-for="(s, idx) in filteredSnippets"
        :key="s.id"
        :class="[
          'snippet-row',
          { 'drag-over': dragOverIndex === idx && dragIndex !== idx },
          { dragging: dragIndex === idx && isDragging },
        ]"
        @touchstart="onTouchStart(idx, s.id, $event)"
        @touchend="onTouchEnd"
        @touchcancel="onTouchEnd"
        @click="openSnippet(s.id)"
      >
        <div class="snippet-icon-wrap">
          <FileText :size="20" color="#0d9488" />
        </div>
        <div class="snippet-info">
          <input
            v-if="renamingId === s.id"
            class="rename-input"
            :data-rename-id="s.id"
            :value="s.title"
            @blur="finishRename(s.id, ($event.target as HTMLInputElement).value)"
            @keydown.enter="finishRename(s.id, ($event.target as HTMLInputElement).value)"
            @keydown.escape="renamingId = null"
            @click.stop
            @touchstart.stop
          />
          <template v-else>
            <span class="snippet-title">{{ s.title }}</span>
            <span class="snippet-time">{{ formatTime(s.updated_at) }}</span>
          </template>
        </div>
        <ChevronRight :size="18" color="#c7c7cc" class="chevron" />
      </div>
    </div>

    <!-- Context menu -->
    <Teleport to="body">
      <div v-if="showMenu" class="ctx-overlay" @pointerdown="closeMenu"></div>
      <div
        v-if="showMenu"
        class="ctx-menu"
        :style="{ left: menuPos.x + 'px', top: menuPos.y + 'px' }"
        @click.stop
        @touchstart.stop
      >
        <div class="ctx-item" @pointerdown.stop="menuRename">✏️ {{ $t('snippet.rename') || '重命名' }}</div>
        <div class="ctx-item" @pointerdown.stop="menuCopyContent">📋 {{ $t('snippet.copyContent') }}</div>
        <div class="ctx-item" @pointerdown.stop="menuShare">📤 {{ $t('snippet.share') }}</div>
        <div class="ctx-item" @pointerdown.stop="menuPinTop">📌 {{ $t('snippet.pinTop') || '置顶' }}</div>
        <div class="ctx-sep"></div>
        <div class="ctx-item ctx-danger" @pointerdown.stop="menuDelete">🗑️ {{ $t('common.delete') }}</div>
      </div>
    </Teleport>

    <!-- Device picker for share -->
    <DevicePickerDialog
      v-if="showDevicePicker"
      @close="showDevicePicker = false"
      @confirm="handleShareConfirm"
    />

    <!-- Delete confirm -->
    <Teleport to="body">
      <template v-if="confirmDeleteId">
        <div class="overlay" @click="confirmDeleteId = null"></div>
        <div class="confirm-dialog">
          <p>{{ $t('snippet.confirmDelete', { title: store.snippets.find(s => s.id === confirmDeleteId)?.title || '' }) }}</p>
          <div class="confirm-actions">
            <button class="btn" @click="confirmDeleteId = null">{{ $t('common.cancel') }}</button>
            <button class="btn btn-danger" @click="confirmDelete">{{ $t('common.delete') }}</button>
          </div>
        </div>
      </template>
    </Teleport>
  </div>
</template>

<style scoped>
.mobile-snippet-list {
  min-height: 100%;
  background: var(--color-ios-bg);
}

.page-header {
  padding: 16px 16px 0;
  padding-top: calc(16px + env(safe-area-inset-top, 0px));
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.page-title {
  font-size: 30px;
  font-weight: 800;
  color: var(--color-ios-text);
  margin: 0;
  letter-spacing: -0.5px;
}

.add-btn {
  width: 36px;
  height: 36px;
  border: none;
  background: var(--color-primary);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  -webkit-tap-highlight-color: transparent;
  transition: background 0.15s;
}

.add-btn:active {
  background: var(--color-primary-hover);
}

.search-bar {
  padding: 10px 16px 6px;
}

.search-input {
  width: 100%;
  padding: 10px 14px;
  border: none;
  border-radius: 12px;
  font-size: 16px;
  background: var(--color-ios-input-bg);
  color: var(--color-ios-text);
  outline: none;
  -webkit-appearance: none;
}

.search-input::placeholder {
  color: var(--color-ios-text-secondary);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 60px 20px;
  gap: 8px;
}

.empty-text {
  font-size: 15px;
  color: var(--color-ios-text-secondary);
}

.empty-hint {
  font-size: 13px;
  color: var(--color-ios-text-tertiary);
}

.snippet-items {
  padding: 6px 16px;
}

.snippet-row {
  display: flex;
  align-items: center;
  gap: 13px;
  padding: 14px;
  background: var(--color-ios-card);
  border-radius: 14px;
  margin-bottom: 8px;
  cursor: pointer;
  -webkit-tap-highlight-color: transparent;
  -webkit-touch-callout: none;
  -webkit-user-select: none;
  user-select: none;
  transition: background 0.15s;
  border: 2px solid transparent;
}

.snippet-row:active {
  background: var(--color-ios-hover);
}

.snippet-row.drag-over {
  border-top: 2px solid var(--color-primary);
}

.snippet-row.dragging {
  opacity: 0.4;
}

.snippet-icon-wrap {
  width: 40px;
  height: 40px;
  background: rgba(13, 148, 136, 0.08);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.snippet-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.snippet-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--color-ios-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.snippet-time {
  font-size: 13px;
  color: var(--color-ios-text-secondary);
}

.chevron {
  flex-shrink: 0;
}

.rename-input {
  width: 100%;
  font-size: 16px;
  font-weight: 500;
  padding: 4px 8px;
  border: 1px solid var(--color-primary);
  border-radius: 8px;
  outline: none;
  background: var(--color-ios-input-bg);
  color: var(--color-ios-text);
  -webkit-appearance: none;
}

/* Context menu */
.ctx-overlay {
  position: fixed;
  inset: 0;
  z-index: 998;
}

.ctx-menu {
  position: fixed;
  z-index: 999;
  background: var(--color-ios-card);
  border-radius: 14px;
  box-shadow: 0 8px 30px rgba(0,0,0,0.18);
  padding: 6px;
  min-width: 160px;
}

.ctx-item {
  padding: 10px 14px;
  font-size: 15px;
  color: var(--color-ios-text);
  border-radius: 10px;
  -webkit-tap-highlight-color: transparent;
}

.ctx-item:active {
  background: var(--color-ios-hover);
}

.ctx-danger {
  color: #ef4444;
}

.ctx-sep {
  height: 1px;
  background: var(--color-ios-separator);
  margin: 4px 10px;
}

/* Delete confirm */
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.3);
  z-index: 200;
}

.confirm-dialog {
  position: fixed;
  top: 50%; left: 50%;
  transform: translate(-50%,-50%);
  background: var(--color-ios-card);
  border-radius: 14px;
  padding: 24px;
  box-shadow: 0 8px 30px rgba(0,0,0,0.18);
  z-index: 201;
  min-width: 280px;
}

.confirm-dialog p {
  font-size: 16px;
  color: var(--color-ios-text);
  margin-bottom: 18px;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.btn {
  padding: 8px 18px;
  border-radius: 10px;
  border: none;
  font-size: 15px;
  background: var(--color-ios-input-bg);
  color: var(--color-ios-text-secondary);
}

.btn-danger {
  background: #ef4444;
  color: #fff;
}
</style>
