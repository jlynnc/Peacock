<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import { Minus, Square, X, Users, Send } from "lucide-vue-next";
import { useRoomStore } from "@/stores/room";
import { useDeviceStore } from "@/stores/device";
import type { Room } from "@/types/room";

const props = defineProps<{
  room: Room;
}>();

const emit = defineEmits<{
  minimize: [];
  maximize: [];
  close: [];
}>();

const roomStore = useRoomStore();
const deviceStore = useDeviceStore();
const inputText = ref("");
const messageListRef = ref<HTMLDivElement>();

const messages = computed(() => roomStore.getRoomMessages(props.room.id));

const selfId = computed(() => deviceStore.selfInfo?.device_id || "");
const selfName = computed(() => deviceStore.selfInfo?.device_name || "");

const memberNames = computed(() => {
  return props.room.member_ids.map((id) => {
    if (id === selfId.value) return "我";
    const device = deviceStore.devices.get(id);
    return device?.device_name || id.slice(0, 8);
  }).join(", ");
});

function getPlatformEmoji(senderId: string): string {
  const device = deviceStore.devices.get(senderId);
  const icons: Record<string, string> = {
    windows: "💻", macos: "🖥️", linux: "🐧",
    android: "📱", ios: "📱", mcp: "🤖", web: "🌐",
  };
  return icons[device?.platform || ""] || "💻";
}

watch(messages, async () => {
  await nextTick();
  if (messageListRef.value) {
    messageListRef.value.scrollTop = messageListRef.value.scrollHeight;
  }
}, { deep: true });

async function handleSend() {
  const text = inputText.value.trim();
  if (!text) return;
  inputText.value = "";
  await roomStore.sendMessage(props.room.id, text, selfId.value, selfName.value);
}

function formatTime(ts: number): string {
  const d = new Date(ts);
  return `${d.getHours().toString().padStart(2, "0")}:${d.getMinutes().toString().padStart(2, "0")}`;
}
</script>

<template>
  <div class="room-chat">
    <!-- Title bar -->
    <div class="titlebar" data-tauri-drag-region>
      <div class="titlebar-left">
        <div class="room-icon">
          <Users :size="16" />
        </div>
        <div class="titlebar-info">
          <span class="title-name">{{ room.name }}</span>
          <span class="title-members">{{ memberNames }}</span>
        </div>
      </div>
      <div class="titlebar-right">
        <button class="win-btn" @click="emit('minimize')"><Minus :size="14" :stroke-width="1.5" /></button>
        <button class="win-btn" @click="emit('maximize')"><Square :size="12" :stroke-width="1.5" /></button>
        <button class="win-btn win-close" @click="emit('close')"><X :size="14" :stroke-width="1.5" /></button>
      </div>
    </div>

    <!-- Messages -->
    <div ref="messageListRef" class="message-list">
      <div v-if="messages.length === 0" class="no-messages">
        <p>群聊已创建，开始聊天吧</p>
      </div>
      <div
        v-for="msg in messages"
        :key="msg.message_id"
        :class="['bubble-row', msg.direction]"
      >
        <!-- Received: platform emoji avatar -->
        <div v-if="msg.direction === 'received'" class="avatar received-avatar">
          {{ getPlatformEmoji(msg.sender_id) }}
        </div>

        <div class="bubble-wrapper" :class="msg.direction">
          <div v-if="msg.direction === 'received'" class="sender-name">{{ msg.sender_name }}</div>
          <div :class="['bubble', msg.direction]">{{ msg.text }}</div>
          <div class="bubble-time">{{ formatTime(msg.timestamp) }}</div>
        </div>

        <!-- Sent: teal gradient "Me" avatar -->
        <div v-if="msg.direction === 'sent'" class="avatar sent-avatar">Me</div>
      </div>
    </div>

    <!-- Input -->
    <div class="input-bar">
      <input
        v-model="inputText"
        type="text"
        class="msg-input"
        placeholder="输入消息..."
        @keyup.enter="handleSend"
      />
      <button class="send-btn" :class="{ active: inputText.trim() }" @click="handleSend">
        <Send :size="16" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.room-chat {
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
  gap: 10px;
  -webkit-app-region: no-drag;
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
}

.titlebar-info { display: flex; flex-direction: column; }
.title-name { font-size: 14px; font-weight: 600; color: var(--color-text); }
.title-members { font-size: 11px; color: var(--color-text-muted); }

.titlebar-right { display: flex; gap: 2px; -webkit-app-region: no-drag; }
.win-btn {
  width: 28px; height: 28px; border: none; background: none;
  color: var(--color-text-muted); cursor: pointer; border-radius: 6px;
  display: flex; align-items: center; justify-content: center; transition: all 0.15s;
}
.win-btn:hover { background: var(--color-bg-input); color: var(--color-text-secondary); }
.win-close:hover { background: var(--color-danger-light); color: var(--color-danger); }

/* Messages — same style as 1-on-1 chat */
.message-list { flex: 1; overflow-y: auto; padding: 16px; }
.no-messages { text-align: center; color: var(--color-text-placeholder); padding: 40px; font-size: 13px; }

.bubble-row {
  display: flex;
  align-items: flex-start;
  margin-bottom: 12px;
}
.bubble-row.sent { justify-content: flex-end; }

.avatar {
  width: 32px; height: 32px; border-radius: 8px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0; margin-top: 2px;
}
.received-avatar {
  background: var(--color-bg-surface);
  font-size: 14px;
  margin-right: 8px;
}
.sent-avatar {
  background: linear-gradient(135deg, #0d9488, #14b8a6);
  color: #fff; font-size: 11px; font-weight: 600;
  margin-left: 8px;
}

.bubble-wrapper { max-width: 60%; display: flex; flex-direction: column; }
.bubble-wrapper.sent { align-items: flex-end; }
.bubble-wrapper.received { align-items: flex-start; }

.sender-name {
  font-size: 11px; font-weight: 500;
  color: var(--color-text-secondary);
  margin-bottom: 3px; padding-left: 4px;
}

.bubble {
  position: relative;
  padding: 10px 14px;
  border-radius: 16px;
  font-size: 13px;
  line-height: 1.5;
  word-break: break-word;
}
.bubble.sent {
  background: var(--color-primary-light);
  color: var(--color-text);
  border: 1px solid rgba(13, 148, 136, 0.15);
}
.bubble.received {
  background: var(--color-bg-input);
  color: var(--color-text);
}

.bubble-time {
  font-size: 10px;
  color: var(--color-text-placeholder);
  margin-top: 2px;
  padding: 0 4px;
}

/* Input bar */
.input-bar {
  display: flex; align-items: center;
  padding: 12px 16px; border-top: 1px solid var(--color-border); gap: 8px;
}
.msg-input {
  flex: 1; padding: 8px 14px;
  border: 1px solid var(--color-border-input); border-radius: 20px;
  font-size: 13px; background: var(--color-bg-surface);
  color: var(--color-text); outline: none;
}
.msg-input:focus { border-color: var(--color-primary); }

.send-btn {
  width: 32px; height: 32px; border: none; border-radius: 50%;
  background: var(--color-bg-input); color: var(--color-text-placeholder);
  cursor: pointer; display: flex; align-items: center; justify-content: center;
  transition: all 0.15s;
}
.send-btn.active { background: var(--color-primary); color: #fff; }
</style>
