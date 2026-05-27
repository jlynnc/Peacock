<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useDeviceStore } from "@/stores/device";
import { useChatStore } from "@/stores/chat";
import { useSettingsStore } from "@/stores/settings";
import { useSnippetStore } from "@/stores/snippet";
import { useRoomStore } from "@/stores/room";
import { isTauri } from "@/utils/platform";

const deviceStore = useDeviceStore();
const chatStore = useChatStore();
const settingsStore = useSettingsStore();
const snippetStore = useSnippetStore();
const roomStore = useRoomStore();

function onWindowFocus() {
  if (isTauri()) {
    invoke("stop_flash").catch(() => {});
  }
}

onMounted(() => {
  deviceStore.startListening();
  chatStore.startListening();
  snippetStore.startListening();
  roomStore.init();
  settingsStore.loadTheme();
  settingsStore.loadSettings();
  window.addEventListener("focus", onWindowFocus);
});

onUnmounted(() => {
  deviceStore.stopListening();
  chatStore.stopListening();
  snippetStore.stopListening();
  roomStore.cleanup();
  window.removeEventListener("focus", onWindowFocus);
});
</script>

<template>
  <router-view />
</template>
