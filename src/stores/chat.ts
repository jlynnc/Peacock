import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type {
  ChatMessage,
  Conversation,
  TransferStatus,
} from "@/types/message";
import {
  sendMessage as ipcSendMessage,
  getMessageHistory,
} from "@/utils/ipc";
import { isTauri } from "@/utils/platform";
import { i18n } from "@/i18n";
import type { FileOffer, TransferTask, TransferProgress } from "@/types/transfer";

export const useChatStore = defineStore("chat", () => {
  const conversations = ref<Map<string, Conversation>>(new Map());
  const totalUnread = computed(() => {
    let count = 0;
    for (const conv of conversations.value.values()) {
      count += conv.unread_count;
    }
    return count;
  });

  let unlisteners: UnlistenFn[] = [];

  function getConversation(deviceId: string): Conversation {
    let conv = conversations.value.get(deviceId);
    if (!conv) {
      conv = { device_id: deviceId, messages: [], unread_count: 0 };
      conversations.value.set(deviceId, conv);
    }
    return conv;
  }

  function getMessages(deviceId: string): ChatMessage[] {
    return getConversation(deviceId).messages;
  }

  function getUnreadCount(deviceId: string): number {
    return conversations.value.get(deviceId)?.unread_count || 0;
  }

  function markAsRead(deviceId: string) {
    const conv = conversations.value.get(deviceId);
    if (conv) {
      conv.unread_count = 0;
    }
  }

  async function sendMessage(deviceId: string, text: string) {
    const conv = getConversation(deviceId);
    const tempMsg: ChatMessage = {
      id: `temp-${Date.now()}`,
      device_id: deviceId,
      direction: "sent",
      content: text,
      msg_type: "text",
      timestamp: Date.now(),
      status: "sending",
    };
    conv.messages.push(tempMsg);
    conv.last_message = tempMsg;

    try {
      // Debug commands
      if (text === "/debug") {
        const { getSelfInfo } = await import("@/utils/ipc");
        const info = await getSelfInfo();
        tempMsg.content = `[DEBUG]\nIP: ${info.ip_addr}\nPort: ${info.tcp_port}\nPlatform: ${info.platform}\nDevice: ${info.device_name}\nID: ${info.device_id}`;
        tempMsg.status = "sent";
        return;
      }
      if (text.startsWith("/udptest")) {
        const { invoke } = await import("@tauri-apps/api/core");
        const parts = text.split(" ");
        const ip = parts[1] || "192.168.31.30";
        try {
          const result = await invoke("udp_test", { targetIp: ip });
          tempMsg.content = `[UDP TEST] ${result}`;
          tempMsg.status = "sent";
        } catch (e) {
          tempMsg.content = `[UDP TEST ERROR] ${e}`;
          tempMsg.status = "failed";
        }
        return;
      }
      const msgId = await ipcSendMessage(deviceId, text);
      tempMsg.id = msgId;
      tempMsg.status = "sent";
    } catch (e) {
      tempMsg.status = "failed";
      tempMsg.content = `${text}\n[ERROR: ${e}]`;
      console.error("Failed to send message:", e);
    }
  }

  /** Add a file transfer message to the chat (called when sending or receiving a file) */
  function addFileMessage(
    deviceId: string,
    transferId: string,
    fileName: string,
    fileSize: number,
    direction: "sent" | "received",
    transferStatus: TransferStatus = "pending",
  ) {
    const conv = getConversation(deviceId);
    // Don't duplicate if already exists
    const existing = conv.messages.find(
      (m) => m.transfer_id === transferId,
    );
    if (existing) return;

    const msg: ChatMessage = {
      id: `file-${transferId}`,
      device_id: deviceId,
      direction,
      content: `${i18n.global.t('chat.filePrefix')}${fileName}`,
      msg_type: "file",
      timestamp: Date.now(),
      status: "sent",
      transfer_id: transferId,
      file_name: fileName,
      file_size: fileSize,
      transferred_bytes: 0,
      transfer_status: transferStatus,
      speed_bps: 0,
    };
    conv.messages.push(msg);
    conv.last_message = msg;
    if (direction === "received") {
      conv.unread_count += 1;
    }
  }

  /** Add a snippet share message to the chat */
  function addSnippetMessage(
    deviceId: string,
    offerId: string,
    title: string,
    content: string,
    tag: string,
    note: string,
    direction: "sent" | "received",
  ) {
    const conv = getConversation(deviceId);
    const existing = conv.messages.find((m) => m.snippet_offer_id === offerId);
    if (existing) return;

    const msg: ChatMessage = {
      id: `snippet-${offerId}`,
      device_id: deviceId,
      direction,
      content: `[${i18n.global.t('snippet.share')}] ${title}`,
      msg_type: "snippet",
      timestamp: Date.now(),
      status: "sent",
      snippet_offer_id: offerId,
      snippet_title: title,
      snippet_content: content,
      snippet_tag: tag,
      snippet_note: note,
      snippet_status: direction === "sent" ? "accepted" : "pending",
    };
    conv.messages.push(msg);
    conv.last_message = msg;
    if (direction === "received") {
      conv.unread_count += 1;
    }
  }

  /** Find a file message by transfer_id across all conversations */
  function findFileMessage(transferId: string): ChatMessage | undefined {
    for (const conv of conversations.value.values()) {
      const msg = conv.messages.find((m) => m.transfer_id === transferId);
      if (msg) return msg;
    }
    return undefined;
  }

  /** Update file transfer progress on the chat message */
  function updateFileProgress(
    transferId: string,
    transferredBytes: number,
    speedBps: number,
    fileSize?: number,
  ) {
    const msg = findFileMessage(transferId);
    if (msg) {
      msg.transferred_bytes = transferredBytes;
      msg.speed_bps = speedBps;
      if (msg.transfer_status !== "active") {
        msg.transfer_status = "active";
      }
      if (fileSize && fileSize > 0) {
        msg.file_size = fileSize;
      }
    }
  }

  /** Update file transfer status */
  function updateFileStatus(transferId: string, status: TransferStatus, filePath?: string) {
    const msg = findFileMessage(transferId);
    if (msg) {
      msg.transfer_status = status;
      if (filePath) {
        msg.file_path = filePath;
      }
      if (status === "completed" && msg.file_size) {
        msg.transferred_bytes = msg.file_size;
        msg.speed_bps = 0;
      }
    }
  }

  async function loadHistory(deviceId: string, offset = 0, limit = 50) {
    if (!isTauri()) return;
    try {
      const history = await getMessageHistory(deviceId, offset, limit);
      const conv = getConversation(deviceId);
      if (offset === 0) {
        conv.messages = history;
      } else {
        conv.messages.unshift(...history);
      }
      if (history.length > 0) {
        conv.last_message = conv.messages[conv.messages.length - 1];
      }
    } catch (e) {
      console.error("Failed to load message history:", e);
    }
  }

  async function startListening() {
    if (!isTauri()) return;

    // Text messages
    const unlistenNewMsg = await listen<ChatMessage>(
      "new-message",
      (event) => {
        const msg = event.payload;
        const conv = getConversation(msg.device_id);
        conv.messages.push(msg);
        conv.last_message = msg;
        conv.unread_count += 1;
        // Flash taskbar when window not focused
        if (!document.hasFocus()) {
          invoke("flash_window").catch(() => {});
        }
      },
    );

    const unlistenSent = await listen<{ message_id: string; status: string }>(
      "message-sent",
      (event) => {
        for (const conv of conversations.value.values()) {
          const msg = conv.messages.find(
            (m) => m.id === event.payload.message_id,
          );
          if (msg) {
            msg.status = event.payload.status as ChatMessage["status"];
            break;
          }
        }
      },
    );

    // File offer received → add as chat message
    const unlistenFileOffer = await listen<FileOffer>(
      "file-offer",
      (event) => {
        const offer = event.payload;
        addFileMessage(
          offer.from_device_id,
          offer.transfer_id,
          offer.file_name,
          offer.file_size,
          "received",
          "pending",
        );
        // Flash taskbar for incoming file
        if (!document.hasFocus()) {
          invoke("flash_window").catch(() => {});
        }
      },
    );

    // Snippet offer received → add as chat message
    const unlistenSnippetOffer = await listen<any>(
      "snippet-offer",
      (event) => {
        const offer = event.payload;
        addSnippetMessage(
          offer.from_device_id,
          offer.offer_id,
          offer.title,
          offer.content,
          offer.tag || "",
          offer.note || "",
          "received",
        );
        // Flash taskbar for incoming snippet
        if (!document.hasFocus()) {
          invoke("flash_window").catch(() => {});
        }
      },
    );

    // File transfer progress
    const unlistenProgress = await listen<
      TransferProgress
    >("transfer-progress", (event) => {
      const p = event.payload;
      updateFileProgress(
        p.transfer_id,
        p.transferred_bytes,
        p.speed_bps,
        p.file_size,
      );
    });

    // File transfer status update (active, failed, etc.)
    const unlistenTransferUpdate = await listen<TransferTask>(
      "transfer-update",
      (event) => {
        const task = event.payload;
        updateFileStatus(
          task.transfer_id,
          task.status as TransferStatus,
          task.file_path,
        );
        // Also ensure the message exists (for sent files that got accepted)
        if (!findFileMessage(task.transfer_id) && task.device_id) {
          addFileMessage(
            task.device_id,
            task.transfer_id,
            task.file_name,
            task.file_size,
            task.direction === "send" ? "sent" : "received",
            task.status as TransferStatus,
          );
        }
      },
    );

    // File transfer complete
    const unlistenTransferComplete = await listen<{
      transfer_id: string;
      status: string;
    }>("transfer-complete", (event) => {
      updateFileStatus(
        event.payload.transfer_id,
        event.payload.status as TransferStatus,
      );
    });

    unlisteners = [
      unlistenNewMsg,
      unlistenSent,
      unlistenFileOffer,
      unlistenSnippetOffer,
      unlistenProgress,
      unlistenTransferUpdate,
      unlistenTransferComplete,
    ];
  }

  function stopListening() {
    for (const unlisten of unlisteners) {
      unlisten();
    }
    unlisteners = [];
  }

  return {
    conversations,
    totalUnread,
    getMessages,
    getUnreadCount,
    getConversation,
    markAsRead,
    sendMessage,
    addFileMessage,
    addSnippetMessage,
    findFileMessage,
    updateFileProgress,
    updateFileStatus,
    loadHistory,
    startListening,
    stopListening,
  };
});
