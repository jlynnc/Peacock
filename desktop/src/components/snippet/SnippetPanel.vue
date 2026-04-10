<script setup lang="ts">
import { ref, computed } from "vue";
import { Minus, Square, X, Search } from "lucide-vue-next";
import { useSnippetStore } from "@/stores/snippet";
import SnippetEditor from "./SnippetEditor.vue";

const snippetStore = useSnippetStore();

const emit = defineEmits<{
  minimize: [];
  maximize: [];
  close: [];
}>();

const searchContentQuery = ref("");

const titleText = computed(() => {
  if (snippetStore.selectedSnippet) {
    return snippetStore.selectedSnippet.title || "Untitled";
  }
  return "Peacock";
});
</script>

<template>
  <div class="snippet-panel">
    <!-- Title bar: snippet name + search + window controls (same layout as ChatWindow) -->
    <div class="snippet-titlebar" data-tauri-drag-region>
      <div class="titlebar-left">
        <span class="snippet-titlebar-text">{{ titleText }}</span>
      </div>
      <div class="titlebar-center" data-tauri-drag-region>
        <div class="search-box">
          <Search :size="16" class="search-icon" />
          <input
            v-model="searchContentQuery"
            class="search-input"
            type="text"
            :placeholder="$t('snippet.searchPlaceholder')"
          />
        </div>
      </div>
      <div class="titlebar-right">
        <button class="win-btn" @click="emit('minimize')"><Minus :size="14" :stroke-width="1.5" /></button>
        <button class="win-btn" @click="emit('maximize')"><Square :size="12" :stroke-width="1.5" /></button>
        <button class="win-btn win-close" @click="emit('close')"><X :size="14" :stroke-width="1.5" /></button>
      </div>
    </div>
    <SnippetEditor />
  </div>
</template>

<style scoped>
.snippet-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--color-bg-surface);
}

.snippet-titlebar {
  display: flex;
  align-items: center;
  height: 56px;
  padding: 0 12px 0 20px;
  flex-shrink: 0;
  -webkit-app-region: drag;
  border-bottom: 1px solid var(--color-border);
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.snippet-titlebar-text {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 200px;
}

.titlebar-center {
  flex: 1;
  display: flex;
  justify-content: center;
  padding: 0 16px;
  -webkit-app-region: drag;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--color-bg-input);
  border: 1px solid var(--color-border-input);
  border-radius: 10px;
  padding: 6px 14px;
  width: 240px;
  -webkit-app-region: no-drag;
  transition: border-color 0.2s;
}

.search-box:focus-within {
  border-color: var(--color-primary);
}

.search-icon {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font-size: 12px;
  color: var(--color-text);
}
.search-input::placeholder {
  color: var(--color-text-placeholder);
}

.titlebar-right {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.drag-spacer {
  flex: 1;
  -webkit-app-region: drag;
}

.win-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: none;
  color: var(--color-text-muted);
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
  -webkit-app-region: no-drag;
}

.win-btn:hover {
  background: var(--color-bg-input);
  color: var(--color-text-secondary);
}

.win-close:hover {
  background: var(--color-danger-light);
  color: var(--color-danger);
}
</style>
