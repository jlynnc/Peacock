<script setup lang="ts">
import { ref } from "vue";
import { Plus, Users, Trash2 } from "lucide-vue-next";
import { useRoomStore } from "@/stores/room";
import { useDeviceStore } from "@/stores/device";

const roomStore = useRoomStore();
const deviceStore = useDeviceStore();

const showCreateDialog = ref(false);
const contextMenu = ref<{ x: number; y: number; roomId: string } | null>(null);

function onRightClick(e: MouseEvent, roomId: string) {
  e.preventDefault();
  contextMenu.value = { x: e.clientX, y: e.clientY, roomId };
}

function closeContextMenu() {
  contextMenu.value = null;
}

function contextDeleteRoom() {
  if (contextMenu.value) {
    roomStore.deleteRoomById(contextMenu.value.roomId);
  }
  contextMenu.value = null;
}
const newRoomName = ref("");
const selectedMembers = ref<string[]>([]);

function openCreateDialog() {
  newRoomName.value = "";
  selectedMembers.value = [];
  showCreateDialog.value = true;
}

function toggleMember(id: string) {
  const idx = selectedMembers.value.indexOf(id);
  if (idx >= 0) {
    selectedMembers.value.splice(idx, 1);
  } else {
    selectedMembers.value.push(id);
  }
}

async function createRoom() {
  if (!newRoomName.value.trim() || selectedMembers.value.length === 0) return;
  await roomStore.createRoom(newRoomName.value.trim(), selectedMembers.value);
  showCreateDialog.value = false;
}
</script>

<template>
  <div class="room-list">
    <div class="room-header">
      <span class="room-title">群聊</span>
      <button class="add-btn" @click="openCreateDialog" title="创建群聊">
        <Plus :size="16" />
      </button>
    </div>

    <div v-if="roomStore.rooms.length === 0" class="no-rooms">
      <p>暂无群聊</p>
    </div>

    <div
      v-for="room in roomStore.rooms"
      :key="room.id"
      :class="['room-item', { active: roomStore.selectedRoomId === room.id }]"
      @click="roomStore.selectRoom(room.id)"
      @contextmenu="onRightClick($event, room.id)"
    >
      <div class="room-icon">
        <Users :size="16" />
      </div>
      <div class="room-info">
        <div class="room-name">{{ room.name }}</div>
        <div class="room-members">{{ room.member_ids.length }} 人</div>
      </div>
    </div>
  </div>

  <!-- Context menu -->
  <Teleport to="body">
    <div v-if="contextMenu" class="context-overlay" @click="closeContextMenu" @contextmenu.prevent="closeContextMenu">
      <div class="context-menu" :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }">
        <div class="context-item danger" @click="contextDeleteRoom">
          <Trash2 :size="14" />
          <span>删除群聊</span>
        </div>
      </div>
    </div>
  </Teleport>

  <!-- Create Room Dialog -->
  <Teleport to="body">
    <div v-if="showCreateDialog" class="dialog-overlay" @click.self="showCreateDialog = false">
      <div class="dialog">
        <h3>创建群聊</h3>
        <input
          v-model="newRoomName"
          class="dialog-input"
          placeholder="群聊名称"
          @keyup.enter="createRoom"
        />
        <div class="member-list">
          <p class="member-label">选择成员：</p>
          <div
            v-for="device in deviceStore.onlineDevices"
            :key="device.device_id"
            :class="['member-item', { selected: selectedMembers.includes(device.device_id) }]"
            @click="toggleMember(device.device_id)"
          >
            <span>{{ device.device_name }}</span>
            <span class="check" v-if="selectedMembers.includes(device.device_id)">✓</span>
          </div>
          <p v-if="deviceStore.onlineDevices.length === 0" class="no-devices">没有在线设备</p>
        </div>
        <div class="dialog-actions">
          <button class="btn-cancel" @click="showCreateDialog = false">取消</button>
          <button class="btn-confirm" @click="createRoom" :disabled="!newRoomName.trim() || selectedMembers.length === 0">创建</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.room-list {
  padding: 0 8px;
}

.room-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 4px;
}

.room-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.add-btn {
  width: 24px;
  height: 24px;
  border: none;
  background: none;
  color: var(--color-text-muted);
  cursor: pointer;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.add-btn:hover {
  background: var(--color-bg-input);
  color: var(--color-primary);
}

.no-rooms {
  padding: 20px;
  text-align: center;
  color: var(--color-text-placeholder);
  font-size: 12px;
}

.room-item {
  display: flex;
  align-items: center;
  padding: 8px;
  border-radius: 8px;
  cursor: pointer;
  gap: 8px;
  transition: all 0.15s;
}

.room-item:hover {
  background: var(--color-bg-input);
}

.room-item.active {
  background: rgba(13, 148, 136, 0.06);
}

.room-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: linear-gradient(135deg, #8b5cf6, #a78bfa);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  flex-shrink: 0;
}

.room-info {
  flex: 1;
  min-width: 0;
}

.room-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.room-members {
  font-size: 11px;
  color: var(--color-text-muted);
}

/* Context menu */
.context-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
}

.context-menu {
  position: fixed;
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 4px;
  min-width: 140px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  z-index: 1001;
}

.context-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  color: var(--color-text);
  transition: background 0.1s;
}

.context-item:hover {
  background: var(--color-bg-input);
}

.context-item.danger {
  color: var(--color-danger);
}

.context-item.danger:hover {
  background: var(--color-danger-light);
}

/* Dialog */
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog {
  background: var(--color-bg-surface);
  border-radius: 12px;
  padding: 24px;
  width: 360px;
  max-height: 480px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
}

.dialog h3 {
  margin: 0 0 16px;
  font-size: 16px;
  color: var(--color-text);
}

.dialog-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 8px;
  font-size: 14px;
  background: var(--color-bg-surface);
  color: var(--color-text);
  outline: none;
  box-sizing: border-box;
}

.dialog-input:focus {
  border-color: var(--color-primary);
}

.member-list {
  margin-top: 12px;
  max-height: 240px;
  overflow-y: auto;
}

.member-label {
  font-size: 12px;
  color: var(--color-text-muted);
  margin: 0 0 8px;
}

.member-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  color: var(--color-text);
  transition: background 0.15s;
}

.member-item:hover {
  background: var(--color-bg-input);
}

.member-item.selected {
  background: rgba(13, 148, 136, 0.08);
  color: var(--color-primary);
}

.check {
  color: var(--color-primary);
  font-weight: 600;
}

.no-devices {
  font-size: 12px;
  color: var(--color-text-placeholder);
  text-align: center;
  padding: 12px;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}

.btn-cancel, .btn-confirm {
  padding: 6px 16px;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  border: none;
}

.btn-cancel {
  background: var(--color-bg-input);
  color: var(--color-text-secondary);
}

.btn-confirm {
  background: var(--color-primary);
  color: #fff;
}

.btn-confirm:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
