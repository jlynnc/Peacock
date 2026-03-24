<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useDeviceStore } from "@/stores/device";
import { useChatStore } from "@/stores/chat";
import { useSettingsStore } from "@/stores/settings";
import { useSnippetStore } from "@/stores/snippet";

const deviceStore = useDeviceStore();
const chatStore = useChatStore();
const settingsStore = useSettingsStore();
const snippetStore = useSnippetStore();

onMounted(() => {
  deviceStore.startListening();
  chatStore.startListening();
  snippetStore.startListening();
  settingsStore.loadSettings();
});

onUnmounted(() => {
  deviceStore.stopListening();
  chatStore.stopListening();
  snippetStore.stopListening();
});
</script>

<template>
  <router-view />
</template>
