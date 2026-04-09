<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useDeviceStore } from "@/stores/device";
import { useChatStore } from "@/stores/chat";
import { useSettingsStore } from "@/stores/settings";
import { useSnippetStore } from "@/stores/snippet";
import { isTauri } from "@/utils/platform";

const deviceStore = useDeviceStore();
const chatStore = useChatStore();
const settingsStore = useSettingsStore();
const snippetStore = useSnippetStore();

function onWindowFocus() {
  if (isTauri()) {
    invoke("stop_flash").catch(() => {});
  }
}

// Restart network when app resumes from background (iOS)
function onVisibilityChange() {
  if (document.visibilityState === "visible" && isTauri()) {
    invoke("restart_discovery").catch(() => {});
  }
}

onMounted(() => {
  deviceStore.startListening();
  chatStore.startListening();
  snippetStore.startListening();
  settingsStore.loadTheme();
  settingsStore.loadSettings();
  window.addEventListener("focus", onWindowFocus);
  document.addEventListener("visibilitychange", onVisibilityChange);
});

onUnmounted(() => {
  deviceStore.stopListening();
  chatStore.stopListening();
  snippetStore.stopListening();
  window.removeEventListener("focus", onWindowFocus);
  document.removeEventListener("visibilitychange", onVisibilityChange);
});
</script>

<template>
  <router-view />
</template>
