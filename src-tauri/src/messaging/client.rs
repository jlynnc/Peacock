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

    tracing::info!("Attempting TCP connect to {}", target_addr);
    let mut stream = TcpStream::connect(target_addr)
        .await
        .map_err(|e| {
            tracing::error!("TCP connect to {} failed: {}", target_addr, e);
            PeacockError::Network(format!("Cannot connect to {}: {}", target_addr, e))
        })?;
    tracing::info!("TCP connected to {} successfully", target_addr);

    debug!("Connected to {} for {:?}", target_addr, packet_type);

    write_packet(&mut stream, packet_type, device_id, &payload_bytes).await?;

    Ok(())
}
