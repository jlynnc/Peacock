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

pub fn spawn_listener(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
) {
    // Timeout checker — runs independently, doesn't need socket
    let state_clone = state.clone();
    let app_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let mut tick = interval(Duration::from_secs(OFFLINE_TIMEOUT_SECS / 2));
        loop {
            tick.tick().await;
            let mut st = state_clone.write().await;
            let timed_out = st.discovery.check_timeouts();
            for device_id in timed_out {
                info!("Device timed out: {}", device_id);
                let _ = app_clone.emit("device-offline", serde_json::json!({ "device_id": device_id }));
            }
        }
    });

    // Listener — self-healing: rebuilds socket on failure
    tauri::async_runtime::spawn(async move {
        loop {
            match run_listener_loop(&state, &app_handle).await {
                Ok(()) => break,
                Err(e) => {
                    warn!("Listener socket failed: {}, rebuilding in 3s...", e);
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            }
        }
    });
}

async fn run_listener_loop(
    state: &Arc<RwLock<AppState>>,
    app_handle: &tauri::AppHandle,
) -> crate::error::Result<()> {
    let socket = Arc::new(create_listen_socket()?);
    info!("Discovery listener started on UDP port {}", DISCOVERY_PORT);

    let mut buf = [0u8; 4096];
    let mut consecutive_errors = 0u32;

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                consecutive_errors = 0;

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
                        handle_announce(
                            state, app_handle, &socket,
                            &device_id_str, source_ip, payload_bytes,
                        ).await;
                    }
                    Some(PacketType::AnnounceResponse) => {
                        handle_announce_response(
                            state, app_handle,
                            &device_id_str, source_ip, payload_bytes,
                        ).await;
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
                    // Message/signaling packets — now received via UDP
                    Some(pt @ (PacketType::Text | PacketType::FileOffer |
                               PacketType::FileAccept | PacketType::FileReject |
                               PacketType::SnippetShare)) => {
                        let peer_addr = SocketAddr::new(source_ip, addr.port());
                        crate::messaging::handler::handle_udp_packet(
                            state, app_handle, header, payload_bytes.to_vec(), peer_addr,
                        ).await;
                    }
                    _ => {}
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                error!("UDP recv error: {}", e);
                if consecutive_errors >= 5 {
                    // Socket is dead — return error to trigger rebuild
                    return Err(crate::error::PeacockError::Network(
                        format!("Listener socket dead after {} errors: {}", consecutive_errors, e),
                    ));
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

async fn handle_announce(
    state: &Arc<RwLock<AppState>>,
    app_handle: &tauri::AppHandle,
    socket: &tokio::net::UdpSocket,
    device_id_str: &str,
    source_ip: IpAddr,
    payload_bytes: &[u8],
) {
    let payload: AnnouncePayload = match decode_payload(payload_bytes) {
        Ok(p) => p,
        Err(e) => {
            debug!("Failed to decode announce: {}", e);
            return;
        }
    };

    let mut st = state.write().await;
    let is_new_or_back = st.discovery.upsert_device(
        device_id_str.to_string(),
        payload.device_name.clone(),
        source_ip,
        payload.tcp_port,
        payload.platform.clone(),
    );

    st.discovery.mark_can_broadcast(device_id_str);

    if is_new_or_back {
        info!("Device discovered (broadcast): {} at {}", payload.device_name, source_ip);
        if let Some(device) = st.discovery.get_device(device_id_str) {
            let _ = app_handle.emit("device-online", device.clone());
        }
    }

    // Merge restricted peers from the broadcast
    let self_id_str = st.device_id.clone();
    let new_peers = st.discovery.merge_restricted_peers(&payload.restricted_peers, &self_id_str);
    for peer_id in &new_peers {
        if let Some(device) = st.discovery.get_device(peer_id) {
            info!("Device discovered (restricted list): {} at {}", device.device_name, device.ip_addr);
            let _ = app_handle.emit("device-online", device.clone());
        }
    }

    // UDP unicast response
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
        restricted_peers: Vec::new(),
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

async fn handle_announce_response(
    state: &Arc<RwLock<AppState>>,
    app_handle: &tauri::AppHandle,
    device_id_str: &str,
    source_ip: IpAddr,
    payload_bytes: &[u8],
) {
    let payload: AnnouncePayload = match decode_payload(payload_bytes) {
        Ok(p) => p,
        Err(e) => {
            debug!("Failed to decode announce response: {}", e);
            return;
        }
    };

    let mut st = state.write().await;
    let is_new_or_back = st.discovery.upsert_device(
        device_id_str.to_string(),
        payload.device_name.clone(),
        source_ip,
        payload.tcp_port,
        payload.platform.clone(),
    );

    st.discovery.mark_responded(device_id_str);

    if is_new_or_back {
        info!("Device discovered (response): {} at {}", payload.device_name, source_ip);
        if let Some(device) = st.discovery.get_device(device_id_str) {
            let _ = app_handle.emit("device-online", device.clone());
        }
    }
}

fn create_listen_socket() -> crate::error::Result<tokio::net::UdpSocket> {
    use socket2::{Domain, Protocol, Socket, Type};

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;

    crate::net_util::bind_socket_to_wifi(&socket).ok();

    socket.bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), DISCOVERY_PORT).into())?;
    socket.set_broadcast(true)?;
    socket.set_nonblocking(true)?;

    let multicast: Ipv4Addr = MULTICAST_ADDR.parse().unwrap();
    if let Err(e) = socket.join_multicast_v4(&multicast, &Ipv4Addr::UNSPECIFIED) {
        warn!("Failed to join multicast group: {}", e);
    }

    let std_socket: std::net::UdpSocket = socket.into();
    Ok(tokio::net::UdpSocket::from_std(std_socket)?)
}

fn uuid_from_bytes(bytes: &[u8; 16]) -> String {
    uuid::Uuid::from_bytes(*bytes).to_string()
}
