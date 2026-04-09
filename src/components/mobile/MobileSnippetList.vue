<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useSnippetStore } from "@/stores/snippet";
import { useChatStore } from "@/stores/chat";
import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "@/utils/platform";
import { Plus, FileText, Edit3, Share2, Pin, Trash2 } from "lucide-vue-next";
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
  if (swipedId.value) { swipedId.value = null; return; }
  if (renamingId.value) return;
  store.selectedId = id;
  router.push({ name: "mobile-snippet-edit", params: { id } });
}

// ── New snippet creation: stay on list, auto-rename ──
const renamingId = ref<string | null>(null);

async function createNew() {
  await store.createNew();
  if (store.selectedId) {
    // Don't navigate — stay on list and start renaming
    renamingId.value = store.selectedId;
  }
}

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

// ── Left-swipe actions ──
const swipedId = ref<string | null>(null);
let swipeStartX = 0;
let swipeStartY = 0;
let swipeTracking = false;
let swipeTargetId = "";

function onSwipeStart(snippetId: string, e: TouchEvent) {
  if (renamingId.value) return;
  const touch = e.touches[0];
  swipeStartX = touch.clientX;
  swipeStartY = touch.clientY;
  swipeTargetId = snippetId;
  swipeTracking = true;
}

function onSwipeMove(e: TouchEvent) {
  if (!swipeTracking) return;
  const touch = e.touches[0];
  const dx = swipeStartX - touch.clientX;
  const dy = Math.abs(touch.clientY - swipeStartY);

  // Must be mostly horizontal swipe left
  if (dx > 40 && dy < dx * 0.5) {
    swipedId.value = swipeTargetId;
    swipeTracking = false;
  } else if (dy > 20) {
    // Vertical scroll — cancel swipe
    swipeTracking = false;
  }
}

function onSwipeEnd() {
  swipeTracking = false;
}

function closeSwipe() {
  swipedId.value = null;
}

// ── Actions ──
const showDevicePicker = ref(false);
const actionSnippetId = ref<string | null>(null);
const confirmDeleteId = ref<string | null>(null);

function actionRename(id: string) {
  swipedId.value = null;
  renamingId.value = id;
}

function actionShare(id: string) {
  swipedId.value = null;
  actionSnippetId.value = id;
  requestAnimationFrame(() => {
    showDevicePicker.value = true;
  });
}

async function handleShareConfirm(deviceIds: string[]) {
  const s = store.snippets.find((s) => s.id === actionSnippetId.value);
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

async function actionPinTop(id: string) {
  swipedId.value = null;
  const items = [...filteredSnippets.value];
  const idx = items.findIndex((s) => s.id === id);
  if (idx <= 0) return;
  const [moved] = items.splice(idx, 1);
  items.unshift(moved);
  const ids = items.map((s) => s.id);
  if (isTauri()) {
    try { await invoke("reorder_snippets", { ids }); await store.loadSnippets(); }
    catch (e) { console.error("Failed to pin:", e); }
  }
}

function actionDelete(id: string) {
  swipedId.value = null;
  confirmDeleteId.value = id;
}

async function confirmDelete() {
  if (confirmDeleteId.value) await store.removeSnippet(confirmDeleteId.value);
  confirmDeleteId.value = null;
}

onMounted(() => {
  store.loadSnippets();
});
</script>

<template>
  <div class="mobile-snippet-list" @click="closeSwipe">
    <div class="page-header">
      <h1 class="page-title">{{ $t('tabs.snippets') }}</h1>
      <button class="add-btn" @click.stop="createNew">
        <Plus :size="22" color="#fff" />
      </button>
    </div>

    <div class="search-bar">
      <input
        v-model="searchQuery"
        type="text"
        class="search-input"
        :placeholder="$t('snippet.searchPlaceholder')"
        @click.stop
      />
    </div>

    <div v-if="filteredSnippets.length === 0" class="empty-state">
      <FileText :size="40" color="#c7c7cc" />
      <p class="empty-text">{{ $t('snippet.noSnippets') }}</p>
      <p class="empty-hint">{{ $t('snippet.newHintMobile') }}</p>
    </div>

    <div v-else class="snippet-items">
      <div
        v-for="s in filteredSnippets"
        :key="s.id"
        :class="['snippet-row-wrap', { swiped: swipedId === s.id }]"
      >
        <div
          class="snippet-row"
          @click.stop="openSnippet(s.id)"
          @touchstart="onSwipeStart(s.id, $event)"
          @touchmove="onSwipeMove($event)"
          @touchend="onSwipeEnd"
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
        </div>

        <!-- Swipe action buttons -->
        <div class="swipe-actions">
          <button class="swipe-btn rename" @click.stop="actionRename(s.id)">
            <Edit3 :size="16" color="#fff" />
            <span>{{ $t('snippet.rename') || '重命名' }}</span>
          </button>
          <button class="swipe-btn share" @click.stop="actionShare(s.id)">
            <Share2 :size="16" color="#fff" />
            <span>{{ $t('snippet.share') }}</span>
          </button>
          <button class="swipe-btn pin" @click.stop="actionPinTop(s.id)">
            <Pin :size="16" color="#fff" />
            <span>{{ $t('snippet.pinTop') || '置顶' }}</span>
          </button>
          <button class="swipe-btn delete" @click.stop="actionDelete(s.id)">
            <Trash2 :size="16" color="#fff" />
            <span>{{ $t('common.delete') }}</span>
          </button>
        </div>
      </div>
    </div>

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
  -webkit-tap-highlight-color: transparent;
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

/* Swipe row wrapper */
.snippet-row-wrap {
  position: relative;
  overflow: hidden;
  border-radius: 14px;
  margin-bottom: 8px;
}

.snippet-row {
  display: flex;
  align-items: center;
  gap: 13px;
  padding: 14px;
  background: var(--color-ios-card);
  border-radius: 14px;
  -webkit-tap-highlight-color: transparent;
  -webkit-touch-callout: none;
  -webkit-user-select: none;
  user-select: none;
  transition: transform 0.3s ease;
  position: relative;
  z-index: 1;
}

.snippet-row-wrap.swiped .snippet-row {
  transform: translateX(-240px);
}

.snippet-row:active {
  background: var(--color-ios-hover);
}

/* Swipe action buttons */
.swipe-actions {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  display: flex;
  align-items: stretch;
}

.swipe-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  width: 60px;
  border: none;
  color: #fff;
  font-size: 10px;
  -webkit-tap-highlight-color: transparent;
}

.swipe-btn.rename { background: #3b82f6; }
.swipe-btn.share { background: #0d9488; }
.swipe-btn.pin { background: #f59e0b; }
.swipe-btn.delete { background: #ef4444; }

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
