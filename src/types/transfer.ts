export interface TransferTask {
  transfer_id: string;
  device_id: string;
  file_name: string;
  file_path: string;
  file_size: number;
  transferred_bytes: number;
  status: TransferStatus;
  direction: TransferDirection;
  speed_bps: number;
  is_folder: boolean;
  file_count: number;
  created_at: number;
}

export type TransferStatus =
  | "pending"
  | "active"
  | "paused"
  | "completed"
  | "failed"
  | "rejected";

export type TransferDirection = "send" | "receive";

export interface TransferProgress {
  transfer_id: string;
  transferred_bytes: number;
  speed_bps: number;
}

export interface FileOffer {
  transfer_id: string;
  file_name: string;
  file_size: number;
  is_folder: boolean;
  file_count: number;
  from_device_id: string;
  from_device_name: string;
}
