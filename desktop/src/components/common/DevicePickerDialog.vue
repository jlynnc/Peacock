<script setup lang="ts">
import { ref, computed } from "vue";
import { useDeviceStore } from "@/stores/device";
import { X, Search } from "lucide-vue-next";

const deviceStore = useDeviceStore();
const searchQuery = ref("");
const selectedIds = ref<Set<string>>(new Set());

const emit = defineEmits<{
  close: [];
  confirm: [deviceIds: string[]];
}>();

const filteredDevices = computed(() => {
  const q = searchQuery.value.toLowerCase();
  return deviceStore.onlineDevices.filter(
    (d) =>
      d.device_name.toLowerCase().includes(q) ||
      d.ip_addr.toLowerCase().includes(q),
  );
});

function toggleDevice(id: string) {
  if (selectedIds.value.has(id)) {
    selectedIds.value.delete(id);
  } else {
    selectedIds.value.add(id);
  }
  // Trigger reactivity
  selectedIds.value = new Set(selectedIds.value);
}

function confirmSend() {
  emit("confirm", Array.from(selectedIds.value));
}
</script>

<template>
  <Teleport to="body">
    <div class="picker-overlay" @click="emit('close')">
      <div class="picker-dialog" @click.stop>
        <!-- Header -->
        <div class="picker-header">
          <span class="picker-title">{{ $t('snippet.share') }}</span>
          <button class="picker-close" @click="emit('close')">
            <X :size="16" />
          </button>
        </div>

        <!-- Search -->
        <div class="picker-search">
          <Search :size="14" class="search-icon" />
          <input
            v-model="searchQuery"
            class="search-input"
            :placeholder="$t('search.devicePlaceholder')"
            autofocus
          />
        </div>

        <!-- Device list -->
        <div class="picker-list">
          <div
            v-for="device in filteredDevices"
            :key="device.device_id"
            :class="['picker-device', { selected: selectedIds.has(device.device_id) }]"
            @click="toggleDevice(device.device_id)"
          >
            <div class="device-checkbox">
              <div :class="['checkbox', { checked: selectedIds.has(device.device_id) }]">
                <span v-if="selectedIds.has(device.device_id)" class="check-mark">✓</span>
              </div>
            </div>
            <div class="device-info">
              <div class="device-name">{{ device.device_name }}</div>
              <div class="device-meta">{{ device.platform }} · {{ device.ip_addr }}</div>
            </div>
          </div>

          <div v-if="filteredDevices.length === 0" class="picker-empty">
            {{ $t('snippet.noOnlineDevices') }}
          </div>
        </div>

        <!-- Footer -->
        <div class="picker-footer">
          <button class="btn-cancel" @click="emit('close')">{{ $t('common.cancel') }}</button>
          <button
            class="btn-confirm"
            :disabled="selectedIds.size === 0"
            @click="confirmSend"
          >
            {{ selectedIds.size > 0
              ? `${$t('snippet.share')} (${selectedIds.size})`
              : $t('snippet.share')
            }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.picker-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
}

.picker-dialog {
  background: var(--color-bg-surface);
  border-radius: 12px;
  width: 360px;
  max-height: 480px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.15);
}

.picker-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 16px 12px;
  border-bottom: 1px solid var(--color-border);
}

.picker-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}

.picker-close {
  width: 28px;
  height: 28px;
  border: none;
  background: none;
  color: var(--color-text-muted);
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}
.picker-close:hover {
  background: var(--color-bg-input);
  color: var(--color-text);
}

.picker-search {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 12px 16px;
  padding: 8px 12px;
  background: var(--color-bg-input);
  border: 1px solid var(--color-border-input);
  border-radius: 8px;
}

.picker-search .search-icon {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.picker-search .search-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font-size: 13px;
  color: var(--color-text);
}
.picker-search .search-input::placeholder {
  color: var(--color-text-placeholder);
}

.picker-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px;
  max-height: 280px;
}

.picker-device {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 8px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
}
.picker-device:hover {
  background: var(--color-bg-input);
}
.picker-device.selected {
  background: var(--color-primary-light);
}

.device-checkbox {
  flex-shrink: 0;
}

.checkbox {
  width: 20px;
  height: 20px;
  border-radius: 6px;
  border: 2px solid var(--color-border-input);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}
.checkbox.checked {
  background: var(--color-primary);
  border-color: var(--color-primary);
}
.check-mark {
  color: #fff;
  font-size: 12px;
  font-weight: 700;
}

.device-info {
  flex: 1;
  min-width: 0;
}

.device-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.device-meta {
  font-size: 11px;
  color: var(--color-text-muted);
  margin-top: 1px;
}

.picker-empty {
  padding: 30px 0;
  text-align: center;
  font-size: 13px;
  color: var(--color-text-placeholder);
}

.picker-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--color-border);
}

.btn-cancel {
  padding: 8px 16px;
  border: none;
  border-radius: 8px;
  background: var(--color-bg-input);
  color: var(--color-text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}
.btn-cancel:hover {
  background: var(--color-border);
}

.btn-confirm {
  padding: 8px 20px;
  border: none;
  border-radius: 8px;
  background: var(--color-primary);
  color: #fff;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}
.btn-confirm:hover {
  background: var(--color-primary-hover);
}
.btn-confirm:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
