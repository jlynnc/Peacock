export interface DeviceInfo {
  device_id: string;
  device_name: string;
  ip_addr: string;
  tcp_port: number;
  platform: DevicePlatform;
  last_seen: number;
  is_online: boolean;
  is_restricted?: boolean;
}

export type DevicePlatform =
  | "windows"
  | "macos"
  | "linux"
  | "android"
  | "ios"
  | "web";

export interface SelfInfo {
  device_id: string;
  device_name: string;
  ip_addr: string;
  tcp_port: number;
  platform: DevicePlatform;
}
