use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::protocol::header::PacketHeader;
use crate::protocol::types::{
    FileAcceptPayload, FileOfferPayload, FileRejectPayload, PacketType,
    RoomCreatePayload, RoomFileOfferPayload, RoomMessagePayload,
    SnippetSharePayload, TextPayload,
};
use crate::protocol::wire::decode_payload;
use crate::state::AppState;
use crate::transfer::tracker::TransferStatus;

/// Handle an incoming packet received via UDP
pub async fn handle_udp_packet(
    state: &Arc<RwLock<AppState>>,
    app: &tauri::AppHandle,
    header: PacketHeader,
    payload: Vec<u8>,
    peer_addr: SocketAddr,
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
        Some(PacketType::SnippetShare) => {
            handle_snippet_share(state, app, &device_id_str, &payload).await;
        }
        Some(PacketType::RoomCreate) => {
            handle_room_create(state, app, &device_id_str, &payload).await;
        }
        Some(PacketType::RoomMessage) => {
            handle_room_message(app, &payload).await;
        }
        Some(PacketType::RoomFileOffer) => {
            handle_room_file_offer(state, app, &payload, peer_addr).await;
        }
        _ => {
            debug!("Unknown packet type {} from {}", header.packet_type, peer_addr);
        }
    }
}

async fn handle_text(
    _state: &Arc<RwLock<AppState>>,
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
                text_payload.text.chars().take(50).collect::<String>()
            );

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
            info!("Snippet offer from {}: {}", device_id_str, snippet.title);

            let from_device_name = {
                let st = state.read().await;
                st.discovery
                    .get_device(device_id_str)
                    .map(|d| d.device_name.clone())
                    .unwrap_or_else(|| device_id_str.to_string())
            };

            // Emit as snippet-offer (like file-offer), don't auto-save
            let _ = app.emit(
                "snippet-offer",
                serde_json::json!({
                    "offer_id": uuid::Uuid::new_v4().to_string(),
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

// ── Room handlers ──

async fn handle_room_create(
    state: &Arc<RwLock<AppState>>,
    app: &tauri::AppHandle,
    _device_id_str: &str,
    payload: &[u8],
) {
    match decode_payload::<RoomCreatePayload>(payload) {
        Ok(room) => {
            info!("Room created: {} ({})", room.room_name, room.room_id);

            // Save to database
            {
                let st = state.read().await;
                let _ = st.db.create_room(&room.room_id, &room.room_name, &room.member_ids);
            }

            let _ = app.emit("room-created", serde_json::json!({
                "room_id": room.room_id,
                "room_name": room.room_name,
                "member_ids": room.member_ids,
            }));
        }
        Err(e) => {
            error!("Failed to decode room create: {}", e);
        }
    }
}

async fn handle_room_message(
    app: &tauri::AppHandle,
    payload: &[u8],
) {
    match decode_payload::<RoomMessagePayload>(payload) {
        Ok(msg) => {
            let _ = app.emit("room-message", serde_json::json!({
                "room_id": msg.room_id,
                "message_id": msg.message_id,
                "sender_id": msg.sender_id,
                "sender_name": msg.sender_name,
                "text": msg.text,
                "timestamp": msg.timestamp,
            }));
        }
        Err(e) => {
            error!("Failed to decode room message: {}", e);
        }
    }
}

async fn handle_room_file_offer(
    state: &Arc<RwLock<AppState>>,
    app: &tauri::AppHandle,
    payload: &[u8],
    peer_addr: SocketAddr,
) {
    match decode_payload::<RoomFileOfferPayload>(payload) {
        Ok(offer) => {
            info!(
                "Room file offer: {} in room {} from {}",
                offer.file_name, offer.room_id, offer.sender_name
            );

            // Create receive task (reuse existing transfer tracker)
            {
                let mut st = state.write().await;
                st.transfers.create_receive_task(
                    offer.transfer_id.clone(),
                    offer.sender_id.clone(),
                    offer.file_name.clone(),
                    offer.file_size,
                    offer.is_folder,
                    offer.file_count,
                );
            }

            let _ = app.emit("room-file-offer", serde_json::json!({
                "room_id": offer.room_id,
                "transfer_id": offer.transfer_id,
                "sender_id": offer.sender_id,
                "sender_name": offer.sender_name,
                "file_name": offer.file_name,
                "file_size": offer.file_size,
                "is_folder": offer.is_folder,
                "file_count": offer.file_count,
            }));
        }
        Err(e) => {
            error!("Failed to decode room file offer: {}", e);
        }
    }
}

