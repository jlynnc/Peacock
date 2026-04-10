use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tauri::Emitter;
use tracing::{error, info};

use crate::protocol::types::CHUNK_SIZE;
use crate::state::AppState;
use crate::transfer::tracker::{FolderEntry, TransferStatus};

/// Start sending a file to the receiver's port
pub async fn start_sending(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
    transfer_id: String,
    file_path: String,
    receiver_addr: SocketAddr,
    resume_offset: u64,
) {
    // Acquire transfer semaphore to limit concurrent transfers
    let semaphore = {
        let st = state.read().await;
        st.transfer_semaphore.clone()
    };

    // Check if this is a folder transfer
    let (is_folder, folder_manifest, base_path) = {
        let st = state.read().await;
        if let Some(task) = st.transfers.get_task(&transfer_id) {
            (task.is_folder, task.folder_manifest.clone(), task.file_path.clone())
        } else {
            (false, Vec::new(), file_path.clone())
        }
    };

    if is_folder && !folder_manifest.is_empty() {
        tauri::async_runtime::spawn(async move {
            let _permit = semaphore.acquire().await.expect("semaphore closed");
            if let Err(e) = do_send_folder(
                state.clone(),
                app_handle.clone(),
                &transfer_id,
                &base_path,
                &folder_manifest,
                receiver_addr,
            )
            .await
            {
                error!("Folder send failed for {}: {}", transfer_id, e);
                let mut st = state.write().await;
                st.transfers.set_status(&transfer_id, TransferStatus::Failed);
                let task = st.transfers.get_task(&transfer_id).cloned();
                drop(st);
                if let Some(task) = task {
                    let _ = app_handle.emit("transfer-update", &task);
                }
            }
        });
    } else {
        tauri::async_runtime::spawn(async move {
            let _permit = semaphore.acquire().await.expect("semaphore closed");
            if let Err(e) = do_send(
                state.clone(),
                app_handle.clone(),
                &transfer_id,
                &file_path,
                receiver_addr,
                resume_offset,
            )
            .await
            {
                error!("File send failed for {}: {}", transfer_id, e);
                let mut st = state.write().await;
                st.transfers.set_status(&transfer_id, TransferStatus::Failed);
                let task = st.transfers.get_task(&transfer_id).cloned();
                drop(st);
                if let Some(task) = task {
                    let _ = app_handle.emit("transfer-update", &task);
                }
            }
        });
    }
}

async fn do_send(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
    transfer_id: &str,
    file_path: &str,
    receiver_addr: SocketAddr,
    resume_offset: u64,
) -> crate::error::Result<()> {
    // Mark as active
    {
        let mut st = state.write().await;
        st.transfers.set_status(transfer_id, TransferStatus::Active);
        if let Some(task) = st.transfers.get_task(transfer_id).cloned() {
            let _ = app_handle.emit("transfer-update", &task);
        }
    }

    let mut file = File::open(file_path).await?;
    let file_size = file.metadata().await?.len();

    if resume_offset > 0 {
        file.seek(SeekFrom::Start(resume_offset)).await?;
    }

    let mut stream = crate::messaging::client::ios_aware_tcp_connect(receiver_addr).await?;
    info!(
        "Sending file to {}, offset={}, size={}",
        receiver_addr, resume_offset, file_size
    );

    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut total_sent = resume_offset;
    let start_time = Instant::now();
    let mut last_report = Instant::now();

    loop {
        // Check if paused/cancelled
        {
            let st = state.read().await;
            if let Some(task) = st.transfers.get_task(transfer_id) {
                match task.status {
                    TransferStatus::Paused => {
                        drop(st);
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                        continue;
                    }
                    TransferStatus::Failed => {
                        return Ok(());
                    }
                    _ => {}
                }
            } else {
                return Ok(());
            }
        }

        let n = file.read(&mut buf).await?;
        if n == 0 {
            break; // EOF
        }

        stream.write_all(&buf[..n]).await?;
        total_sent += n as u64;

        // Throttled progress reporting (every 100ms)
        if last_report.elapsed().as_millis() >= 100 {
            let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
            let speed = ((total_sent - resume_offset) as f64 / elapsed) as u64;

            {
                let mut st = state.write().await;
                st.transfers.update_progress(transfer_id, total_sent, speed);
            }

            let _ = app_handle.emit(
                "transfer-progress",
                serde_json::json!({
                    "transfer_id": transfer_id,
                    "transferred_bytes": total_sent,
                    "speed_bps": speed,
                    "file_size": file_size,
                }),
            );
            last_report = Instant::now();
        }
    }

    stream.flush().await?;
    stream.shutdown().await?;

    // Mark completed
    {
        let mut st = state.write().await;
        st.transfers.update_progress(transfer_id, file_size, 0);
        st.transfers.set_status(transfer_id, TransferStatus::Completed);
        if let Some(task) = st.transfers.get_task(transfer_id).cloned() {
            let _ = app_handle.emit("transfer-update", &task);
        }
    }

    info!("File send completed: {}", transfer_id);
    Ok(())
}

/// Send a folder: write manifest JSON first, then stream all files sequentially
async fn do_send_folder(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
    transfer_id: &str,
    base_path: &str,
    manifest: &[FolderEntry],
    receiver_addr: SocketAddr,
) -> crate::error::Result<()> {
    // Mark as active
    let total_size = {
        let mut st = state.write().await;
        st.transfers.set_status(transfer_id, TransferStatus::Active);
        let size = st.transfers.get_task(transfer_id).map(|t| t.file_size).unwrap_or(0);
        if let Some(task) = st.transfers.get_task(transfer_id).cloned() {
            let _ = app_handle.emit("transfer-update", &task);
        }
        size
    };

    let mut stream = crate::messaging::client::ios_aware_tcp_connect(receiver_addr).await?;
    info!(
        "Sending folder to {}, {} files, {} bytes",
        receiver_addr, manifest.len(), total_size
    );

    // 1. Write manifest as JSON: [manifest_len: u64][manifest_json]
    let manifest_json = serde_json::to_vec(manifest)
        .map_err(|e| crate::error::PeacockError::General(format!("JSON error: {}", e)))?;
    let manifest_len = manifest_json.len() as u64;
    stream.write_all(&manifest_len.to_le_bytes()).await?;
    stream.write_all(&manifest_json).await?;

    // 2. Stream each file's data
    let base = Path::new(base_path);
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut total_sent: u64 = 0;
    let start_time = Instant::now();
    let mut last_report = Instant::now();

    for entry in manifest {
        let file_path = base.join(&entry.relative_path);
        let mut file = File::open(&file_path).await?;

        loop {
            // Check pause/cancel
            {
                let st = state.read().await;
                if let Some(task) = st.transfers.get_task(transfer_id) {
                    match task.status {
                        TransferStatus::Paused => {
                            drop(st);
                            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                            continue;
                        }
                        TransferStatus::Failed => {
                            return Ok(());
                        }
                        _ => {}
                    }
                } else {
                    return Ok(());
                }
            }

            let n = file.read(&mut buf).await?;
            if n == 0 {
                break;
            }

            stream.write_all(&buf[..n]).await?;
            total_sent += n as u64;

            if last_report.elapsed().as_millis() >= 100 {
                let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                let speed = (total_sent as f64 / elapsed) as u64;

                {
                    let mut st = state.write().await;
                    st.transfers.update_progress(transfer_id, total_sent, speed);
                }

                let _ = app_handle.emit(
                    "transfer-progress",
                    serde_json::json!({
                        "transfer_id": transfer_id,
                        "transferred_bytes": total_sent,
                        "speed_bps": speed,
                        "file_size": total_size,
                    }),
                );
                last_report = Instant::now();
            }
        }
    }

    stream.flush().await?;
    stream.shutdown().await?;

    // Mark completed
    {
        let mut st = state.write().await;
        st.transfers.update_progress(transfer_id, total_size, 0);
        st.transfers.set_status(transfer_id, TransferStatus::Completed);
        if let Some(task) = st.transfers.get_task(transfer_id).cloned() {
            let _ = app_handle.emit("transfer-update", &task);
        }
    }

    info!("Folder send completed: {}", transfer_id);
    Ok(())
}
