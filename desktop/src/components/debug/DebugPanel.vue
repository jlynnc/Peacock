<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Minus, Square, X, RefreshCw } from "lucide-vue-next";

const emit = defineEmits<{
  minimize: [];
  maximize: [];
  close: [];
}>();

const debugState = ref<any>(null);
const autoRefresh = ref(true);
let timer: ReturnType<typeof setInterval> | null = null;

async function refresh() {
  try {
    debugState.value = await invoke("get_debug_state");
  } catch (e) {
    console.error("Debug fetch failed:", e);
  }
}

async function toggleBroadcast() {
  if (!debugState.value) return;
  const newVal = !debugState.value.broadcast_enabled;
  await invoke("set_broadcast_enabled", { enabled: newVal });
  await refresh();
}

onMounted(() => {
  refresh();
  timer = setInterval(() => {
    if (autoRefresh.value) refresh();
  }, 2000);
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
});

function formatTimestamp(ts: number): string {
  if (!ts) return "never";
  const now = Math.floor(Date.now() / 1000);
  const age = now - ts;
  if (age < 0) return `${ts}`;
  if (age < 60) return `${age}s ago`;
  return `${Math.floor(age / 60)}m ago`;
}
</script>

<template>
  <div class="debug-panel">
    <!-- Title bar -->
    <div class="titlebar" data-tauri-drag-region>
      <div class="titlebar-left">
        <span class="title">🔧 Debug Panel</span>
        <button class="refresh-btn" @click="refresh"><RefreshCw :size="14" /></button>
        <label class="auto-label">
          <input type="checkbox" v-model="autoRefresh" /> Auto
        </label>
      </div>
      <div class="titlebar-right">
        <button class="win-btn" @click="emit('minimize')"><Minus :size="14" :stroke-width="1.5" /></button>
        <button class="win-btn" @click="emit('maximize')"><Square :size="12" :stroke-width="1.5" /></button>
        <button class="win-btn win-close" @click="emit('close')"><X :size="14" :stroke-width="1.5" /></button>
      </div>
    </div>

    <div class="debug-content" v-if="debugState">
      <!-- Self info -->
      <section>
        <h3>📡 Self</h3>
        <div class="kv">
          <span>Name:</span><span>{{ debugState.self.device_name }}</span>
          <span>ID:</span><span class="mono">{{ debugState.self.device_id }}</span>
          <span>IP:</span><span>{{ debugState.self.ip_addr }}</span>
          <span>Platform:</span><span>{{ debugState.self.platform }}</span>
        </div>
      </section>

      <!-- Broadcast toggle -->
      <section>
        <h3>📻 Broadcast</h3>
        <div class="toggle-row">
          <span :class="['status-dot', debugState.broadcast_enabled ? 'green' : 'red']"></span>
          <span>{{ debugState.broadcast_enabled ? 'Enabled' : 'Disabled' }}</span>
          <button class="toggle-btn" @click="toggleBroadcast">
            {{ debugState.broadcast_enabled ? 'Disable' : 'Enable' }}
          </button>
        </div>
      </section>

      <!-- Devices -->
      <section>
        <h3>🖥️ Devices ({{ debugState.devices.length }})</h3>
        <table v-if="debugState.devices.length">
          <thead>
            <tr><th>Name</th><th>Platform</th><th>IP</th><th>Last Seen</th><th>Last Broadcast</th><th>Restricted</th></tr>
          </thead>
          <tbody>
            <tr v-for="d in debugState.devices" :key="d.device_id">
              <td>{{ d.device_name }}</td>
              <td>{{ d.platform }}</td>
              <td class="mono">{{ d.ip_addr }}</td>
              <td>{{ formatTimestamp(d.last_seen) }}</td>
              <td>{{ formatTimestamp(d.last_broadcast_at) }}</td>
              <td><span :class="['dot', d.is_restricted ? 'yellow' : 'green']"></span></td>
            </tr>
          </tbody>
        </table>
        <p v-else class="empty">No devices</p>
      </section>

      <!-- Restricted Peers (in our broadcast) -->
      <section>
        <h3>🔒 Restricted Peers in Broadcast ({{ debugState.restricted_peers.length }})</h3>
        <table v-if="debugState.restricted_peers.length">
          <thead>
            <tr><th>Name</th><th>ID</th><th>IP</th><th>Platform</th></tr>
          </thead>
          <tbody>
            <tr v-for="p in debugState.restricted_peers" :key="p.device_id">
              <td>{{ p.device_name }}</td>
              <td class="mono">{{ p.device_id.slice(0, 8) }}...</td>
              <td class="mono">{{ p.ip_addr }}</td>
              <td>{{ p.platform }}</td>
            </tr>
          </tbody>
        </table>
        <p v-else class="empty">None</p>
      </section>

      <!-- Rooms -->
      <section>
        <h3>💬 Rooms ({{ debugState.rooms.length }})</h3>
        <div v-for="r in debugState.rooms" :key="r.id" class="room-debug">
          <strong>{{ r.name }}</strong>
          <span class="mono"> ({{ r.id.slice(0, 8) }}...)</span>
          <span> — {{ r.member_ids.length }} members</span>
        </div>
        <p v-if="!debugState.rooms.length" class="empty">No rooms</p>
      </section>
    </div>

    <div v-else class="loading">Loading...</div>
  </div>
</template>

<style scoped>
.debug-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-bg-surface);
}

.titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 56px;
  padding: 0 12px 0 20px;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
  -webkit-app-region: drag;
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 12px;
  -webkit-app-region: no-drag;
}

.title { font-size: 14px; font-weight: 600; color: var(--color-text); }

.refresh-btn {
  width: 28px; height: 28px; border: none; background: var(--color-bg-input);
  border-radius: 6px; cursor: pointer; display: flex; align-items: center;
  justify-content: center; color: var(--color-text-secondary);
}
.refresh-btn:hover { color: var(--color-primary); }

.auto-label {
  font-size: 12px; color: var(--color-text-muted);
  display: flex; align-items: center; gap: 4px;
}

.titlebar-right { display: flex; gap: 2px; -webkit-app-region: no-drag; }
.win-btn {
  width: 28px; height: 28px; border: none; background: none;
  color: var(--color-text-muted); cursor: pointer; border-radius: 6px;
  display: flex; align-items: center; justify-content: center;
}
.win-btn:hover { background: var(--color-bg-input); color: var(--color-text-secondary); }
.win-close:hover { background: var(--color-danger-light); color: var(--color-danger); }

.debug-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

section {
  margin-bottom: 20px;
}

h3 {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 8px;
  padding-bottom: 4px;
  border-bottom: 1px solid var(--color-border);
}

.kv {
  display: grid;
  grid-template-columns: 80px 1fr;
  gap: 4px 8px;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.mono { font-family: monospace; font-size: 11px; }

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}

th {
  text-align: left;
  padding: 4px 8px;
  font-weight: 600;
  color: var(--color-text-muted);
  border-bottom: 1px solid var(--color-border);
}

td {
  padding: 4px 8px;
  color: var(--color-text-secondary);
}

tr:hover td { background: var(--color-bg-input); }

.dot, .status-dot {
  width: 8px; height: 8px; border-radius: 50%; display: inline-block;
}
.green { background: #22c55e; }
.yellow { background: #f97316; }
.red { background: #ef4444; }

.toggle-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.toggle-btn {
  padding: 4px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-bg-input);
  color: var(--color-text-secondary);
  font-size: 12px;
  cursor: pointer;
}
.toggle-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }

.room-debug {
  font-size: 12px;
  color: var(--color-text-secondary);
  padding: 4px 0;
}

.empty {
  font-size: 12px;
  color: var(--color-text-placeholder);
  font-style: italic;
}

.loading {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-placeholder);
}
</style>
