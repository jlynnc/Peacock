export interface Room {
  id: string;
  name: string;
  member_ids: string[];
  created_at: number;
}

export interface RoomMessage {
  room_id: string;
  message_id: string;
  sender_id: string;
  sender_name: string;
  text: string;
  timestamp: number;
  direction: "sent" | "received";
}
