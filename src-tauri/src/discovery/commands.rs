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

/// Notify the messaging server to rebuild its socket (called when app resumes)
#[tauri::command]
pub async fn rebuild_server(
    rebuild_tx: tauri::State<'_, tokio::sync::watch::Sender<bool>>,
) -> Result<(), PeacockError> {
    let _ = rebuild_tx.send(true);
    Ok(())
}

/// Debug: send a UDP unicast packet to a target IP to test if iOS can send UDP
#[tauri::command]
pub async fn udp_test(target_ip: String) -> Result<String, PeacockError> {
    use std::net::SocketAddr;

    let target: SocketAddr = format!("{}:52002", target_ip)
        .parse()
        .map_err(|e| PeacockError::Network(format!("Invalid IP: {}", e)))?;

    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| PeacockError::Network(format!("Bind: {}", e)))?;

    // On iOS, bind to Wi-Fi
    #[cfg(target_os = "ios")]
    {
        use std::os::unix::io::AsRawFd;
        let _ = crate::net_util::bind_to_wifi(socket.as_raw_fd());
    }

    let msg = b"PEACOCK_UDP_TEST";
    socket
        .send_to(msg, target)
        .await
        .map_err(|e| PeacockError::Network(format!("Send: {}", e)))?;

    Ok(format!("Sent UDP to {}", target))
}

#[derive(serde::Serialize)]
pub struct SelfInfo {
    pub device_id: String,
    pub device_name: String,
    pub ip_addr: String,
    pub tcp_port: u16,
    pub platform: String,
}
