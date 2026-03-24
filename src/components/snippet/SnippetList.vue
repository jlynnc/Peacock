<script setup lang="ts">
import { onMounted, nextTick, watch } from "vue";
import { useSnippetStore } from "@/stores/snippet";

const store = useSnippetStore();

onMounted(() => {
  store.loadSnippets();
});

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
  const title = value.trim() || "新建片段";
  store.saveSnippet(id, { title });
  store.renamingId = null;
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
        placeholder="搜索片段..."
      />
    </div>

    <div class="list-body">
      <div
        v-for="s in store.filteredSnippets"
        :key="s.id"
        :class="['snippet-item', { active: store.selectedId === s.id }]"
        @click="store.selectedId = s.id"
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
        暂无片段
      </div>
    </div>

    <div class="list-footer">
      <button class="btn-new" @click="store.createNew()">+ 新建</button>
    </div>
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
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 13px;
  background: var(--color-bg);
  color: var(--color-text);
  outline: none;
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
  padding: 10px 10px;
  border-radius: 6px;
  cursor: pointer;
  margin-bottom: 1px;
  transition: background 0.15s;
}
.snippet-item:hover {
  background: rgba(0, 0, 0, 0.04);
}
.snippet-item.active {
  background: rgba(7, 193, 96, 0.1);
}

.snippet-title {
  font-size: 13px;
  font-weight: 500;
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
  border-radius: 3px;
  outline: none;
  background: #fff;
  color: var(--color-text);
}

.snippet-time {
  font-size: 11px;
  color: var(--color-text-secondary);
  margin-top: 2px;
}

.empty {
  text-align: center;
  color: var(--color-text-secondary);
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
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.2s;
}
.btn-new:hover {
  background: var(--color-primary-hover);
}
</style>
