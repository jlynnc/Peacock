use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::messaging::handler;
use crate::protocol::types::MESSAGING_PORT;
use crate::protocol::wire::read_packet;
use crate::state::AppState;

/// Spawn the TCP messaging server
pub fn spawn_server(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_server(state, app_handle).await {
            error!("Messaging server failed: {}", e);
        }
    });
}

async fn run_server(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
) -> crate::error::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), MESSAGING_PORT);
    let listener = TcpListener::bind(addr).await?;
    info!("Messaging server started on TCP port {}", MESSAGING_PORT);

    loop {
        match listener.accept().await {
            Ok((mut stream, peer_addr)) => {
                debug!("Incoming TCP connection from {}", peer_addr);
                let state = state.clone();
                let app = app_handle.clone();

                tauri::async_runtime::spawn(async move {
                    match read_packet(&mut stream).await {
                        Ok((header, payload)) => {
                            handler::handle_packet(
                                &state,
                                &app,
                                header,
                                payload,
                                peer_addr,
                                &mut stream,
                            )
                            .await;
                        }
                        Err(e) => {
                            debug!("Failed to read packet from {}: {}", peer_addr, e);
                        }
                    }
                });
            }
            Err(e) => {
                error!("TCP accept error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
}
