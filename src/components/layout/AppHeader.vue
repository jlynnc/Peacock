<script setup lang="ts">
const emit = defineEmits<{
  openSettings: [];
}>();
</script>

<template>
  <!-- Header removed: branding and settings are now in AppSidebar -->
  <div class="app-header-bridge" />
</template>

<style scoped>
.app-header-bridge {
  display: none;
}
</style>
