use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info};

use crate::protocol::types::{MESSAGING_PORT, PROBE_CONCURRENCY};
use crate::state::AppState;

/// Layer 3: Active TCP probe - scan the local subnet for Peacock instances
pub fn spawn_probe(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        // Wait a bit for multicast/broadcast to work first
        tokio::time::sleep(Duration::from_secs(5)).await;

        if let Err(e) = run_probe(state, app_handle).await {
            debug!("Probe task ended: {}", e);
        }
    });
}

async fn run_probe(
    state: Arc<RwLock<AppState>>,
    _app_handle: tauri::AppHandle,
) -> crate::error::Result<()> {
    let local_ip = match local_ip_address::local_ip() {
        Ok(IpAddr::V4(ip)) => ip,
        _ => {
            debug!("Cannot determine local IPv4, skipping probe");
            return Ok(());
        }
    };

    let octets = local_ip.octets();
    let subnet_prefix = [octets[0], octets[1], octets[2]];

    info!(
        "Starting TCP probe on subnet {}.{}.{}.0/24",
        subnet_prefix[0], subnet_prefix[1], subnet_prefix[2]
    );

    let semaphore = Arc::new(Semaphore::new(PROBE_CONCURRENCY));

    let mut handles = Vec::new();

    for host in 1..=254u8 {
        let ip = Ipv4Addr::new(subnet_prefix[0], subnet_prefix[1], subnet_prefix[2], host);

        // Skip our own IP
        if ip == local_ip {
            continue;
        }

        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let state = state.clone();

        let handle = tauri::async_runtime::spawn(async move {
            let _permit = permit;
            let addr = SocketAddr::new(IpAddr::V4(ip), MESSAGING_PORT);

            // Try to connect with a short timeout
            match tokio::time::timeout(Duration::from_millis(500), TcpStream::connect(addr)).await {
                Ok(Ok(mut stream)) => {
                    // Successfully connected - this is a Peacock instance!
                    // We could send a probe/handshake here
                    debug!("TCP probe found Peacock at {}", ip);

                    // Read header to verify it's a Peacock instance
                    // For now, just note the discovery
                    let _ = stream;
                }
                _ => {
                    // Connection failed or timed out - not a Peacock instance
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all probes to complete
    for handle in handles {
        let _ = handle.await;
    }

    info!("TCP probe completed");
    Ok(())
}
