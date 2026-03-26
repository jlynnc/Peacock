use crate::discovery::device::DiscoveryState;
use crate::protocol::types::MESSAGING_PORT;
use crate::storage::db::Database;
use crate::transfer::tracker::TransferManager;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Maximum concurrent file transfers
pub const MAX_CONCURRENT_TRANSFERS: usize = 10;

pub struct AppState {
    pub device_id: String,
    pub device_id_bytes: [u8; 16],
    pub device_name: String,
    pub ip_addr: String,
    pub tcp_port: u16,
    pub platform: String,
    pub discovery: DiscoveryState,
    pub transfers: TransferManager,
    pub db: Database,
    pub data_dir: PathBuf,
    pub download_dir: PathBuf,
    pub transfer_semaphore: Arc<Semaphore>,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> crate::error::Result<Self> {
        let db = Database::new(&data_dir)?;

        // Load or generate device ID
        let device_id = match db.get_setting("device_id")? {
            Some(id) => id,
            None => {
                let id = uuid::Uuid::new_v4().to_string();
                db.set_setting("device_id", &id)?;
                id
            }
        };

        let device_id_bytes = uuid::Uuid::parse_str(&device_id)
            .unwrap()
            .into_bytes();

        // Load or generate device name
        let device_name = match db.get_setting("device_name")? {
            Some(name) => name,
            None => {
                let name = hostname::get()
                    .map(|h| h.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "Peacock Device".to_string());
                db.set_setting("device_name", &name)?;
                name
            }
        };

        // Detect local IP
        let ip_addr = local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        // Detect platform
        let platform = detect_platform().to_string();

        // Download directory: use setting or default to user's Downloads
        let download_dir = match db.get_setting("download_dir")? {
            Some(dir) => PathBuf::from(dir),
            None => dirs::download_dir().unwrap_or_else(|| data_dir.join("downloads")),
        };

        Ok(Self {
            device_id,
            device_id_bytes,
            device_name,
            ip_addr,
            tcp_port: MESSAGING_PORT,
            platform,
            discovery: DiscoveryState::new(),
            transfers: TransferManager::new(),
            db,
            data_dir,
            download_dir,
            transfer_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_TRANSFERS)),
        })
    }
}

fn detect_platform() -> &'static str {
    #[cfg(target_os = "windows")]
    return "windows";
    #[cfg(target_os = "macos")]
    return "macos";
    #[cfg(target_os = "linux")]
    return "linux";
    #[cfg(target_os = "android")]
    return "android";
    #[cfg(target_os = "ios")]
    return "ios";
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android",
        target_os = "ios"
    )))]
    return "unknown";
}
