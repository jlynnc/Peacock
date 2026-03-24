use std::sync::Arc;
use tokio::sync::RwLock;

use crate::discovery::device::DeviceInfo;
use crate::error::PeacockError;
use crate::state::AppState;

#[tauri::command]
pub async fn get_online_devices(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<Vec<DeviceInfo>, PeacockError> {
    let state = state.read().await;
    Ok(state.discovery.get_online_devices())
}

#[tauri::command]
pub async fn get_self_info(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<SelfInfo, PeacockError> {
    let state = state.read().await;
    Ok(SelfInfo {
        device_id: state.device_id.clone(),
        device_name: state.device_name.clone(),
        ip_addr: state.ip_addr.clone(),
        tcp_port: state.tcp_port,
        platform: state.platform.clone(),
    })
}

#[derive(serde::Serialize)]
pub struct SelfInfo {
    pub device_id: String,
    pub device_name: String,
    pub ip_addr: String,
    pub tcp_port: u16,
    pub platform: String,
}
