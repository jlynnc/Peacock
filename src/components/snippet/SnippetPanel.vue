<script setup lang="ts">
import SnippetEditor from "./SnippetEditor.vue";
</script>

<template>
  <div class="snippet-panel">
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
</style>
