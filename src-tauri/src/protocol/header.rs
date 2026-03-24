use crate::protocol::types::{PacketType, MAGIC, VERSION};
use serde::{Deserialize, Serialize};

/// Packet header - 32 bytes total
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub packet_type: u16,
    pub device_id: [u8; 16],
    pub payload_length: u32,
    pub reserved: [u8; 4],
}

impl PacketHeader {
    pub const SIZE: usize = 32;

    pub fn new(packet_type: PacketType, device_id: &[u8; 16], payload_length: u32) -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            packet_type: packet_type as u16,
            device_id: *device_id,
            payload_length,
            reserved: [0; 4],
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..6].copy_from_slice(&self.version.to_be_bytes());
        buf[6..8].copy_from_slice(&self.packet_type.to_be_bytes());
        buf[8..24].copy_from_slice(&self.device_id);
        buf[24..28].copy_from_slice(&self.payload_length.to_be_bytes());
        buf[28..32].copy_from_slice(&self.reserved);
        buf
    }

    pub fn from_bytes(buf: &[u8; Self::SIZE]) -> Option<Self> {
        if buf[0..4] != MAGIC {
            return None;
        }
        let version = u16::from_be_bytes([buf[4], buf[5]]);
        let packet_type = u16::from_be_bytes([buf[6], buf[7]]);
        let mut device_id = [0u8; 16];
        device_id.copy_from_slice(&buf[8..24]);
        let payload_length = u32::from_be_bytes([buf[24], buf[25], buf[26], buf[27]]);
        let mut reserved = [0u8; 4];
        reserved.copy_from_slice(&buf[28..32]);

        Some(Self {
            magic: MAGIC,
            version,
            packet_type,
            device_id,
            payload_length,
            reserved,
        })
    }

    pub fn get_packet_type(&self) -> Option<PacketType> {
        PacketType::from_u16(self.packet_type)
    }

    pub fn is_valid(&self) -> bool {
        self.magic == MAGIC && self.version == VERSION
    }
}
