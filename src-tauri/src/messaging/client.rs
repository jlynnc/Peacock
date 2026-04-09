use std::net::SocketAddr;
use tokio::net::TcpStream;
use tracing::debug;

use crate::error::{PeacockError, Result};
use crate::protocol::types::PacketType;
use crate::protocol::wire::{encode_payload, write_packet};

/// Send a typed message to a target device
pub async fn send_to_device<T: serde::Serialize>(
    target_addr: SocketAddr,
    packet_type: PacketType,
    device_id: &[u8; 16],
    payload: &T,
) -> Result<()> {
    let payload_bytes = encode_payload(payload)?;

    let mut stream = connect_tcp(target_addr).await?;

    debug!("Connected to {} for {:?}", target_addr, packet_type);

    write_packet(&mut stream, packet_type, device_id, &payload_bytes).await?;

    Ok(())
}

/// Connect TCP — on iOS, bind to local Wi-Fi IP first to force correct interface
async fn connect_tcp(target_addr: SocketAddr) -> Result<TcpStream> {
    #[cfg(target_os = "ios")]
    {
        use socket2::{Domain, Protocol, Socket, Type};
        use std::net::{Ipv4Addr, SocketAddrV4};

        let local_ip = crate::state::detect_local_ip();
        let local_v4: Ipv4Addr = local_ip.parse().unwrap_or(Ipv4Addr::UNSPECIFIED);
        let bind_addr = SocketAddr::V4(SocketAddrV4::new(local_v4, 0));

        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
            .map_err(|e| PeacockError::Network(format!("Socket create: {}", e)))?;

        socket.bind(&bind_addr.into())
            .map_err(|e| PeacockError::Network(format!("Bind to {}: {}", bind_addr, e)))?;

        socket.set_nonblocking(true)
            .map_err(|e| PeacockError::Network(format!("Set nonblocking: {}", e)))?;

        // Non-blocking connect — will return WouldBlock, that's expected
        match socket.connect(&target_addr.into()) {
            Ok(_) => {}
            Err(e) if e.raw_os_error() == Some(36) => {} // EINPROGRESS on iOS/macOS
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                return Err(PeacockError::Network(format!("Connect to {}: {}", target_addr, e)));
            }
        }

        let std_stream: std::net::TcpStream = socket.into();
        let stream = TcpStream::from_std(std_stream)
            .map_err(|e| PeacockError::Network(format!("Tokio wrap: {}", e)))?;

        // Wait for connection to complete
        stream.writable().await
            .map_err(|e| PeacockError::Network(format!("Connect await {}: {}", target_addr, e)))?;

        // Check for connection error
        if let Some(err) = stream.take_error()
            .map_err(|e| PeacockError::Network(format!("take_error: {}", e)))? {
            return Err(PeacockError::Network(format!("Connect to {}: {}", target_addr, err)));
        }

        Ok(stream)
    }

    #[cfg(not(target_os = "ios"))]
    {
        TcpStream::connect(target_addr)
            .await
            .map_err(|e| PeacockError::Network(format!("Cannot connect to {}: {}", target_addr, e)))
    }
}
