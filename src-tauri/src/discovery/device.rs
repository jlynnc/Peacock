use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::protocol::types::OFFLINE_TIMEOUT_SECS;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub ip_addr: String,
    pub tcp_port: u16,
    pub platform: String,
    pub last_seen: u64,
    pub is_online: bool,
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

    /// Upsert a device from a discovery announcement
    pub fn upsert_device(
        &mut self,
        device_id: String,
        device_name: String,
        ip_addr: IpAddr,
        tcp_port: u16,
        platform: String,
    ) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

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

    /// Mark a device as offline by ID
    pub fn mark_offline(&mut self, device_id: &str) -> bool {
        if let Some(device) = self.devices.get_mut(device_id) {
            if device.is_online {
                device.is_online = false;
                return true;
            }
        }
        false
    }

    /// Check for timed-out devices and mark them offline
    pub fn check_timeouts(&mut self) -> Vec<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut timed_out = Vec::new();

        for (id, device) in self.devices.iter_mut() {
            if device.is_online && (now - device.last_seen) > OFFLINE_TIMEOUT_SECS {
                device.is_online = false;
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
}
