<script setup lang="ts">
import { Minus, Square, X } from "lucide-vue-next";
import SnippetEditor from "./SnippetEditor.vue";

const emit = defineEmits<{
  minimize: [];
  maximize: [];
  close: [];
}>();
</script>

<template>
  <div class="snippet-panel">
    <!-- Title bar with window controls -->
    <div class="snippet-titlebar" data-tauri-drag-region>
      <span class="snippet-titlebar-text">片段编辑</span>
      <div class="drag-spacer" data-tauri-drag-region></div>
      <button class="win-btn" @click="emit('minimize')"><Minus :size="14" :stroke-width="1.5" /></button>
      <button class="win-btn" @click="emit('maximize')"><Square :size="12" :stroke-width="1.5" /></button>
      <button class="win-btn win-close" @click="emit('close')"><X :size="14" :stroke-width="1.5" /></button>
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
  background: #fff;
}

.snippet-titlebar {
  display: flex;
  align-items: center;
  height: 56px;
  padding: 0 12px 0 20px;
  flex-shrink: 0;
  -webkit-app-region: drag;
  border-bottom: 1px solid #f0f0f0;
}

.snippet-titlebar-text {
  font-size: 14px;
  font-weight: 600;
  color: #1a1a1a;
  -webkit-app-region: no-drag;
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
  color: #bbb;
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
  -webkit-app-region: no-drag;
}

.win-btn:hover {
  background: #f0f0f0;
  color: #666;
}

.win-close:hover {
  background: #fee2e2;
  color: #ef4444;
}
</style>
