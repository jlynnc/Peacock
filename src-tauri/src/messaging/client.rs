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

    let mut stream = TcpStream::connect(target_addr)
        .await
        .map_err(|e| PeacockError::Network(format!("Cannot connect to {}: {}", target_addr, e)))?;

    debug!("Connected to {} for {:?}", target_addr, packet_type);

    write_packet(&mut stream, packet_type, device_id, &payload_bytes).await?;

    Ok(())
}
