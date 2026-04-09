use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::error::PeacockError;
use crate::messaging::client::send_to_device;
use crate::protocol::types::{PacketType, TextPayload};
use crate::state::AppState;

#[tauri::command]
pub async fn send_message(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    device_id: String,
    text: String,
) -> Result<String, PeacockError> {
    let (target_addr, self_device_id_bytes, message_id) = {
        let state = state.read().await;
        let device = state
            .discovery
            .get_device(&device_id)
            .ok_or_else(|| {
                PeacockError::DeviceNotFound(device_id.clone())
            })?;

        let addr: SocketAddr = format!("{}:{}", device.ip_addr, device.tcp_port)
            .parse()
            .map_err(|e| PeacockError::Network(format!("Invalid address: {}", e)))?;

        let msg_id = uuid::Uuid::new_v4().to_string();
        (addr, state.device_id_bytes, msg_id)
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let payload = TextPayload {
        message_id: message_id.clone(),
        text: text.clone(),
        timestamp: now,
    };

    send_to_device(target_addr, PacketType::Text, &self_device_id_bytes, &payload).await?;

    // Chat history not persisted — messages are memory-only

    Ok(message_id)
}

#[tauri::command]
pub async fn get_message_history(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    device_id: String,
    offset: i64,
    limit: i64,
) -> Result<Vec<serde_json::Value>, PeacockError> {
    let state = state.read().await;
    let messages = state.db.get_messages(&device_id, offset, limit)?;
    Ok(messages)
}
