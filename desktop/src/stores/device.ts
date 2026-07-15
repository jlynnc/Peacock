import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DeviceInfo, SelfInfo } from "@/types/device";
import { getOnlineDevices, getSelfInfo } from "@/utils/ipc";
import { isTauri } from "@/utils/platform";

export const useDeviceStore = defineStore("device", () => {
  const devices = ref<Map<string, DeviceInfo>>(new Map());
  const selectedDeviceId = ref<string | null>(null);
  const selfInfo = ref<SelfInfo | null>(null);
  const sidebarTab = ref<"devices" | "snippets" | "rooms" | "debug">("devices");

  const onlineDevices = computed(() =>
    Array.from(devices.value.values())
      .filter((d) => d.is_online)
      .sort((a, b) => a.device_name.localeCompare(b.device_name)),
  );

  const selectedDevice = computed(() =>
    selectedDeviceId.value
      ? devices.value.get(selectedDeviceId.value) || null
      : null,
  );

  const onlineCount = computed(() => onlineDevices.value.length);

  let unlisteners: UnlistenFn[] = [];

  function selectDevice(deviceId: string | null) {
    selectedDeviceId.value = deviceId;
  }

  async function startListening() {
    if (!isTauri()) return;

    try {
      selfInfo.value = await getSelfInfo();
      const initialDevices = await getOnlineDevices();
      for (const device of initialDevices) {
        devices.value.set(device.device_id, device);
      }
    } catch (e) {
      console.error("Failed to get initial device list:", e);
    }

    const unlistenOnline = await listen<DeviceInfo>(
      "device-online",
      (event) => {
        devices.value.set(event.payload.device_id, event.payload);
      },
    );

    const unlistenOffline = await listen<{ device_id: string }>(
      "device-offline",
      (event) => {
        const device = devices.value.get(event.payload.device_id);
        if (device) {
          device.is_online = false;
          devices.value.set(event.payload.device_id, { ...device });
        }
      },
    );

    unlisteners = [unlistenOnline, unlistenOffline];
  }

  function stopListening() {
    for (const unlisten of unlisteners) {
      unlisten();
    }
    unlisteners = [];
  }

  return {
    devices,
    selectedDeviceId,
    selfInfo,
    sidebarTab,
    onlineDevices,
    selectedDevice,
    onlineCount,
    selectDevice,
    startListening,
    stopListening,
  };
});
