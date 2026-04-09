use std::net::SocketAddr;
use tracing::debug;

use crate::error::{PeacockError, Result};
use crate::protocol::header::PacketHeader;
use crate::protocol::types::{PacketType, DISCOVERY_PORT};
use crate::protocol::wire::encode_payload;

/// Send a typed message to a target device via UDP unicast
pub async fn send_to_device<T: serde::Serialize>(
    target_addr: SocketAddr,
    packet_type: PacketType,
    device_id: &[u8; 16],
    payload: &T,
) -> Result<()> {
    let payload_bytes = encode_payload(payload)?;

    let header = PacketHeader::new(packet_type, device_id, payload_bytes.len() as u32);
    let mut packet = Vec::with_capacity(PacketHeader::SIZE + payload_bytes.len());
    packet.extend_from_slice(&header.to_bytes());
    packet.extend_from_slice(&payload_bytes);

    // Send to the target's UDP discovery port
    let udp_target = SocketAddr::new(target_addr.ip(), DISCOVERY_PORT);

    let socket = create_udp_send_socket()?;
    socket
        .send_to(&packet, udp_target)
        .await
        .map_err(|e| PeacockError::Network(format!("UDP send to {}: {}", udp_target, e)))?;

    debug!("Sent {:?} to {} ({} bytes)", packet_type, udp_target, packet.len());

    Ok(())
}

fn create_udp_send_socket() -> Result<tokio::net::UdpSocket> {
    use socket2::{Domain, Protocol, Socket, Type};
    use std::net::{IpAddr, Ipv4Addr};

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|e| PeacockError::Network(format!("Socket create: {}", e)))?;

    crate::net_util::bind_socket_to_wifi(&socket).ok();

    socket.set_nonblocking(true)
        .map_err(|e| PeacockError::Network(format!("Set nonblocking: {}", e)))?;

    socket.bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0).into())
        .map_err(|e| PeacockError::Network(format!("Bind: {}", e)))?;

    let std_socket: std::net::UdpSocket = socket.into();
    tokio::net::UdpSocket::from_std(std_socket)
        .map_err(|e| PeacockError::Network(format!("Tokio wrap: {}", e)))
}

/// TCP connect — still needed for file transfer only
pub async fn ios_aware_tcp_connect(target_addr: SocketAddr) -> Result<tokio::net::TcpStream> {
    #[cfg(target_os = "ios")]
    {
        use socket2::{Domain, Protocol, Socket, Type};

        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
            .map_err(|e| PeacockError::Network(format!("Socket create: {}", e)))?;

        crate::net_util::bind_socket_to_wifi(&socket)
            .map_err(|e| PeacockError::Network(format!("Bind to Wi-Fi: {}", e)))?;

        socket.set_nonblocking(true)
            .map_err(|e| PeacockError::Network(format!("Set nonblocking: {}", e)))?;

        match socket.connect(&target_addr.into()) {
            Ok(_) => {}
            Err(e) if e.raw_os_error() == Some(36) => {}
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                return Err(PeacockError::Network(format!("Connect to {}: {}", target_addr, e)));
            }
        }

        let std_stream: std::net::TcpStream = socket.into();
        let stream = tokio::net::TcpStream::from_std(std_stream)
            .map_err(|e| PeacockError::Network(format!("Tokio wrap: {}", e)))?;

        stream.writable().await
            .map_err(|e| PeacockError::Network(format!("Connect await {}: {}", target_addr, e)))?;

        if let Some(err) = stream.take_error()
            .map_err(|e| PeacockError::Network(format!("take_error: {}", e)))? {
            return Err(PeacockError::Network(format!("Connect to {}: {}", target_addr, err)));
        }

        Ok(stream)
    }

    #[cfg(not(target_os = "ios"))]
    {
        tokio::net::TcpStream::connect(target_addr)
            .await
            .map_err(|e| PeacockError::Network(format!("Cannot connect to {}: {}", target_addr, e)))
    }
}
