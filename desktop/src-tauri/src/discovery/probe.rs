use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info};

use crate::protocol::header::PacketHeader;
use crate::protocol::types::{
    AnnouncePayload, PacketType, MESSAGING_PORT, PROBE_CONCURRENCY,
};
use crate::protocol::wire::{decode_payload, encode_payload, write_packet, read_packet};
use crate::state::AppState;

/// Spawn the TCP probe — scans local subnet for Peacock instances
pub fn spawn_probe(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        // Wait for multicast/broadcast discovery to work first
        tokio::time::sleep(Duration::from_secs(5)).await;

        loop {
            if let Err(e) = run_probe(state.clone(), app_handle.clone()).await {
                debug!("Probe task ended: {}", e);
            }
            // Repeat probe every 30 seconds to find new devices
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}

async fn run_probe(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
) -> crate::error::Result<()> {
    // Use our detect_local_ip which works correctly on iOS
    let local_ip_str = crate::state::detect_local_ip();
    let local_ip: Ipv4Addr = match local_ip_str.parse() {
        Ok(ip) => ip,
        _ => {
            debug!("Cannot parse local IP '{}', skipping probe", local_ip_str);
            return Ok(());
        }
    };

    let octets = local_ip.octets();
    let subnet_prefix = [octets[0], octets[1], octets[2]];

    debug!(
        "TCP probe scanning {}.{}.{}.0/24",
        subnet_prefix[0], subnet_prefix[1], subnet_prefix[2]
    );

    let semaphore = Arc::new(Semaphore::new(PROBE_CONCURRENCY));

    // Get our device info for the handshake
    let (device_id_bytes, device_name, platform, tcp_port) = {
        let st = state.read().await;
        (
            st.device_id_bytes,
            st.device_name.clone(),
            st.platform.clone(),
            st.tcp_port,
        )
    };

    let mut handles = Vec::new();

    for host in 1..=254u8 {
        let ip = Ipv4Addr::new(subnet_prefix[0], subnet_prefix[1], subnet_prefix[2], host);

        if ip == local_ip {
            continue;
        }

        // Skip IPs we already know about
        {
            let st = state.read().await;
            let already_known = st.discovery.get_online_devices().iter().any(|d| d.ip_addr == ip.to_string());
            if already_known {
                continue;
            }
        }

        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let state = state.clone();
        let app = app_handle.clone();
        let dev_id = device_id_bytes;
        let dev_name = device_name.clone();
        let plat = platform.clone();

        let handle = tauri::async_runtime::spawn(async move {
            let _permit = permit;
            let addr = SocketAddr::new(IpAddr::V4(ip), MESSAGING_PORT);

            match tokio::time::timeout(
                Duration::from_millis(500),
                crate::messaging::client::ios_aware_tcp_connect(addr),
            )
            .await
            {
                Ok(Ok(mut stream)) => {
                    debug!("TCP probe connected to {}", ip);

                    // Send our Announce so the remote side discovers us
                    let payload = AnnouncePayload {
                        device_name: dev_name,
                        platform: plat,
                        tcp_port,
                        features: 0xFFFF,
                        restricted_peers: Vec::new(),
                    };
                    if let Ok(payload_bytes) = encode_payload(&payload) {
                        let _ = write_packet(
                            &mut stream,
                            PacketType::Announce,
                            &dev_id,
                            &payload_bytes,
                        )
                        .await;
                    }

                    // Try to read remote's response (they might send Announce back)
                    match tokio::time::timeout(Duration::from_millis(500), read_packet(&mut stream)).await {
                        Ok(Ok((header, payload_bytes))) => {
                            if header.get_packet_type() == Some(PacketType::Announce) {
                                if let Ok(announce) = decode_payload::<AnnouncePayload>(&payload_bytes) {
                                    let device_id = uuid::Uuid::from_bytes(header.device_id).to_string();
                                    let mut st = state.write().await;
                                    let is_new = st.discovery.upsert_from_response(
                                        device_id.clone(),
                                        announce.device_name.clone(),
                                        IpAddr::V4(ip),
                                        announce.tcp_port,
                                        announce.platform.clone(),
                                    );
                                    if is_new {
                                        info!("TCP probe discovered: {} at {}", announce.device_name, ip);
                                        if let Some(device) = st.discovery.get_device_with_status(&device_id) {
                                            let _ = app.emit("device-online", &device);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            // Remote didn't respond with Announce — that's OK
                            // At least we sent ours so they know about us
                        }
                    }
                }
                _ => {}
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    debug!("TCP probe completed");
    Ok(())
}
