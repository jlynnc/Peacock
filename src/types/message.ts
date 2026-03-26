export interface ChatMessage {
  id: string;
  device_id: string;
  direction: MessageDirection;
  content: string;
  msg_type: MessageType;
  timestamp: number;
  status: MessageStatus;
  // File transfer fields (only for msg_type === "file")
  transfer_id?: string;
  file_name?: string;
  file_size?: number;
  transferred_bytes?: number;
  transfer_status?: TransferStatus;
  speed_bps?: number;
  file_path?: string;
  // Snippet share fields (only for msg_type === "snippet")
  snippet_offer_id?: string;
  snippet_title?: string;
  snippet_content?: string;
  snippet_tag?: string;
  snippet_note?: string;
  snippet_status?: "pending" | "accepted" | "rejected";
}

export type MessageDirection = "sent" | "received";

export type MessageType = "text" | "file" | "snippet" | "clipboard" | "system";

export type MessageStatus = "sending" | "sent" | "failed";

export type TransferStatus =
  | "pending"
  | "active"
  | "paused"
  | "completed"
  | "failed"
  | "rejected";

export interface Conversation {
  device_id: string;
  messages: ChatMessage[];
  unread_count: number;
  last_message?: ChatMessage;
  draft?: string;
}
