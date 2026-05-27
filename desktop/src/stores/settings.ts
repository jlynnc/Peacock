import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { updateDeviceName, updateDownloadDir, getSelfInfo } from "@/utils/ipc";
import { isTauri, isMobile } from "@/utils/platform";

export const useSettingsStore = defineStore("settings", () => {
  const deviceName = ref("My Device");
  const downloadDir = ref("");
  const autoAcceptFiles = ref(false);
  const autoAcceptMaxSize = ref(10 * 1024 * 1024); // 10MB
  const clipboardSyncEnabled = ref(false);
  const theme = ref<"system" | "light" | "dark">("system");
  const autoStart = ref(false);
  const contextMenu = ref(false);
  const maxConcurrent = ref(10);
  const debugMode = ref(false);

  async function loadSettings() {
    if (!isTauri()) return;
    try {
      const info = await getSelfInfo();
      deviceName.value = info.device_name;

      const dir = await invoke<string>("get_download_dir");
      downloadDir.value = dir;

      // Load autostart and context menu state (desktop only)
      if (!isMobile()) {
        try {
          const { isEnabled } = await import("@tauri-apps/plugin-autostart");
          autoStart.value = await isEnabled();
        } catch (e) {
          console.warn("Autostart plugin not available:", e);
        }
        try {
          contextMenu.value = await invoke<boolean>("is_context_menu_registered");
        } catch (e) {
          console.warn("Context menu check not available:", e);
        }
      }
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

  function setTheme(t: "system" | "light" | "dark") {
    theme.value = t;
    document.documentElement.setAttribute("data-theme", t);
    localStorage.setItem("peacock-theme", t);
  }

  function loadTheme() {
    const saved = localStorage.getItem("peacock-theme") as "system" | "light" | "dark" | null;
    if (saved) {
      theme.value = saved;
    }
    document.documentElement.setAttribute("data-theme", theme.value);
  }

  async function setMaxConcurrent(n: number) {
    maxConcurrent.value = n;
    if (isTauri()) {
      try {
        await invoke("set_max_concurrent", { max: n });
      } catch (e) {
        console.warn("Failed to set max concurrent:", e);
      }
    }
  }

  async function setContextMenu(enabled: boolean) {
    if (!isTauri() || isMobile()) return;
    try {
      if (enabled) {
        await invoke("register_context_menu");
      } else {
        await invoke("unregister_context_menu");
      }
      contextMenu.value = enabled;
    } catch (e) {
      console.error("Failed to set context menu:", e);
    }
  }

  async function setAutoStart(enabled: boolean) {
    if (!isTauri() || isMobile()) return;
    try {
      const { enable, disable } = await import("@tauri-apps/plugin-autostart");
      if (enabled) {
        await enable();
      } else {
        await disable();
      }
      autoStart.value = enabled;
    } catch (e) {
      console.error("Failed to set autostart:", e);
    }
  }

  return {
    deviceName,
    downloadDir,
    autoAcceptFiles,
    autoAcceptMaxSize,
    clipboardSyncEnabled,
    theme,
    autoStart,
    contextMenu,
    maxConcurrent,
    debugMode,
    loadSettings,
    setDeviceName,
    setDownloadDir,
    setAutoAccept,
    setTheme,
    loadTheme,
    setMaxConcurrent,
    setContextMenu,
    setAutoStart,
  };
});
