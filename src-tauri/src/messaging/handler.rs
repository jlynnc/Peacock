use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::protocol::header::PacketHeader;
use crate::protocol::types::{
    FileAcceptPayload, FileOfferPayload, FileRejectPayload, PacketType, SnippetSharePayload,
    TextPayload,
};
use crate::protocol::wire::decode_payload;
use crate::state::AppState;
use crate::transfer::tracker::TransferStatus;

/// Handle an incoming packet after header + payload have been read
pub async fn handle_packet(
    state: &Arc<RwLock<AppState>>,
    app: &tauri::AppHandle,
    header: PacketHeader,
    payload: Vec<u8>,
    peer_addr: SocketAddr,
    _stream: &mut TcpStream,
) {
    let device_id_str = uuid::Uuid::from_bytes(header.device_id).to_string();

    match header.get_packet_type() {
        Some(PacketType::Text) => {
            handle_text(state, app, &device_id_str, &payload, peer_addr).await;
        }
        Some(PacketType::FileOffer) => {
            handle_file_offer(state, app, &device_id_str, &payload, peer_addr).await;
        }
        Some(PacketType::FileAccept) => {
            handle_file_accept(state, app, &device_id_str, &payload, peer_addr).await;
        }
        Some(PacketType::FileReject) => {
            handle_file_reject(state, app, &device_id_str, &payload).await;
        }
        Some(PacketType::Clipboard) => {
            debug!("Clipboard from {}", device_id_str);
        }
        Some(PacketType::SnippetShare) => {
            handle_snippet_share(state, app, &device_id_str, &payload).await;
        }
        _ => {
            debug!(
                "Unknown packet type {} from {}",
                header.packet_type, peer_addr
            );
        }
    }
}

async fn handle_text(
    state: &Arc<RwLock<AppState>>,
    app: &tauri::AppHandle,
    device_id_str: &str,
    payload: &[u8],
    peer_addr: SocketAddr,
) {
    match decode_payload::<TextPayload>(payload) {
        Ok(text_payload) => {
            info!(
                "Text message from {} ({}): {}",
                device_id_str,
                peer_addr,
                &text_payload.text[..text_payload.text.len().min(50)]
            );

            {
                let state = state.read().await;
                if let Err(e) = state.db.store_message(
                    &text_payload.message_id,
                    device_id_str,
                    "received",
                    &text_payload.text,
                    "text",
                    text_payload.timestamp,
                ) {
                    error!("Failed to store message: {}", e);
                }
            }

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let _ = app.emit(
                "new-message",
                serde_json::json!({
                    "id": text_payload.message_id,
                    "device_id": device_id_str,
                    "direction": "received",
                    "content": text_payload.text,
                    "msg_type": "text",
                    "timestamp": if text_payload.timestamp > 0 { text_payload.timestamp } else { now },
                    "status": "sent"
                }),
            );
        }
        Err(e) => {
            error!("Failed to decode text message: {}", e);
        }
    }
}

async fn handle_file_offer(
    state: &Arc<RwLock<AppState>>,
    app: &tauri::AppHandle,
    device_id_str: &str,
    payload: &[u8],
    peer_addr: SocketAddr,
) {
    match decode_payload::<FileOfferPayload>(payload) {
        Ok(offer) => {
            info!(
                "File offer from {} ({}): {} ({} bytes)",
                device_id_str, peer_addr, offer.file_name, offer.file_size
            );

            // Create a receive task in tracker
            {
                let mut st = state.write().await;
                st.transfers.create_receive_task(
                    offer.transfer_id.clone(),
                    device_id_str.to_string(),
                    offer.file_name.clone(),
                    offer.file_size,
                    offer.is_folder,
                    offer.file_count,
                );
            }

            // Get device name for display
            let from_device_name = {
                let st = state.read().await;
                st.discovery
                    .get_device(device_id_str)
                    .map(|d| d.device_name.clone())
                    .unwrap_or_else(|| device_id_str.to_string())
            };

            // Emit to frontend for user to accept/reject
            let _ = app.emit(
                "file-offer",
                serde_json::json!({
                    "transfer_id": offer.transfer_id,
                    "file_name": offer.file_name,
                    "file_size": offer.file_size,
                    "is_folder": offer.is_folder,
                    "file_count": offer.file_count,
                    "from_device_id": device_id_str,
                    "from_device_name": from_device_name,
                }),
            );
        }
        Err(e) => {
            error!("Failed to decode file offer: {}", e);
        }
    }
}

async fn handle_file_accept(
    state: &Arc<RwLock<AppState>>,
    app: &tauri::AppHandle,
    device_id_str: &str,
    payload: &[u8],
    peer_addr: SocketAddr,
) {
    match decode_payload::<FileAcceptPayload>(payload) {
        Ok(accept) => {
            info!(
                "File accepted by {} for {}, receiver port={}",
                device_id_str, accept.transfer_id, accept.receiver_port
            );

            // Get file path and build receiver address
            let (file_path, receiver_addr) = {
                let st = state.read().await;
                let task = match st.transfers.get_task(&accept.transfer_id) {
                    Some(t) => t,
                    None => {
                        error!("Transfer {} not found", accept.transfer_id);
                        return;
                    }
                };

                let fp = task.file_path.clone();

                // Use the peer's IP with the receiver port
                let peer_ip = peer_addr.ip();
                let addr = SocketAddr::new(peer_ip, accept.receiver_port);
                (fp, addr)
            };

            // Start sending the file
            crate::transfer::sender::start_sending(
                state.clone(),
                app.clone(),
                accept.transfer_id,
                file_path,
                receiver_addr,
                accept.resume_offset,
            )
            .await;
        }
        Err(e) => {
            error!("Failed to decode file accept: {}", e);
        }
    }
}

async fn handle_file_reject(
    state: &Arc<RwLock<AppState>>,
    app: &tauri::AppHandle,
    device_id_str: &str,
    payload: &[u8],
) {
    match decode_payload::<FileRejectPayload>(payload) {
        Ok(reject) => {
            info!(
                "File rejected by {} for {}",
                device_id_str, reject.transfer_id
            );

            {
                let mut st = state.write().await;
                st.transfers
                    .set_status(&reject.transfer_id, TransferStatus::Rejected);
                let task = st.transfers.get_task(&reject.transfer_id).cloned();
                drop(st);
                if let Some(task) = task {
                    let _ = app.emit("transfer-update", &task);
                }
            }
        }
        Err(e) => {
            error!("Failed to decode file reject: {}", e);
        }
    }
}

async fn handle_snippet_share(
    state: &Arc<RwLock<AppState>>,
    app: &tauri::AppHandle,
    device_id_str: &str,
    payload: &[u8],
) {
    match decode_payload::<SnippetSharePayload>(payload) {
        Ok(snippet) => {
            info!("Snippet shared from {}: {}", device_id_str, snippet.title);

            let snippet_id = uuid::Uuid::new_v4().to_string();

            {
                let st = state.read().await;
                if let Err(e) = st.db.create_snippet(
                    &snippet_id,
                    &snippet.title,
                    &snippet.content,
                    &snippet.tag,
                    &snippet.note,
                ) {
                    error!("Failed to save shared snippet: {}", e);
                    return;
                }
            }

            let from_device_name = {
                let st = state.read().await;
                st.discovery
                    .get_device(device_id_str)
                    .map(|d| d.device_name.clone())
                    .unwrap_or_else(|| device_id_str.to_string())
            };

            let _ = app.emit(
                "snippet-received",
                serde_json::json!({
                    "id": snippet_id,
                    "title": snippet.title,
                    "content": snippet.content,
                    "tag": snippet.tag,
                    "note": snippet.note,
                    "from_device_id": device_id_str,
                    "from_device_name": from_device_name,
                }),
            );
        }
        Err(e) => {
            error!("Failed to decode snippet share: {}", e);
        }
    }
}
