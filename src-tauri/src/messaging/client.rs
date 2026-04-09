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

async fn connect_tcp(target_addr: SocketAddr) -> Result<TcpStream> {
    // On iOS, bind the socket to the Wi-Fi interface (en0) using IP_BOUND_IF
    // Without this, iOS may route through the wrong interface → "No route to host"
    #[cfg(target_os = "ios")]
    {
        use socket2::{Domain, Protocol, Socket, Type};

        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
            .map_err(|e| PeacockError::Network(format!("Socket create: {}", e)))?;

        // Get en0 (Wi-Fi) interface index and bind socket to it
        let ifindex = unsafe { libc_ifnametoindex() };
        if ifindex > 0 {
            unsafe {
                let fd = socket2_fd(&socket);
                let ret = libc::setsockopt(
                    fd,
                    libc::IPPROTO_IP,
                    25, // IP_BOUND_IF on iOS/macOS
                    &ifindex as *const u32 as *const libc::c_void,
                    std::mem::size_of::<u32>() as u32,
                );
                if ret != 0 {
                    return Err(PeacockError::Network(format!(
                        "setsockopt IP_BOUND_IF failed: {}",
                        std::io::Error::last_os_error()
                    )));
                }
            }
        }

        socket.set_nonblocking(true)
            .map_err(|e| PeacockError::Network(format!("Set nonblocking: {}", e)))?;

        // Non-blocking connect
        match socket.connect(&target_addr.into()) {
            Ok(_) => {}
            Err(e) if e.raw_os_error() == Some(36) => {} // EINPROGRESS
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                return Err(PeacockError::Network(format!("Connect to {}: {}", target_addr, e)));
            }
        }

        let std_stream: std::net::TcpStream = socket.into();
        let stream = TcpStream::from_std(std_stream)
            .map_err(|e| PeacockError::Network(format!("Tokio wrap: {}", e)))?;

        // Wait for async connect to complete
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
        TcpStream::connect(target_addr)
            .await
            .map_err(|e| PeacockError::Network(format!("Cannot connect to {}: {}", target_addr, e)))
    }
}

/// Get the en0 (Wi-Fi) interface index on iOS
#[cfg(target_os = "ios")]
unsafe fn libc_ifnametoindex() -> u32 {
    let name = std::ffi::CString::new("en0").unwrap();
    libc::if_nametoindex(name.as_ptr())
}

/// Get raw file descriptor from socket2::Socket
#[cfg(target_os = "ios")]
unsafe fn socket2_fd(socket: &socket2::Socket) -> i32 {
    use std::os::unix::io::AsRawFd;
    socket.as_raw_fd()
}
