export interface ClipboardEntry {
  id: string;
  content_preview: string;
  source_device_id: string;
  source_device_name: string;
  timestamp: number;
  content_type: ClipboardContentType;
}

export type ClipboardContentType = "text" | "image";
