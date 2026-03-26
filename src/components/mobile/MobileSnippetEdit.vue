<script setup lang="ts">
import { ref, watch, computed, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useSnippetStore } from "@/stores/snippet";
import { useDeviceStore } from "@/stores/device";
import { isTauri } from "@/utils/platform";
import { ChevronLeft, Copy, Share2, Trash2 } from "lucide-vue-next";

const route = useRoute();
const router = useRouter();
const store = useSnippetStore();
const deviceStore = useDeviceStore();

const snippetId = computed(() => route.params.id as string);
const snippet = computed(() =>
  store.snippets.find((s) => s.id === snippetId.value) || null,
);

const title = ref("");
const content = ref("");
const note = ref("");
const saveStatus = ref<"saved" | "saving" | "idle">("idle");
const confirmDelete = ref(false);
const copied = ref(false);
let saveTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  snippet,
  (s) => {
    if (s) {
      title.value = s.title;
      content.value = s.content;
      note.value = s.note;
      saveStatus.value = "idle";
    }
  },
  { immediate: true },
);

function scheduleSave() {
  if (saveTimer) clearTimeout(saveTimer);
  saveStatus.value = "saving";
  saveTimer = setTimeout(async () => {
    if (!snippetId.value) return;
    await store.saveSnippet(snippetId.value, {
      title: title.value,
      content: content.value,
      note: note.value,
    });
    saveStatus.value = "saved";
  }, 600);
}

function onTitleInput() {
  scheduleSave();
}

function onContentInput() {
  scheduleSave();
}

async function copyContent() {
  if (!content.value) return;
  try {
    if (isTauri()) {
      const { writeText } = await import("@tauri-apps/plugin-clipboard-manager");
      await writeText(content.value);
    } else {
      await navigator.clipboard.writeText(content.value);
    }
    if (snippetId.value) store.incrementCopyCount(snippetId.value);
    copied.value = true;
    setTimeout(() => (copied.value = false), 1500);
  } catch (e) {
    console.error("Failed to copy:", e);
  }
}

async function shareToDevice() {
  const s = snippet.value;
  if (!s) return;
  const devices = deviceStore.onlineDevices;
  if (devices.length === 0) return;
  // Share to first online device as a simple action on mobile
  // In a real app this could open a device picker
  await store.shareToDevice(devices[0].device_id, s);
}

async function doDelete() {
  if (!snippetId.value) return;
  await store.removeSnippet(snippetId.value);
  confirmDelete.value = false;
  router.back();
}

function goBack() {
  router.back();
}

onMounted(() => {
  if (snippetId.value) {
    store.selectedId = snippetId.value;
  }
});
</script>

<template>
  <div class="mobile-snippet-edit">
    <!-- Top nav bar -->
    <div class="edit-nav">
      <button class="back-btn" @click="goBack">
        <ChevronLeft :size="28" />
      </button>
      <div class="nav-center">
        <span class="nav-title">{{ $t('snippet.editSnippet') }}</span>
        <span class="save-indicator">
          <template v-if="saveStatus === 'saving'">{{ $t('snippet.saving') }}</template>
          <template v-else-if="saveStatus === 'saved'">{{ $t('snippet.saved') }}</template>
        </span>
      </div>
      <div class="nav-spacer"></div>
    </div>

    <div v-if="snippet" class="edit-body">
      <!-- Title -->
      <input
        v-model="title"
        class="title-input"
        :placeholder="$t('snippet.titlePlaceholder')"
        @input="onTitleInput"
      />

      <!-- Content -->
      <textarea
        v-model="content"
        class="content-input"
        :placeholder="$t('snippet.contentPlaceholder')"
        @input="onContentInput"
      ></textarea>

      <!-- Bottom toolbar -->
      <div class="bottom-toolbar">
        <button
          :class="['toolbar-btn', 'btn-copy', { 'btn-copied': copied }]"
          @click="copyContent"
        >
          <Copy :size="16" />
          <span>{{ copied ? $t('snippet.copied') : $t('snippet.copy') }}</span>
        </button>
        <button class="toolbar-btn btn-share" @click="shareToDevice">
          <Share2 :size="16" />
          <span>{{ $t('snippet.share') }}</span>
        </button>
        <button class="toolbar-btn btn-delete" @click="confirmDelete = true">
          <Trash2 :size="16" />
          <span>{{ $t('common.delete') }}</span>
        </button>
      </div>
    </div>

    <div v-else class="empty-state">
      <p>{{ $t('snippet.notFound') }}</p>
    </div>

    <!-- Delete confirm -->
    <div v-if="confirmDelete" class="overlay" @click="confirmDelete = false"></div>
    <div v-if="confirmDelete" class="confirm-dialog">
      <p class="confirm-text">{{ $t('snippet.confirmDelete', { title: snippet?.title }) }}</p>
      <div class="confirm-actions">
        <button class="confirm-btn btn-cancel" @click="confirmDelete = false">{{ $t('common.cancel') }}</button>
        <button class="confirm-btn btn-confirm-delete" @click="doDelete">{{ $t('common.delete') }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mobile-snippet-edit {
  display: flex;
  flex-direction: column;
  height: 100vh;
  height: 100dvh;
  background: var(--color-ios-card);
}

.edit-nav {
  display: flex;
  align-items: center;
  padding: 8px 4px;
  padding-top: calc(8px + env(safe-area-inset-top, 0px));
  background: var(--color-ios-card);
  border-bottom: 0.5px solid var(--color-ios-border);
  flex-shrink: 0;
  min-height: 44px;
}

.back-btn {
  width: 44px;
  height: 44px;
  border: none;
  background: none;
  color: var(--color-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
  -webkit-tap-highlight-color: transparent;
}

.nav-center {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 0;
}

.nav-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-ios-text);
}

.save-indicator {
  font-size: 12px;
  color: var(--color-ios-text-secondary);
  min-height: 16px;
}

.nav-spacer {
  width: 44px;
  flex-shrink: 0;
}

.edit-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.title-input {
  padding: 16px 16px 8px;
  font-size: 22px;
  font-weight: 700;
  border: none;
  outline: none;
  background: transparent;
  color: var(--color-ios-text);
  flex-shrink: 0;
  -webkit-appearance: none;
}

.title-input::placeholder {
  color: var(--color-ios-text-tertiary);
}

.content-input {
  flex: 1;
  padding: 8px 16px;
  font-family: "SF Mono", "Cascadia Code", "Fira Code", "Consolas", monospace;
  font-size: 15px;
  line-height: 1.7;
  border: none;
  outline: none;
  background: transparent;
  color: var(--color-ios-text);
  resize: none;
  -webkit-appearance: none;
}

.content-input::placeholder {
  color: var(--color-ios-text-tertiary);
}

.bottom-toolbar {
  display: flex;
  gap: 10px;
  padding: 12px 16px;
  padding-bottom: calc(12px + env(safe-area-inset-bottom, 0px));
  border-top: 0.5px solid var(--color-ios-border);
  flex-shrink: 0;
}

.toolbar-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 12px 0;
  border: none;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  -webkit-tap-highlight-color: transparent;
  transition: all 0.15s;
}

.btn-copy {
  background: var(--color-primary);
  color: #fff;
}

.btn-copy:active {
  background: var(--color-primary-hover);
}

.btn-copied {
  background: #059669;
}

.btn-share {
  background: var(--color-ios-bg);
  color: var(--color-ios-text-secondary);
}

.btn-share:active {
  background: var(--color-ios-input-bg);
}

.btn-delete {
  background: rgba(239, 68, 68, 0.1);
  color: var(--color-danger);
}

.btn-delete:active {
  background: #fee2e2;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-ios-text-secondary);
  font-size: 15px;
}

.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 50;
}

.confirm-dialog {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: var(--color-ios-card);
  border-radius: 14px;
  padding: 24px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  z-index: 51;
  width: calc(100% - 64px);
  max-width: 300px;
}

.confirm-text {
  font-size: 16px;
  color: var(--color-ios-text);
  text-align: center;
  margin-bottom: 20px;
}

.confirm-actions {
  display: flex;
  gap: 10px;
}

.confirm-btn {
  flex: 1;
  padding: 12px;
  border: none;
  border-radius: 10px;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  -webkit-tap-highlight-color: transparent;
}

.btn-cancel {
  background: var(--color-ios-bg);
  color: var(--color-ios-text);
}

.btn-confirm-delete {
  background: var(--color-danger);
  color: #fff;
}
</style>
