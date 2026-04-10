use serde::{Deserialize, Serialize};

/// Magic bytes: "PCOK"
pub const MAGIC: [u8; 4] = [0x50, 0x43, 0x4F, 0x4B];

/// Protocol version
pub const VERSION: u16 = 1;

/// UDP discovery port
pub const DISCOVERY_PORT: u16 = 52000;

/// TCP messaging/signaling port
pub const MESSAGING_PORT: u16 = 52001;

/// Multicast group address for discovery
pub const MULTICAST_ADDR: &str = "224.0.1.100";

/// Discovery beacon interval in seconds
pub const BEACON_INTERVAL_SECS: u64 = 10;

/// Device offline timeout in seconds
pub const OFFLINE_TIMEOUT_SECS: u64 = 30;

/// File transfer chunk size (64KB)
pub const CHUNK_SIZE: usize = 64 * 1024;

/// TCP probe concurrency (for active scanning)
pub const PROBE_CONCURRENCY: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum PacketType {
    /// UDP broadcast: Device announcement
    Announce = 1,
    /// UDP: Device going offline
    Bye = 2,
    /// UDP unicast: Response to an Announce (proves device is alive)
    AnnounceResponse = 3,
    /// TCP: Text message
    Text = 10,
    /// TCP: File transfer offer
    FileOffer = 20,
    /// TCP: File transfer accepted
    FileAccept = 21,
    /// TCP: File transfer rejected
    FileReject = 22,
    /// TCP: File data chunk
    FileChunk = 23,
    /// TCP: Clipboard content
    Clipboard = 30,
    /// TCP: Snippet share
    SnippetShare = 31,
    /// TCP: Acknowledgment
    Ack = 99,
}

impl PacketType {
    pub fn from_u16(v: u16) -> Option<Self> {
        match v {
            1 => Some(Self::Announce),
            2 => Some(Self::Bye),
            3 => Some(Self::AnnounceResponse),
            10 => Some(Self::Text),
            20 => Some(Self::FileOffer),
            21 => Some(Self::FileAccept),
            22 => Some(Self::FileReject),
            23 => Some(Self::FileChunk),
            30 => Some(Self::Clipboard),
            31 => Some(Self::SnippetShare),
            99 => Some(Self::Ack),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnouncePayload {
    pub device_name: String,
    pub platform: String,
    pub tcp_port: u16,
    pub features: u32,
    /// Devices known to be broadcast-restricted (included in broadcasts to help them be discovered)
    #[serde(default)]
    pub restricted_peers: Vec<PeerInfo>,
}

/// Compact device info for the restricted peers list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub device_id: String,
    pub device_name: String,
    pub ip_addr: String,
    pub tcp_port: u16,
    pub platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPayload {
    pub message_id: String,
    pub text: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOfferPayload {
    pub transfer_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub is_folder: bool,
    pub file_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAcceptPayload {
    pub transfer_id: String,
    pub receiver_port: u16,
    pub resume_offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRejectPayload {
    pub transfer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardPayload {
    pub content: String,
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetSharePayload {
    pub title: String,
    pub content: String,
    pub tag: String,
    pub note: String,
}
