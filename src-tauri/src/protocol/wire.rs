use crate::error::{PeacockError, Result};
use crate::protocol::header::PacketHeader;
use crate::protocol::types::PacketType;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Encode a payload using bincode
pub fn encode_payload<T: serde::Serialize>(payload: &T) -> Result<Vec<u8>> {
    bincode::serialize(payload).map_err(PeacockError::Bincode)
}

/// Decode a payload using bincode
pub fn decode_payload<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T> {
    bincode::deserialize(data).map_err(PeacockError::Bincode)
}

/// Build a complete packet (header + payload bytes)
pub fn build_packet(
    packet_type: PacketType,
    device_id: &[u8; 16],
    payload: &[u8],
) -> Vec<u8> {
    let header = PacketHeader::new(packet_type, device_id, payload.len() as u32);
    let mut buf = Vec::with_capacity(PacketHeader::SIZE + payload.len());
    buf.extend_from_slice(&header.to_bytes());
    buf.extend_from_slice(payload);
    buf
}

/// Build a complete packet from a serializable payload
pub fn build_typed_packet<T: serde::Serialize>(
    packet_type: PacketType,
    device_id: &[u8; 16],
    payload: &T,
) -> Result<Vec<u8>> {
    let payload_bytes = encode_payload(payload)?;
    Ok(build_packet(packet_type, device_id, &payload_bytes))
}

/// Read a full packet from a TCP stream: header + payload
pub async fn read_packet(stream: &mut TcpStream) -> Result<(PacketHeader, Vec<u8>)> {
    let mut header_buf = [0u8; PacketHeader::SIZE];
    stream.read_exact(&mut header_buf).await?;

    let header = PacketHeader::from_bytes(&header_buf)
        .ok_or_else(|| PeacockError::Network("Invalid packet header".into()))?;

    if !header.is_valid() {
        return Err(PeacockError::Network("Invalid packet magic/version".into()));
    }

    let mut payload = vec![0u8; header.payload_length as usize];
    if header.payload_length > 0 {
        stream.read_exact(&mut payload).await?;
    }

    Ok((header, payload))
}

/// Write a full packet to a TCP stream
pub async fn write_packet(
    stream: &mut TcpStream,
    packet_type: PacketType,
    device_id: &[u8; 16],
    payload: &[u8],
) -> Result<()> {
    let packet = build_packet(packet_type, device_id, payload);
    stream.write_all(&packet).await?;
    stream.flush().await?;
    Ok(())
}

/// Write a typed packet to a TCP stream
pub async fn write_typed_packet<T: serde::Serialize>(
    stream: &mut TcpStream,
    packet_type: PacketType,
    device_id: &[u8; 16],
    payload: &T,
) -> Result<()> {
    let payload_bytes = encode_payload(payload)?;
    write_packet(stream, packet_type, device_id, &payload_bytes).await
}
