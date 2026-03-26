<script setup lang="ts">
import { computed } from "vue";
import type { ChatMessage } from "@/types/message";
import { useSnippetStore } from "@/stores/snippet";
import { ClipboardList, Check, X } from "lucide-vue-next";

const props = defineProps<{
  message: ChatMessage;
}>();

const snippetStore = useSnippetStore();

const status = computed(() => props.message.snippet_status || "pending");

const preview = computed(() => {
  const text = props.message.snippet_content || "";
  return text.length > 80 ? text.substring(0, 80) + "..." : text;
});

async function accept() {
  try {
    await snippetStore.createFromShare(
      props.message.snippet_title || "",
      props.message.snippet_content || "",
      props.message.snippet_tag || "",
      props.message.snippet_note || "",
    );
    props.message.snippet_status = "accepted";
  } catch (e) {
    console.error("Failed to accept snippet:", e);
  }
}

function reject() {
  props.message.snippet_status = "rejected";
}
</script>

<template>
  <div :class="['snippet-card', message.direction, status]">
    <div class="snippet-card-header">
      <ClipboardList :size="18" class="snippet-icon" />
      <div class="snippet-info">
        <div class="snippet-title">{{ message.snippet_title }}</div>
        <div class="snippet-preview">{{ preview }}</div>
      </div>
    </div>

    <!-- Pending: show accept/reject -->
    <div v-if="status === 'pending' && message.direction === 'received'" class="snippet-actions">
      <button class="btn-accept" @click="accept">
        <Check :size="14" /> {{ $t('transfer.accept') }}
      </button>
      <button class="btn-reject" @click="reject">
        <X :size="14" /> {{ $t('transfer.reject') }}
      </button>
    </div>

    <!-- Accepted -->
    <div v-else-if="status === 'accepted'" class="snippet-status accepted">
      {{ $t('snippet.saved') }}
    </div>

    <!-- Rejected -->
    <div v-else-if="status === 'rejected'" class="snippet-status rejected">
      {{ $t('transfer.rejected') }}
    </div>

    <!-- Sent -->
    <div v-else-if="message.direction === 'sent'" class="snippet-status sent">
      {{ $t('snippet.share') }}
    </div>
  </div>
</template>

<style scoped>
.snippet-card {
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  padding: 10px 12px;
  min-width: 220px;
  max-width: 340px;
}

.snippet-card-header {
  display: flex;
  gap: 8px;
  align-items: flex-start;
}

.snippet-icon {
  color: var(--color-primary);
  flex-shrink: 0;
  margin-top: 2px;
}

.snippet-info {
  flex: 1;
  min-width: 0;
}

.snippet-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.snippet-preview {
  font-size: 11px;
  color: var(--color-text-secondary);
  margin-top: 2px;
  line-height: 1.4;
  word-break: break-all;
}

.snippet-actions {
  display: flex;
  gap: 6px;
  margin-top: 8px;
}

.btn-accept, .btn-reject {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 5px 10px;
  border: none;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-accept {
  background: var(--color-primary);
  color: #fff;
}
.btn-accept:hover {
  background: var(--color-primary-hover);
}

.btn-reject {
  background: var(--color-bg-input);
  color: var(--color-text-secondary);
}
.btn-reject:hover {
  background: var(--color-border);
}

.snippet-status {
  font-size: 11px;
  margin-top: 6px;
  padding-top: 6px;
  border-top: 1px solid var(--color-border);
}

.snippet-status.accepted {
  color: var(--color-primary);
}
.snippet-status.rejected {
  color: var(--color-text-muted);
}
.snippet-status.sent {
  color: var(--color-text-secondary);
}
</style>
