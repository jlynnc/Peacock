<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useSnippetStore } from "@/stores/snippet";
import { Plus, FileText, ChevronRight } from "lucide-vue-next";

const router = useRouter();
const store = useSnippetStore();
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

onMounted(() => {
  store.loadSnippets();
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

    <div v-else class="snippet-items">
      <div
        v-for="s in filteredSnippets"
        :key="s.id"
        class="snippet-row"
        @click="openSnippet(s.id)"
      >
        <div class="snippet-icon-wrap">
          <FileText :size="20" color="#0d9488" />
        </div>
        <div class="snippet-info">
          <span class="snippet-title">{{ s.title }}</span>
          <span class="snippet-time">{{ formatTime(s.updated_at) }}</span>
        </div>
        <ChevronRight :size="18" color="#c7c7cc" class="chevron" />
      </div>
    </div>
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
  transition: background 0.15s;
}

.snippet-row:active {
  background: var(--color-ios-hover);
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
</style>
