use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{debug, error, info, warn};

use crate::messaging::handler;
use crate::protocol::types::MESSAGING_PORT;
use crate::protocol::wire::read_packet;
use crate::state::AppState;

pub fn spawn_server(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        loop {
            match run_server_loop(&state, &app_handle).await {
                Ok(()) => break,
                Err(e) => {
                    warn!("Messaging server failed: {}, restarting in 3s...", e);
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            }
        }
    });
}

async fn run_server_loop(
    state: &Arc<RwLock<AppState>>,
    app_handle: &tauri::AppHandle,
) -> crate::error::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), MESSAGING_PORT);

    #[cfg(target_os = "ios")]
    let listener = {
        use socket2::{Domain, Protocol, Socket, Type};
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        socket.set_reuse_address(true)?;
        crate::net_util::bind_socket_to_wifi(&socket).ok();
        socket.bind(&addr.into())?;
        socket.listen(128)?;
        socket.set_nonblocking(true)?;
        let std_listener: std::net::TcpListener = socket.into();
        TcpListener::from_std(std_listener)?
    };

    #[cfg(not(target_os = "ios"))]
    let listener = TcpListener::bind(addr).await?;

    info!("Messaging server started on TCP port {}", MESSAGING_PORT);

    let mut consecutive_errors = 0u32;

    loop {
        match listener.accept().await {
            Ok((mut stream, peer_addr)) => {
                consecutive_errors = 0;
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
                consecutive_errors += 1;
                error!("TCP accept error: {}", e);
                if consecutive_errors >= 10 {
                    return Err(crate::error::PeacockError::Network(
                        format!("Server socket dead after {} errors: {}", consecutive_errors, e),
                    ));
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}
