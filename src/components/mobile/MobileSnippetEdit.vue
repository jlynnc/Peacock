<script setup lang="ts">
import { computed, onMounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useSnippetStore } from "@/stores/snippet";
import { ChevronLeft } from "lucide-vue-next";
import SnippetEditor from "@/components/snippet/SnippetEditor.vue";

const route = useRoute();
const router = useRouter();
const store = useSnippetStore();

const snippetId = computed(() => route.params.id as string);

onMounted(() => {
  if (snippetId.value) {
    store.selectedId = snippetId.value;
  }
});

// Navigate back when the snippet is deleted (removed from the list)
watch(
  () => store.snippets.find((s) => s.id === snippetId.value),
  (current, previous) => {
    if (previous && !current) {
      router.back();
    }
  },
);
</script>

<template>
  <div class="mobile-snippet-edit">
    <!-- Top nav bar -->
    <div class="edit-nav">
      <button class="back-btn" @click="router.back()">
        <ChevronLeft :size="28" />
      </button>
      <div class="nav-center">
        <span class="nav-title">{{ store.selectedSnippet?.title || $t('snippet.editSnippet') }}</span>
      </div>
      <div class="nav-spacer"></div>
    </div>

    <!-- Reuse desktop SnippetEditor -->
    <div class="editor-wrapper">
      <SnippetEditor />
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
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}

.nav-spacer {
  width: 44px;
  flex-shrink: 0;
}

.editor-wrapper {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.editor-wrapper :deep(.editor) {
  flex: 1;
}

.editor-wrapper :deep(.empty-editor) {
  flex: 1;
}
</style>
