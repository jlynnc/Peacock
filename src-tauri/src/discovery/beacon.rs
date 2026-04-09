use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, warn};

use crate::protocol::header::PacketHeader;
use crate::protocol::types::{
    AnnouncePayload, PacketType, BEACON_INTERVAL_SECS, DISCOVERY_PORT, MULTICAST_ADDR,
};
use crate::protocol::wire::encode_payload;
use crate::state::AppState;

/// Spawn the discovery beacon task that broadcasts our presence
pub fn spawn_beacon(state: Arc<RwLock<AppState>>) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_beacon(state).await {
            error!("Beacon task failed: {}", e);
        }
    });
}

async fn run_beacon(state: Arc<RwLock<AppState>>) -> crate::error::Result<()> {
    println!("[PEACOCK-DEBUG] Beacon starting...");
    let socket = create_udp_socket()?;
    println!("[PEACOCK-DEBUG] Beacon UDP socket created");

    let mut tick = interval(Duration::from_secs(BEACON_INTERVAL_SECS));

    loop {
        tick.tick().await;

        let state = state.read().await;
        let payload = AnnouncePayload {
            device_name: state.device_name.clone(),
            platform: state.platform.clone(),
            tcp_port: state.tcp_port,
            features: 0xFFFF, // All features supported
        };

        let payload_bytes = match encode_payload(&payload) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Failed to encode announce payload: {}", e);
                continue;
            }
        };

        let header =
            PacketHeader::new(PacketType::Announce, &state.device_id_bytes, payload_bytes.len() as u32);
        let mut packet = Vec::with_capacity(PacketHeader::SIZE + payload_bytes.len());
        packet.extend_from_slice(&header.to_bytes());
        packet.extend_from_slice(&payload_bytes);

        // Layer 1: UDP Multicast
        let multicast_addr: SocketAddr =
            format!("{}:{}", MULTICAST_ADDR, DISCOVERY_PORT).parse().unwrap();
        if let Err(e) = socket.send_to(&packet, multicast_addr).await {
            debug!("Multicast send failed: {}", e);
        }

        // Layer 2: Directed subnet broadcast
        if let Some(broadcast_addr) = get_broadcast_address() {
            let broadcast_target = SocketAddr::new(broadcast_addr, DISCOVERY_PORT);
            if let Err(e) = socket.send_to(&packet, broadcast_target).await {
                debug!("Broadcast send failed: {}", e);
            }
        }

        // Also send to 255.255.255.255 (limited broadcast)
        let limited_broadcast = SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), DISCOVERY_PORT);
        if let Err(e) = socket.send_to(&packet, limited_broadcast).await {
            debug!("Limited broadcast send failed: {}", e);
        }
    }
}

fn create_udp_socket() -> crate::error::Result<tokio::net::UdpSocket> {
    use socket2::{Domain, Protocol, Socket, Type};

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_broadcast(true)?;

    // Join multicast group
    let multicast: Ipv4Addr = MULTICAST_ADDR.parse().unwrap();
    if let Err(e) = socket.join_multicast_v4(&multicast, &Ipv4Addr::UNSPECIFIED) {
        warn!("Failed to join multicast group: {}", e);
    }

    socket.set_nonblocking(true)?;
    socket.bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0).into())?;

    let std_socket: std::net::UdpSocket = socket.into();
    Ok(tokio::net::UdpSocket::from_std(std_socket)?)
}

/// Get the broadcast address for the primary network interface
fn get_broadcast_address() -> Option<IpAddr> {
    if let Ok(ip) = local_ip_address::local_ip() {
        if let IpAddr::V4(ipv4) = ip {
            // Assume /24 subnet
            let octets = ipv4.octets();
            return Some(IpAddr::V4(Ipv4Addr::new(octets[0], octets[1], octets[2], 255)));
        }
    }
    None
}

/// Send a BYE packet for graceful shutdown
pub async fn send_bye(state: &AppState) -> crate::error::Result<()> {
    let socket = create_udp_socket()?;

    let header = PacketHeader::new(PacketType::Bye, &state.device_id_bytes, 0);
    let packet = header.to_bytes().to_vec();

    // Send BYE to multicast
    let multicast_addr: SocketAddr =
        format!("{}:{}", MULTICAST_ADDR, DISCOVERY_PORT).parse().unwrap();
    let _ = socket.send_to(&packet, multicast_addr).await;

    // Send BYE to broadcast
    let broadcast = SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), DISCOVERY_PORT);
    let _ = socket.send_to(&packet, broadcast).await;

    Ok(())
}
