use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::protocol::types::{PeerInfo, OFFLINE_TIMEOUT_SECS};

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
    /// Devices that have responded to our broadcast but never broadcast themselves
    broadcast_restricted: HashSet<String>,
    /// Devices we've seen broadcast (to distinguish from response-only devices)
    has_broadcast: HashSet<String>,
}

impl DiscoveryState {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            broadcast_restricted: HashSet::new(),
            has_broadcast: HashSet::new(),
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

    /// Mark that we received a broadcast FROM this device (it can broadcast)
    pub fn mark_can_broadcast(&mut self, device_id: &str) {
        self.has_broadcast.insert(device_id.to_string());
        self.broadcast_restricted.remove(device_id);
    }

    /// Mark that we received a response FROM this device (but not a broadcast)
    /// If we've never seen this device broadcast, it's restricted
    pub fn mark_responded(&mut self, device_id: &str) {
        if !self.has_broadcast.contains(device_id) {
            self.broadcast_restricted.insert(device_id.to_string());
        }
    }

    /// Get the list of broadcast-restricted devices (for including in our broadcasts)
    pub fn get_restricted_peers(&self) -> Vec<PeerInfo> {
        self.broadcast_restricted
            .iter()
            .filter_map(|id| {
                self.devices.get(id).filter(|d| d.is_online).map(|d| PeerInfo {
                    device_id: d.device_id.clone(),
                    device_name: d.device_name.clone(),
                    ip_addr: d.ip_addr.clone(),
                    tcp_port: d.tcp_port,
                    platform: d.platform.clone(),
                })
            })
            .collect()
    }

    /// Add devices from a restricted peers list (received in someone else's broadcast)
    pub fn merge_restricted_peers(&mut self, peers: &[PeerInfo], self_device_id: &str) -> Vec<String> {
        let mut newly_added = Vec::new();
        for peer in peers {
            if peer.device_id == self_device_id {
                continue; // don't add ourselves
            }
            let ip: IpAddr = match peer.ip_addr.parse() {
                Ok(ip) => ip,
                Err(_) => continue,
            };
            let is_new = self.upsert_device(
                peer.device_id.clone(),
                peer.device_name.clone(),
                ip,
                peer.tcp_port,
                peer.platform.clone(),
            );
            if is_new {
                newly_added.push(peer.device_id.clone());
            }
            // This peer is restricted (that's why it's in the list)
            self.broadcast_restricted.insert(peer.device_id.clone());
        }
        newly_added
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

        // Clean up restricted tracking for offline devices
        for id in &timed_out {
            self.broadcast_restricted.remove(id);
            self.has_broadcast.remove(id);
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
