<script setup lang="ts">
import type { ChatMessage } from "@/types/message";
import { formatTime } from "@/utils/format";
import ChatFileCard from "./ChatFileCard.vue";

defineProps<{
  message: ChatMessage;
}>();
</script>

<template>
  <div :class="['bubble-row', message.direction]">
    <div :class="['bubble', message.direction, { 'bubble-file': message.msg_type === 'file' }]">
      <!-- Text message -->
      <div v-if="message.msg_type === 'text'" class="bubble-content">{{ message.content }}</div>
      <!-- File transfer card -->
      <ChatFileCard v-else-if="message.msg_type === 'file'" :message="message" />
      <div class="bubble-meta">
        <span class="bubble-time">{{ formatTime(message.timestamp) }}</span>
        <span v-if="message.direction === 'sent' && message.msg_type === 'text'" class="bubble-status">
          <span v-if="message.status === 'sending'" class="status-sending">...</span>
          <span v-else-if="message.status === 'sent'" class="status-sent"></span>
          <span v-else-if="message.status === 'failed'" class="status-failed"> 发送失败</span>
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.bubble-row {
  display: flex;
  width: 100%;
}

.bubble-row.sent {
  justify-content: flex-end;
}

.bubble-row.received {
  justify-content: flex-start;
}

.bubble {
  max-width: 60%;
  padding: 10px 14px;
  border-radius: 8px;
  word-wrap: break-word;
  white-space: pre-wrap;
}

.bubble.sent {
  background: var(--color-bg-bubble-self);
  border-top-right-radius: 2px;
}

.bubble.received {
  background: var(--color-bg-bubble-other);
  border-top-left-radius: 2px;
}

.bubble-file {
  padding: 8px 10px;
  max-width: 380px;
}

.bubble-content {
  font-size: 14px;
  line-height: 1.5;
}

.bubble-meta {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
  margin-top: 4px;
}

.bubble-time {
  font-size: 11px;
  color: var(--color-text-secondary);
}

.bubble-status {
  font-size: 11px;
}

.status-sending {
  opacity: 0.6;
}

.status-sent {
  color: var(--color-primary);
}

.status-failed {
  color: #f44336;
}
</style>
