use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::protocol::types::{PeerInfo, BEACON_INTERVAL_SECS, OFFLINE_TIMEOUT_SECS};

/// Broadcast-restricted threshold: if no broadcast received within this many seconds,
/// the device is considered restricted. Set to 3x beacon interval.
const RESTRICTED_THRESHOLD_SECS: u64 = BEACON_INTERVAL_SECS * 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub ip_addr: String,
    pub tcp_port: u16,
    pub platform: String,
    pub last_seen: u64,
    pub is_online: bool,
    /// Timestamp of last broadcast received from this device (0 = never)
    #[serde(skip)]
    pub last_broadcast_at: u64,
    /// Whether this device is broadcast-restricted (derived, set before sending to frontend)
    #[serde(default)]
    pub is_restricted: bool,
}

#[derive(Debug)]
pub struct DiscoveryState {
    devices: HashMap<String, DeviceInfo>,
}

impl DiscoveryState {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    /// Add or update a device discovered via AnnounceResponse (Rule 1).
    /// Returns true if device is new or was offline.
    pub fn upsert_from_response(
        &mut self,
        device_id: String,
        device_name: String,
        ip_addr: IpAddr,
        tcp_port: u16,
        platform: String,
    ) -> bool {
        let now = now_secs();
        let is_new = !self.devices.contains_key(&device_id);

        let device = self
            .devices
            .entry(device_id.clone())
            .or_insert_with(|| DeviceInfo {
                device_id: device_id.clone(),
                device_name: device_name.clone(),
                ip_addr: ip_addr.to_string(),
                tcp_port,
                platform: platform.clone(),
                last_seen: now,
                is_online: true,
                last_broadcast_at: 0,
                is_restricted: false,
            });

        device.device_name = device_name;
        device.ip_addr = ip_addr.to_string();
        device.tcp_port = tcp_port;
        device.platform = platform;
        device.last_seen = now;

        let was_offline = !device.is_online;
        device.is_online = true;

        is_new || was_offline
    }

    /// Add a device discovered via restricted_peers self-discovery (Rule 2).
    pub fn upsert_from_restricted_self_discovery(
        &mut self,
        device_id: String,
        device_name: String,
        ip_addr: IpAddr,
        tcp_port: u16,
        platform: String,
    ) -> bool {
        self.upsert_from_response(device_id, device_name, ip_addr, tcp_port, platform)
    }

    /// Called when we receive a broadcast FROM a device.
    /// Updates last_broadcast_at. Does NOT add the device to our list.
    pub fn mark_received_broadcast(&mut self, device_id: &str) {
        if let Some(device) = self.devices.get_mut(device_id) {
            let now = now_secs();
            device.last_broadcast_at = now;
            device.last_seen = now;
        }
    }

    /// Called before each beacon send: refresh restricted status for all devices
    /// based on whether we've received their broadcast within the threshold.
    pub fn refresh_restricted_status(&mut self) {
        let now = now_secs();
        for device in self.devices.values_mut() {
            if !device.is_online {
                continue;
            }
            device.is_restricted = device.last_broadcast_at == 0
                || (now - device.last_broadcast_at) > RESTRICTED_THRESHOLD_SECS;
        }
    }

    /// Get the list of restricted devices (for including in our broadcasts).
    pub fn get_restricted_peers(&self) -> Vec<PeerInfo> {
        self.devices
            .values()
            .filter(|d| d.is_online && d.is_restricted)
            .map(|d| PeerInfo {
                device_id: d.device_id.clone(),
                device_name: d.device_name.clone(),
                ip_addr: d.ip_addr.clone(),
                tcp_port: d.tcp_port,
                platform: d.platform.clone(),
            })
            .collect()
    }

    /// Mark a device as offline by ID
    pub fn mark_offline(&mut self, device_id: &str) -> bool {
        if let Some(device) = self.devices.get_mut(device_id) {
            if device.is_online {
                device.is_online = false;
                device.last_broadcast_at = 0;
                return true;
            }
        }
        false
    }

    /// Check for timed-out devices and mark them offline
    pub fn check_timeouts(&mut self) -> Vec<String> {
        let now = now_secs();
        let mut timed_out = Vec::new();

        for (id, device) in self.devices.iter_mut() {
            if device.is_online && (now - device.last_seen) > OFFLINE_TIMEOUT_SECS {
                device.is_online = false;
                device.last_broadcast_at = 0;
                timed_out.push(id.clone());
            }
        }

        timed_out
    }

    /// Get all online devices
    pub fn get_online_devices(&self) -> Vec<DeviceInfo> {
        self.devices
            .values()
            .filter(|d| d.is_online)
            .cloned()
            .collect()
    }

    /// Get a device by ID
    pub fn get_device(&self, device_id: &str) -> Option<&DeviceInfo> {
        self.devices.get(device_id)
    }

    /// Get a device clone with current status
    pub fn get_device_with_status(&self, device_id: &str) -> Option<DeviceInfo> {
        self.devices.get(device_id).cloned()
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
