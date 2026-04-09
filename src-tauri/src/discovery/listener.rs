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
use crate::protocol::wire::{decode_payload, encode_payload};
use crate::state::AppState;

/// Spawn the discovery listener task
pub fn spawn_listener(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
) {
    // Main listener
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
    let socket = Arc::new(create_listen_socket()?);
    info!("Discovery listener started on UDP port {}", DISCOVERY_PORT);

    let mut buf = [0u8; 4096];

    // Spawn timeout checker
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

                let payload_end = PacketHeader::SIZE + header.payload_length as usize;
                if len < payload_end {
                    continue;
                }
                let payload_bytes = &buf[PacketHeader::SIZE..payload_end];

                match header.get_packet_type() {
                    Some(PacketType::Announce) => {
                        // Received a UDP broadcast from another device
                        let payload: AnnouncePayload = match decode_payload(payload_bytes) {
                            Ok(p) => p,
                            Err(e) => {
                                debug!("Failed to decode announce: {}", e);
                                continue;
                            }
                        };

                        let mut st = state.write().await;
                        let is_new_or_back = st.discovery.upsert_device(
                            device_id_str.clone(),
                            payload.device_name.clone(),
                            source_ip,
                            payload.tcp_port,
                            payload.platform.clone(),
                        );

                        // This device can broadcast
                        st.discovery.mark_can_broadcast(&device_id_str);

                        if is_new_or_back {
                            info!("Device discovered (broadcast): {} at {}", payload.device_name, source_ip);
                            if let Some(device) = st.discovery.get_device(&device_id_str) {
                                let _ = app_handle.emit("device-online", device.clone());
                            }
                        }

                        // Process restricted peers list from the broadcast
                        let self_id_str = st.device_id.clone();
                        let new_peers = st.discovery.merge_restricted_peers(
                            &payload.restricted_peers,
                            &self_id_str,
                        );
                        for peer_id in &new_peers {
                            if let Some(device) = st.discovery.get_device(peer_id) {
                                info!("Device discovered (via restricted list): {} at {}", device.device_name, device.ip_addr);
                                let _ = app_handle.emit("device-online", device.clone());
                            }
                        }

                        // Send UDP unicast response back
                        let self_name = st.device_name.clone();
                        let self_platform = st.platform.clone();
                        let self_port = st.tcp_port;
                        let self_id_bytes = st.device_id_bytes;
                        drop(st);

                        let response = AnnouncePayload {
                            device_name: self_name,
                            platform: self_platform,
                            tcp_port: self_port,
                            features: 0xFFFF,
                            restricted_peers: Vec::new(), // responses don't carry the list
                        };
                        if let Ok(resp_bytes) = encode_payload(&response) {
                            let resp_header = PacketHeader::new(
                                PacketType::AnnounceResponse,
                                &self_id_bytes,
                                resp_bytes.len() as u32,
                            );
                            let mut packet = Vec::with_capacity(PacketHeader::SIZE + resp_bytes.len());
                            packet.extend_from_slice(&resp_header.to_bytes());
                            packet.extend_from_slice(&resp_bytes);

                            let target = SocketAddr::new(source_ip, DISCOVERY_PORT);
                            let _ = socket.send_to(&packet, target).await;
                        }
                    }

                    Some(PacketType::AnnounceResponse) => {
                        // Received a UDP unicast response — this device is alive
                        let payload: AnnouncePayload = match decode_payload(payload_bytes) {
                            Ok(p) => p,
                            Err(e) => {
                                debug!("Failed to decode announce response: {}", e);
                                continue;
                            }
                        };

                        let mut st = state.write().await;
                        let is_new_or_back = st.discovery.upsert_device(
                            device_id_str.clone(),
                            payload.device_name.clone(),
                            source_ip,
                            payload.tcp_port,
                            payload.platform.clone(),
                        );

                        // This device responded but may not be able to broadcast
                        st.discovery.mark_responded(&device_id_str);

                        if is_new_or_back {
                            info!("Device discovered (response): {} at {}", payload.device_name, source_ip);
                            if let Some(device) = st.discovery.get_device(&device_id_str) {
                                let _ = app_handle.emit("device-online", device.clone());
                            }
                        }
                    }

                    Some(PacketType::Bye) => {
                        let mut st = state.write().await;
                        if st.discovery.mark_offline(&device_id_str) {
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

    // On iOS, bind to Wi-Fi interface
    crate::net_util::bind_socket_to_wifi(&socket).ok();

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
