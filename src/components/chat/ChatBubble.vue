<script setup lang="ts">
import type { ChatMessage } from "@/types/message";
import { formatTime } from "@/utils/format";
import ChatFileCard from "./ChatFileCard.vue";

import { computed } from "vue";

defineProps<{
  message: ChatMessage;
  deviceName?: string;
}>();

const avatarEmoji = computed(() => "💻");
</script>

<template>
  <div :class="['bubble-row', message.direction]">
    <!-- Received avatar -->
    <div v-if="message.direction === 'received'" class="avatar received-avatar">
      {{ avatarEmoji }}
    </div>

    <div :class="['bubble', message.direction, { 'bubble-file': message.msg_type === 'file' }]">
      <!-- Sender name for received -->
      <div v-if="message.direction === 'received' && deviceName" class="sender-name">
        {{ deviceName }}
      </div>

      <!-- Text message -->
      <div v-if="message.msg_type === 'text'" class="bubble-content">{{ message.content }}</div>
      <!-- File transfer card -->
      <ChatFileCard v-else-if="message.msg_type === 'file'" :message="message" />

      <div class="bubble-meta">
        <span :class="['bubble-time', message.direction]">{{ formatTime(message.timestamp) }}</span>
        <span v-if="message.direction === 'sent' && message.msg_type === 'text'" class="bubble-status">
          <span v-if="message.status === 'sending'" class="status-sending">...</span>
          <span v-else-if="message.status === 'sent'" class="status-sent"></span>
          <span v-else-if="message.status === 'failed'" class="status-failed"> 发送失败</span>
        </span>
      </div>
    </div>

    <!-- Sent avatar -->
    <div v-if="message.direction === 'sent'" class="avatar sent-avatar">
      我
    </div>
  </div>
</template>

<style scoped>
.bubble-row {
  display: flex;
  width: 100%;
  align-items: flex-start;
}

.bubble-row.sent {
  justify-content: flex-end;
}

.bubble-row.received {
  justify-content: flex-start;
}

.avatar {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 2px;
}

.received-avatar {
  background: #e8f0fe;
  font-size: 14px;
  margin-right: 8px;
}

.sent-avatar {
  background: linear-gradient(135deg, #0d9488, #14b8a6);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  margin-left: 8px;
}

.bubble {
  max-width: 55%;
  padding: 10px 14px;
  border-radius: 12px;
  word-wrap: break-word;
  white-space: pre-wrap;
}

.bubble.sent {
  background: linear-gradient(135deg, #e6fffa, #ccfbf1);
  border: 1px solid #b2f5ea;
  border-top-right-radius: 4px;
  color: #134e4a;
}

.bubble.received {
  background: #fff;
  border: 1px solid #f0f0f0;
  border-top-left-radius: 4px;
}

.bubble-file {
  padding: 8px 10px;
  max-width: 380px;
}

.sender-name {
  font-size: 11px;
  font-weight: 600;
  color: #6366f1;
  margin-bottom: 3px;
}

.bubble-content {
  font-size: 13px;
  line-height: 1.55;
}

.bubble-meta {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
  margin-top: 4px;
}

.bubble-time {
  font-size: 10px;
  color: #bbb;
  text-align: right;
}

.bubble-time.sent {
  color: #5eaaa0;
}

.bubble-status {
  font-size: 11px;
}

.status-sending {
  opacity: 0.6;
}

.status-sent {
  color: #0d9488;
}

.status-failed {
  color: var(--color-danger, #ef4444);
  font-size: 11px;
}
</style>
