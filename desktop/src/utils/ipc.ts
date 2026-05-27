import { invoke } from "@tauri-apps/api/core";
import type { DeviceInfo, SelfInfo } from "@/types/device";
import type { ChatMessage } from "@/types/message";
import type { TransferTask } from "@/types/transfer";

// Discovery commands
export async function getOnlineDevices(): Promise<DeviceInfo[]> {
  return invoke("get_online_devices");
}

export async function getSelfInfo(): Promise<SelfInfo> {
  return invoke("get_self_info");
}

// Messaging commands
export async function sendMessage(
  deviceId: string,
  text: string,
): Promise<string> {
  return invoke("send_message", { deviceId, text });
}

export async function getMessageHistory(
  deviceId: string,
  offset: number,
  limit: number,
): Promise<ChatMessage[]> {
  return invoke("get_message_history", { deviceId, offset, limit });
}

// Transfer commands
export async function sendFile(
  deviceId: string,
  filePath: string,
): Promise<string> {
  return invoke("send_file", { deviceId, filePath });
}

export async function sendFolder(
  deviceId: string,
  folderPath: string,
): Promise<string> {
  return invoke("send_folder", { deviceId, folderPath });
}

export async function sendPaths(
  deviceId: string,
  paths: string[],
): Promise<string[]> {
  return invoke("send_paths", { deviceId, paths });
}

export async function acceptTransfer(transferId: string): Promise<void> {
  return invoke("accept_transfer", { transferId });
}

export async function rejectTransfer(transferId: string): Promise<void> {
  return invoke("reject_transfer", { transferId });
}

export async function pauseTransfer(transferId: string): Promise<void> {
  return invoke("pause_transfer", { transferId });
}

export async function resumeTransfer(transferId: string): Promise<void> {
  return invoke("resume_transfer", { transferId });
}

export async function cancelTransfer(transferId: string): Promise<void> {
  return invoke("cancel_transfer", { transferId });
}

export async function getActiveTransfers(): Promise<TransferTask[]> {
  return invoke("get_active_transfers");
}

// Clipboard commands
export async function enableClipboardSync(enabled: boolean): Promise<void> {
  return invoke("enable_clipboard_sync", { enabled });
}

export async function pushClipboard(deviceId: string): Promise<void> {
  return invoke("push_clipboard", { deviceId });
}

// Settings commands
export async function updateDeviceName(name: string): Promise<void> {
  return invoke("update_device_name", { name });
}

export async function updateDownloadDir(path: string): Promise<void> {
  return invoke("update_download_dir", { path });
}

// File utilities
export async function openFileLocation(path: string): Promise<void> {
  return invoke("open_file_location", { path });
}

export async function deleteFile(path: string): Promise<void> {
  return invoke("delete_file", { path });
}

// Snippet commands
export async function getSnippets(): Promise<any[]> {
  return invoke("get_snippets");
}

export async function createSnippet(
  title: string,
  content: string,
  tag: string,
  note: string,
): Promise<string> {
  return invoke("create_snippet", { title, content, tag, note });
}

export async function updateSnippet(
  id: string,
  title: string,
  content: string,
  tag: string,
  note: string,
): Promise<void> {
  return invoke("update_snippet", { id, title, content, tag, note });
}

export async function deleteSnippet(id: string): Promise<void> {
  return invoke("delete_snippet", { id });
}

export async function copySnippetCount(id: string): Promise<void> {
  return invoke("copy_snippet", { id });
}

export async function shareSnippet(
  deviceId: string,
  title: string,
  content: string,
  tag: string,
  note: string,
): Promise<void> {
  return invoke("share_snippet", { deviceId, title, content, tag, note });
}

// Room commands
export async function createRoom(
  roomName: string,
  memberIds: string[],
): Promise<string> {
  return invoke("create_room", { roomName, memberIds });
}

export async function getRooms(): Promise<any[]> {
  return invoke("get_rooms");
}

export async function deleteRoom(roomId: string): Promise<void> {
  return invoke("delete_room", { roomId });
}

export async function sendRoomMessage(
  roomId: string,
  text: string,
): Promise<string> {
  return invoke("send_room_message", { roomId, text });
}

export async function sendRoomFile(
  roomId: string,
  filePath: string,
): Promise<string> {
  return invoke("send_room_file", { roomId, filePath });
}
