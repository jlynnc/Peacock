import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { updateDeviceName, updateDownloadDir, getSelfInfo } from "@/utils/ipc";
import { isTauri } from "@/utils/platform";

export const useSettingsStore = defineStore("settings", () => {
  const deviceName = ref("My Device");
  const downloadDir = ref("");
  const autoAcceptFiles = ref(false);
  const autoAcceptMaxSize = ref(10 * 1024 * 1024); // 10MB
  const clipboardSyncEnabled = ref(false);
  const theme = ref<"light" | "dark">("light");

  async function loadSettings() {
    if (!isTauri()) return;
    try {
      const info = await getSelfInfo();
      deviceName.value = info.device_name;

      const dir = await invoke<string>("get_download_dir");
      downloadDir.value = dir;
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  }

  async function setDeviceName(name: string) {
    if (!isTauri()) return;
    await updateDeviceName(name);
    deviceName.value = name;
  }

  async function setDownloadDir(path: string) {
    if (!isTauri()) return;
    await updateDownloadDir(path);
    downloadDir.value = path;
  }

  function setAutoAccept(enabled: boolean, maxSize?: number) {
    autoAcceptFiles.value = enabled;
    if (maxSize !== undefined) {
      autoAcceptMaxSize.value = maxSize;
    }
  }

  function setTheme(t: "light" | "dark") {
    theme.value = t;
    document.documentElement.setAttribute("data-theme", t);
  }

  return {
    deviceName,
    downloadDir,
    autoAcceptFiles,
    autoAcceptMaxSize,
    clipboardSyncEnabled,
    theme,
    loadSettings,
    setDeviceName,
    setDownloadDir,
    setAutoAccept,
    setTheme,
  };
});
