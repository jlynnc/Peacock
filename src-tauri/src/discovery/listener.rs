use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

use crate::protocol::header::PacketHeader;
use crate::protocol::types::{
    AnnouncePayload, PacketType, DISCOVERY_PORT, MULTICAST_ADDR, OFFLINE_TIMEOUT_SECS,
};
use crate::protocol::wire::decode_payload;
use crate::state::AppState;

/// Spawn the discovery listener task
pub fn spawn_listener(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_listener(state, app_handle).await {
            error!("Listener task failed: {}", e);
        }
    });
}

async fn run_listener(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
) -> crate::error::Result<()> {
    println!("[PEACOCK-DEBUG] Discovery listener starting...");
    let socket = create_listen_socket()?;
    println!("[PEACOCK-DEBUG] Discovery listener STARTED on UDP port {}", DISCOVERY_PORT);
    info!("Discovery listener started on UDP port {}", DISCOVERY_PORT);

    let mut buf = [0u8; 2048];

    // Also spawn a timeout checker
    let state_clone = state.clone();
    let app_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let mut tick = interval(Duration::from_secs(OFFLINE_TIMEOUT_SECS / 2));
        loop {
            tick.tick().await;
            let mut state = state_clone.write().await;
            let timed_out = state.discovery.check_timeouts();
            for device_id in timed_out {
                info!("Device timed out: {}", device_id);
                let _ = app_clone.emit("device-offline", serde_json::json!({ "device_id": device_id }));
            }
        }
    });

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                if len < PacketHeader::SIZE {
                    continue;
                }

                let header_bytes: [u8; PacketHeader::SIZE] =
                    match buf[..PacketHeader::SIZE].try_into() {
                        Ok(b) => b,
                        Err(_) => continue,
                    };

                let header = match PacketHeader::from_bytes(&header_bytes) {
                    Some(h) if h.is_valid() => h,
                    _ => continue,
                };

                // Ignore our own packets
                let self_id = {
                    let s = state.read().await;
                    s.device_id_bytes
                };
                if header.device_id == self_id {
                    continue;
                }

                let device_id_str = uuid_from_bytes(&header.device_id);
                let source_ip = addr.ip();

                match header.get_packet_type() {
                    Some(PacketType::Announce) => {
                        let payload_end = PacketHeader::SIZE + header.payload_length as usize;
                        if len < payload_end {
                            continue;
                        }
                        let payload_bytes = &buf[PacketHeader::SIZE..payload_end];

                        let payload: AnnouncePayload = match decode_payload(payload_bytes) {
                            Ok(p) => p,
                            Err(e) => {
                                debug!("Failed to decode announce: {}", e);
                                continue;
                            }
                        };

                        let mut state = state.write().await;
                        let is_new_or_back = state.discovery.upsert_device(
                            device_id_str.clone(),
                            payload.device_name.clone(),
                            source_ip,
                            payload.tcp_port,
                            payload.platform.clone(),
                        );

                        if is_new_or_back {
                            println!("[PEACOCK-DEBUG] Device discovered: {} at {}:{}", payload.device_name, source_ip, payload.tcp_port);
                            info!(
                                "Device discovered: {} ({}) at {}",
                                payload.device_name, device_id_str, source_ip
                            );
                            if let Some(device) = state.discovery.get_device(&device_id_str) {
                                let _ = app_handle.emit("device-online", device.clone());
                            }
                        }
                    }
                    Some(PacketType::Bye) => {
                        let mut state = state.write().await;
                        if state.discovery.mark_offline(&device_id_str) {
                            info!("Device bye: {}", device_id_str);
                            let _ = app_handle.emit(
                                "device-offline",
                                serde_json::json!({ "device_id": device_id_str }),
                            );
                        }
                    }
                    _ => {
                        debug!("Ignoring non-discovery packet from {}", source_ip);
                    }
                }
            }
            Err(e) => {
                error!("UDP recv error: {}", e);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

fn create_listen_socket() -> crate::error::Result<tokio::net::UdpSocket> {
    use socket2::{Domain, Protocol, Socket, Type};

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;

    // On Windows, we need SO_REUSEADDR before bind
    #[cfg(target_os = "windows")]
    {
        // Windows-specific: allow port reuse
        let _ = socket.set_reuse_address(true);
    }

    socket.bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), DISCOVERY_PORT).into())?;
    socket.set_broadcast(true)?;
    socket.set_nonblocking(true)?;

    // Join multicast group
    let multicast: Ipv4Addr = MULTICAST_ADDR.parse().unwrap();
    if let Err(e) = socket.join_multicast_v4(&multicast, &Ipv4Addr::UNSPECIFIED) {
        warn!("Failed to join multicast group for listening: {}", e);
    }

    let std_socket: std::net::UdpSocket = socket.into();
    Ok(tokio::net::UdpSocket::from_std(std_socket)?)
}

fn uuid_from_bytes(bytes: &[u8; 16]) -> String {
    uuid::Uuid::from_bytes(*bytes).to_string()
}
