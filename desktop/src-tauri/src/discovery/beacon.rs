use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

use crate::protocol::header::PacketHeader;
use crate::protocol::types::{
    AnnouncePayload, PacketType, BEACON_INTERVAL_SECS, DISCOVERY_PORT, MULTICAST_ADDR,
};
use crate::protocol::wire::encode_payload;
use crate::state::AppState;

pub fn spawn_beacon(state: Arc<RwLock<AppState>>) {
    tauri::async_runtime::spawn(async move {
        loop {
            match run_beacon_loop(&state).await {
                Ok(()) => break, // clean exit
                Err(e) => {
                    warn!("Beacon socket failed: {}, rebuilding in 3s...", e);
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            }
        }
    });
}

async fn run_beacon_loop(state: &Arc<RwLock<AppState>>) -> crate::error::Result<()> {
    let socket = create_udp_socket()?;
    info!("Beacon started");

    let mut tick = interval(Duration::from_secs(BEACON_INTERVAL_SECS));
    let mut consecutive_errors = 0u32;

    loop {
        tick.tick().await;

        let mut st = state.write().await;
        st.discovery.refresh_restricted_status();
        let restricted_peers = st.discovery.get_restricted_peers();
        let payload = AnnouncePayload {
            device_name: st.device_name.clone(),
            platform: st.platform.clone(),
            tcp_port: st.tcp_port,
            features: 0xFFFF,
            restricted_peers,
        };

        let payload_bytes = match encode_payload(&payload) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Failed to encode announce: {}", e);
                continue;
            }
        };

        let header = PacketHeader::new(
            PacketType::Announce,
            &st.device_id_bytes,
            payload_bytes.len() as u32,
        );
        let mut packet = Vec::with_capacity(PacketHeader::SIZE + payload_bytes.len());
        packet.extend_from_slice(&header.to_bytes());
        packet.extend_from_slice(&payload_bytes);
        drop(st);

        let mut send_ok = false;

        // Layer 1: Multicast
        let multicast_addr: SocketAddr =
            format!("{}:{}", MULTICAST_ADDR, DISCOVERY_PORT).parse().unwrap();
        if socket.send_to(&packet, multicast_addr).await.is_ok() {
            send_ok = true;
        }

        // Layer 2: Directed subnet broadcast
        if let Some(broadcast_addr) = get_broadcast_address() {
            let target = SocketAddr::new(broadcast_addr, DISCOVERY_PORT);
            if socket.send_to(&packet, target).await.is_ok() {
                send_ok = true;
            }
        }

        // Layer 3: Limited broadcast
        let limited = SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), DISCOVERY_PORT);
        if socket.send_to(&packet, limited).await.is_ok() {
            send_ok = true;
        }

        if send_ok {
            consecutive_errors = 0;
        } else {
            consecutive_errors += 1;
            // If all sends fail repeatedly, socket is dead — rebuild
            if consecutive_errors >= 3 {
                warn!("Beacon: all sends failed {} times, rebuilding socket", consecutive_errors);
                return Err(crate::error::PeacockError::Network("Socket dead".into()));
            }
        }
    }
}

fn create_udp_socket() -> crate::error::Result<tokio::net::UdpSocket> {
    use socket2::{Domain, Protocol, Socket, Type};

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_broadcast(true)?;

    let multicast: Ipv4Addr = MULTICAST_ADDR.parse().unwrap();
    if let Err(e) = socket.join_multicast_v4(&multicast, &Ipv4Addr::UNSPECIFIED) {
        warn!("Failed to join multicast group: {}", e);
    }

    crate::net_util::bind_socket_to_wifi(&socket).ok();

    socket.set_nonblocking(true)?;
    socket.bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0).into())?;

    let std_socket: std::net::UdpSocket = socket.into();
    Ok(tokio::net::UdpSocket::from_std(std_socket)?)
}

fn get_broadcast_address() -> Option<IpAddr> {
    let ip_str = crate::state::detect_local_ip();
    if let Ok(ipv4) = ip_str.parse::<Ipv4Addr>() {
        let octets = ipv4.octets();
        return Some(IpAddr::V4(Ipv4Addr::new(octets[0], octets[1], octets[2], 255)));
    }
    None
}

pub async fn send_bye(state: &AppState) -> crate::error::Result<()> {
    let socket = create_udp_socket()?;

    let header = PacketHeader::new(PacketType::Bye, &state.device_id_bytes, 0);
    let packet = header.to_bytes().to_vec();

    let multicast_addr: SocketAddr =
        format!("{}:{}", MULTICAST_ADDR, DISCOVERY_PORT).parse().unwrap();
    let _ = socket.send_to(&packet, multicast_addr).await;

    let broadcast = SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), DISCOVERY_PORT);
    let _ = socket.send_to(&packet, broadcast).await;

    Ok(())
}
