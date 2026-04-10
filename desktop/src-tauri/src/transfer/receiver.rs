use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tauri::Emitter;
use tracing::{error, info};

use crate::protocol::types::CHUNK_SIZE;
use crate::state::AppState;
use crate::transfer::tracker::{FolderEntry, TransferStatus};

/// Open a receiver port and return it. The actual receiving happens in a spawned task.
pub async fn start_receiving(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
    transfer_id: String,
    file_name: String,
    file_size: u64,
    download_dir: PathBuf,
    is_folder: bool,
) -> crate::error::Result<u16> {
    // Bind to a random available port (with Wi-Fi binding on iOS)
    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);

    #[cfg(target_os = "ios")]
    let listener = {
        use socket2::{Domain, Protocol, Socket, Type};
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        socket.set_reuse_address(true)?;
        crate::net_util::bind_socket_to_wifi(&socket).ok();
        socket.bind(&bind_addr.into())?;
        socket.listen(128)?;
        socket.set_nonblocking(true)?;
        let std_listener: std::net::TcpListener = socket.into();
        TcpListener::from_std(std_listener)?
    };

    #[cfg(not(target_os = "ios"))]
    let listener = TcpListener::bind(bind_addr).await?;

    let port = listener.local_addr()?.port();

    info!(
        "Receiver listening on port {} for transfer {} (folder={})",
        port, transfer_id, is_folder
    );

    let resume_offset = if !is_folder {
        // Check for existing .part file for resume (single file only)
        let part_path = download_dir.join(format!("{}.part", file_name));
        if part_path.exists() {
            fs::metadata(&part_path).await?.len()
        } else {
            0
        }
    } else {
        0
    };

    // Update task with receiver port and resume info
    {
        let mut st = state.write().await;
        if let Some(task) = st.transfers.get_task_mut(&transfer_id) {
            task.receiver_port = Some(port);
            task.resume_offset = resume_offset;
            if is_folder {
                task.file_path = download_dir.join(&file_name).to_string_lossy().to_string();
            } else {
                task.file_path = download_dir.join(&file_name).to_string_lossy().to_string();
            }
            task.status = TransferStatus::Active;
        }
    }

    // Spawn the actual receiving task
    let tid = transfer_id.clone();
    tauri::async_runtime::spawn(async move {
        let result = if is_folder {
            do_receive_folder(
                state.clone(),
                app_handle.clone(),
                &tid,
                &file_name,
                file_size,
                download_dir,
                listener,
            )
            .await
        } else {
            do_receive(
                state.clone(),
                app_handle.clone(),
                &tid,
                &file_name,
                file_size,
                download_dir,
                listener,
                resume_offset,
            )
            .await
        };

        if let Err(e) = result {
            error!("File receive failed for {}: {}", tid, e);
            let mut st = state.write().await;
            st.transfers.set_status(&tid, TransferStatus::Failed);
            let task = st.transfers.get_task(&tid).cloned();
            drop(st);
            if let Some(task) = task {
                let _ = app_handle.emit("transfer-update", &task);
            }
        }
    });

    Ok(port)
}

async fn do_receive(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
    transfer_id: &str,
    file_name: &str,
    file_size: u64,
    download_dir: PathBuf,
    listener: TcpListener,
    resume_offset: u64,
) -> crate::error::Result<()> {
    // Wait for sender to connect (with timeout)
    let (mut stream, peer_addr) = tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        listener.accept(),
    )
    .await
    .map_err(|_| crate::error::PeacockError::Transfer("Sender connection timeout".into()))?
    .map_err(|e| crate::error::PeacockError::Transfer(format!("Accept failed: {}", e)))?;

    info!("Sender connected from {} for {}", peer_addr, transfer_id);

    fs::create_dir_all(&download_dir).await?;
    let part_path = download_dir.join(format!("{}.part", file_name));
    let final_path = download_dir.join(file_name);

    // Open file for append (resume) or create new
    let mut file = if resume_offset > 0 {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(&part_path)
            .await?
    } else {
        File::create(&part_path).await?
    };

    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut total_received = resume_offset;
    let start_time = Instant::now();
    let mut last_report = Instant::now();

    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break; // Sender closed connection
        }

        file.write_all(&buf[..n]).await?;
        total_received += n as u64;

        // Throttled progress reporting
        if last_report.elapsed().as_millis() >= 100 {
            let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
            let speed = ((total_received - resume_offset) as f64 / elapsed) as u64;

            {
                let mut st = state.write().await;
                st.transfers
                    .update_progress(transfer_id, total_received, speed);
            }

            let _ = app_handle.emit(
                "transfer-progress",
                serde_json::json!({
                    "transfer_id": transfer_id,
                    "transferred_bytes": total_received,
                    "speed_bps": speed,
                    "file_size": file_size,
                }),
            );
            last_report = Instant::now();
        }
    }

    file.flush().await?;

    // Rename .part to final name
    let final_path = get_unique_path(&final_path).await;
    fs::rename(&part_path, &final_path).await?;

    // Mark completed
    {
        let mut st = state.write().await;
        st.transfers
            .update_progress(transfer_id, file_size, 0);
        st.transfers
            .set_status(transfer_id, TransferStatus::Completed);
        if let Some(task) = st.transfers.get_task_mut(transfer_id) {
            task.file_path = final_path.to_string_lossy().to_string();
        }
        let task = st.transfers.get_task(transfer_id).cloned();
        drop(st);
        if let Some(task) = task {
            let _ = app_handle.emit("transfer-update", &task);
        }
    }

    info!(
        "File receive completed: {} -> {:?}",
        transfer_id, final_path
    );
    Ok(())
}

/// Receive a folder: read manifest, then receive files in order
async fn do_receive_folder(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
    transfer_id: &str,
    folder_name: &str,
    total_size: u64,
    download_dir: PathBuf,
    listener: TcpListener,
) -> crate::error::Result<()> {
    let (mut stream, peer_addr) = tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        listener.accept(),
    )
    .await
    .map_err(|_| crate::error::PeacockError::Transfer("Sender connection timeout".into()))?
    .map_err(|e| crate::error::PeacockError::Transfer(format!("Accept failed: {}", e)))?;

    info!("Sender connected from {} for folder {}", peer_addr, transfer_id);

    // 1. Read manifest: [manifest_len: u64][manifest_json]
    let mut len_buf = [0u8; 8];
    stream.read_exact(&mut len_buf).await?;
    let manifest_len = u64::from_le_bytes(len_buf) as usize;

    let mut manifest_buf = vec![0u8; manifest_len];
    stream.read_exact(&mut manifest_buf).await?;

    let manifest: Vec<FolderEntry> = serde_json::from_slice(&manifest_buf)
        .map_err(|e| crate::error::PeacockError::Transfer(format!("Bad manifest: {}", e)))?;

    info!("Folder manifest: {} files", manifest.len());

    // Create the folder
    let folder_path = download_dir.join(folder_name);
    let folder_path = get_unique_path(&folder_path).await;
    fs::create_dir_all(&folder_path).await?;

    // 2. Receive each file
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut total_received: u64 = 0;
    let start_time = Instant::now();
    let mut last_report = Instant::now();

    for entry in &manifest {
        let file_path = folder_path.join(&entry.relative_path);

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut file = File::create(&file_path).await?;
        let mut remaining = entry.size;

        while remaining > 0 {
            let to_read = (remaining as usize).min(CHUNK_SIZE);
            let n = stream.read(&mut buf[..to_read]).await?;
            if n == 0 {
                return Err(crate::error::PeacockError::Transfer(
                    "Connection closed before all data received".into(),
                ));
            }

            file.write_all(&buf[..n]).await?;
            remaining -= n as u64;
            total_received += n as u64;

            if last_report.elapsed().as_millis() >= 100 {
                let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                let speed = (total_received as f64 / elapsed) as u64;

                {
                    let mut st = state.write().await;
                    st.transfers.update_progress(transfer_id, total_received, speed);
                }

                let _ = app_handle.emit(
                    "transfer-progress",
                    serde_json::json!({
                        "transfer_id": transfer_id,
                        "transferred_bytes": total_received,
                        "speed_bps": speed,
                        "file_size": total_size,
                    }),
                );
                last_report = Instant::now();
            }
        }

        file.flush().await?;
    }

    // Mark completed
    {
        let mut st = state.write().await;
        st.transfers.update_progress(transfer_id, total_size, 0);
        st.transfers.set_status(transfer_id, TransferStatus::Completed);
        if let Some(task) = st.transfers.get_task_mut(transfer_id) {
            task.file_path = folder_path.to_string_lossy().to_string();
        }
        let task = st.transfers.get_task(transfer_id).cloned();
        drop(st);
        if let Some(task) = task {
            let _ = app_handle.emit("transfer-update", &task);
        }
    }

    info!("Folder receive completed: {} -> {:?}", transfer_id, folder_path);
    Ok(())
}

/// If file/dir already exists, add (1), (2) etc.
async fn get_unique_path(path: &PathBuf) -> PathBuf {
    if !path.exists() {
        return path.clone();
    }

    let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let parent = path.parent().unwrap();

    let mut i = 1;
    loop {
        let new_path = if path.is_dir() || ext.is_empty() {
            parent.join(format!("{}({})", stem, i))
        } else {
            parent.join(format!("{}({}){}", stem, i, ext))
        };
        if !new_path.exists() {
            return new_path;
        }
        i += 1;
    }
}
