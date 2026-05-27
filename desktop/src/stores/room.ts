import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Room, RoomMessage } from "@/types/room";
import {
  createRoom as ipcCreateRoom,
  getRooms as ipcGetRooms,
  deleteRoom as ipcDeleteRoom,
  sendRoomMessage as ipcSendRoomMessage,
  sendRoomFile as ipcSendRoomFile,
} from "@/utils/ipc";
import { isTauri } from "@/utils/platform";

export const useRoomStore = defineStore("room", () => {
  const rooms = ref<Room[]>([]);
  const messages = ref<Map<string, RoomMessage[]>>(new Map());
  const selectedRoomId = ref<string | null>(null);

  const selectedRoom = computed(() =>
    rooms.value.find((r) => r.id === selectedRoomId.value) || null,
  );

  function getRoomMessages(roomId: string): RoomMessage[] {
    return messages.value.get(roomId) || [];
  }

  let unlisteners: UnlistenFn[] = [];

  async function init() {
    if (!isTauri()) return;

    // Load rooms from DB
    try {
      const dbRooms = await ipcGetRooms();
      rooms.value = dbRooms.map((r: any) => ({
        id: r.id,
        name: r.name,
        member_ids: r.member_ids,
        created_at: r.created_at,
      }));
    } catch (e) {
      console.error("Failed to load rooms:", e);
    }

    // Listen for room created (from other devices)
    const unlistenCreated = await listen<any>("room-created", (event) => {
      const { room_id, room_name, member_ids } = event.payload;
      // Don't duplicate
      if (!rooms.value.find((r) => r.id === room_id)) {
        rooms.value.push({
          id: room_id,
          name: room_name,
          member_ids,
          created_at: Date.now() / 1000,
        });
      }
    });

    // Listen for room messages
    const unlistenMessage = await listen<any>("room-message", (event) => {
      const msg = event.payload;
      const roomMsgs = messages.value.get(msg.room_id) || [];
      roomMsgs.push({
        room_id: msg.room_id,
        message_id: msg.message_id,
        sender_id: msg.sender_id,
        sender_name: msg.sender_name,
        text: msg.text,
        timestamp: msg.timestamp,
        direction: "received",
      });
      messages.value.set(msg.room_id, roomMsgs);
    });

    // Listen for room file offers
    const unlistenFileOffer = await listen<any>("room-file-offer", (event) => {
      const offer = event.payload;
      const roomMsgs = messages.value.get(offer.room_id) || [];
      roomMsgs.push({
        room_id: offer.room_id,
        message_id: `file-${offer.transfer_id}`,
        sender_id: offer.sender_id,
        sender_name: offer.sender_name,
        text: `[文件] ${offer.file_name}`,
        timestamp: Date.now(),
        direction: "received",
      });
      messages.value.set(offer.room_id, roomMsgs);
    });

    unlisteners = [unlistenCreated, unlistenMessage, unlistenFileOffer];
  }

  async function createRoom(name: string, memberIds: string[]) {
    const roomId = await ipcCreateRoom(name, memberIds);
    rooms.value.push({
      id: roomId,
      name,
      member_ids: memberIds,
      created_at: Date.now() / 1000,
    });
    return roomId;
  }

  async function deleteRoomById(roomId: string) {
    await ipcDeleteRoom(roomId);
    rooms.value = rooms.value.filter((r) => r.id !== roomId);
    messages.value.delete(roomId);
    if (selectedRoomId.value === roomId) {
      selectedRoomId.value = null;
    }
  }

  async function sendMessage(roomId: string, text: string, selfId: string, selfName: string) {
    const messageId = await ipcSendRoomMessage(roomId, text);
    const roomMsgs = messages.value.get(roomId) || [];
    roomMsgs.push({
      room_id: roomId,
      message_id: messageId,
      sender_id: selfId,
      sender_name: selfName,
      text,
      timestamp: Date.now(),
      direction: "sent",
    });
    messages.value.set(roomId, roomMsgs);
  }

  async function sendFile(roomId: string, filePath: string, fileName: string, selfId: string, selfName: string) {
    const transferId = await ipcSendRoomFile(roomId, filePath);
    const roomMsgs = messages.value.get(roomId) || [];
    roomMsgs.push({
      room_id: roomId,
      message_id: `file-${transferId}`,
      sender_id: selfId,
      sender_name: selfName,
      text: `[文件] ${fileName}`,
      timestamp: Date.now(),
      direction: "sent",
    });
    messages.value.set(roomId, roomMsgs);
  }

  function selectRoom(roomId: string | null) {
    selectedRoomId.value = roomId;
  }

  function cleanup() {
    for (const unlisten of unlisteners) {
      unlisten();
    }
    unlisteners = [];
  }

  return {
    rooms,
    messages,
    selectedRoomId,
    selectedRoom,
    getRoomMessages,
    init,
    createRoom,
    deleteRoomById,
    sendMessage,
    sendFile,
    selectRoom,
    cleanup,
  };
});
