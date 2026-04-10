use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::error::PeacockError;
use crate::messaging::client::send_to_device;
use crate::protocol::types::{FileOfferPayload, FileAcceptPayload, FileRejectPayload, PacketType};
use crate::state::AppState;
use crate::transfer::tracker::{FolderEntry, TransferStatus};

/// Walk a directory recursively and collect all files with relative paths
fn walk_dir(base: &Path, current: &Path) -> Result<Vec<FolderEntry>, PeacockError> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            entries.extend(walk_dir(base, &path)?);
        } else if path.is_file() {
            let relative = path.strip_prefix(base)
                .map_err(|e| PeacockError::General(format!("Path error: {}", e)))?;
            let size = std::fs::metadata(&path)?.len();
            entries.push(FolderEntry {
                relative_path: relative.to_string_lossy().replace('\\', "/"),
                size,
            });
        }
    }
    Ok(entries)
}

#[tauri::command]
pub async fn send_file(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    device_id: String,
    file_path: String,
) -> Result<String, PeacockError> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(PeacockError::Transfer(format!("File not found: {}", file_path)));
    }

    let metadata = std::fs::metadata(&file_path)?;
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let file_size = metadata.len();

    let transfer_id = uuid::Uuid::new_v4().to_string();

    let (target_addr, self_device_id_bytes) = {
        let mut st = state.write().await;
        let device = st
            .discovery
            .get_device(&device_id)
            .ok_or_else(|| PeacockError::DeviceNotFound(device_id.clone()))?;

        let addr: SocketAddr = format!("{}:{}", device.ip_addr, device.tcp_port)
            .parse()
            .map_err(|e| PeacockError::Network(format!("Invalid address: {}", e)))?;

        st.transfers.create_send_task(
            transfer_id.clone(),
            device_id.clone(),
            file_name.clone(),
            file_path.clone(),
            file_size,
            false,
            1,
        );

        (addr, st.device_id_bytes)
    };

    let payload = FileOfferPayload {
        transfer_id: transfer_id.clone(),
        file_name,
        file_size,
        is_folder: false,
        file_count: 1,
    };

    info!("Sending file offer {} to {}", transfer_id, device_id);
    send_to_device(target_addr, PacketType::FileOffer, &self_device_id_bytes, &payload).await?;

    Ok(transfer_id)
}

/// Send a file from raw bytes (for iOS where we get File blobs, not file paths)
#[tauri::command]
pub async fn send_file_bytes(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    device_id: String,
    file_name: String,
    file_data: Vec<u8>,
) -> Result<String, PeacockError> {
    // Write bytes to a temp file
    let temp_dir = {
        let st = state.read().await;
        st.data_dir.join("temp_send")
    };
    std::fs::create_dir_all(&temp_dir)?;
    let temp_path = temp_dir.join(&file_name);
    std::fs::write(&temp_path, &file_data)?;

    let file_size = file_data.len() as u64;
    let transfer_id = uuid::Uuid::new_v4().to_string();
    let file_path_str = temp_path.to_string_lossy().to_string();

    let (target_addr, self_device_id_bytes) = {
        let mut st = state.write().await;
        let device = st
            .discovery
            .get_device(&device_id)
            .ok_or_else(|| PeacockError::DeviceNotFound(device_id.clone()))?;

        let addr: SocketAddr = format!("{}:{}", device.ip_addr, device.tcp_port)
            .parse()
            .map_err(|e| PeacockError::Network(format!("Invalid address: {}", e)))?;

        st.transfers.create_send_task(
            transfer_id.clone(),
            device_id.clone(),
            file_name.clone(),
            file_path_str,
            file_size,
            false,
            1,
        );

        (addr, st.device_id_bytes)
    };

    let payload = FileOfferPayload {
        transfer_id: transfer_id.clone(),
        file_name,
        file_size,
        is_folder: false,
        file_count: 1,
    };

    info!("Sending file bytes offer {} to {}", transfer_id, device_id);
    send_to_device(target_addr, PacketType::FileOffer, &self_device_id_bytes, &payload).await?;

    Ok(transfer_id)
}

#[tauri::command]
pub async fn send_folder(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    device_id: String,
    folder_path: String,
) -> Result<String, PeacockError> {
    let path = Path::new(&folder_path);
    if !path.exists() || !path.is_dir() {
        return Err(PeacockError::Transfer(format!("Folder not found: {}", folder_path)));
    }

    let folder_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Walk the directory to build manifest
    let manifest = walk_dir(path, path)?;
    if manifest.is_empty() {
        return Err(PeacockError::Transfer("Folder is empty".into()));
    }

    let file_count = manifest.len() as u32;
    let total_size: u64 = manifest.iter().map(|e| e.size).sum();
    let transfer_id = uuid::Uuid::new_v4().to_string();

    let (target_addr, self_device_id_bytes) = {
        let mut st = state.write().await;
        let device = st
            .discovery
            .get_device(&device_id)
            .ok_or_else(|| PeacockError::DeviceNotFound(device_id.clone()))?;

        let addr: SocketAddr = format!("{}:{}", device.ip_addr, device.tcp_port)
            .parse()
            .map_err(|e| PeacockError::Network(format!("Invalid address: {}", e)))?;

        st.transfers.create_send_task(
            transfer_id.clone(),
            device_id.clone(),
            folder_name.clone(),
            folder_path.clone(),
            total_size,
            true,
            file_count,
        );

        // Store manifest in the task
        if let Some(task) = st.transfers.get_task_mut(&transfer_id) {
            task.folder_manifest = manifest;
        }

        (addr, st.device_id_bytes)
    };

    let payload = FileOfferPayload {
        transfer_id: transfer_id.clone(),
        file_name: folder_name,
        file_size: total_size,
        is_folder: true,
        file_count,
    };

    info!("Sending folder offer {} ({} files, {} bytes) to {}",
        transfer_id, file_count, total_size, device_id);
    send_to_device(target_addr, PacketType::FileOffer, &self_device_id_bytes, &payload).await?;

    Ok(transfer_id)
}

/// Send multiple paths (files and/or folders) - used by drag-and-drop
#[tauri::command]
pub async fn send_paths(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    device_id: String,
    paths: Vec<String>,
) -> Result<Vec<String>, PeacockError> {
    let mut transfer_ids = Vec::new();

    for p in &paths {
        let path = Path::new(p);
        if !path.exists() {
            continue;
        }

        if path.is_dir() {
            let folder_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let manifest = walk_dir(path, path)?;
            if manifest.is_empty() {
                continue;
            }

            let file_count = manifest.len() as u32;
            let total_size: u64 = manifest.iter().map(|e| e.size).sum();
            let transfer_id = uuid::Uuid::new_v4().to_string();

            let (target_addr, self_device_id_bytes) = {
                let mut st = state.write().await;
                let device = st
                    .discovery
                    .get_device(&device_id)
                    .ok_or_else(|| PeacockError::DeviceNotFound(device_id.clone()))?;

                let addr: SocketAddr = format!("{}:{}", device.ip_addr, device.tcp_port)
                    .parse()
                    .map_err(|e| PeacockError::Network(format!("Invalid address: {}", e)))?;

                st.transfers.create_send_task(
                    transfer_id.clone(),
                    device_id.clone(),
                    folder_name.clone(),
                    p.clone(),
                    total_size,
                    true,
                    file_count,
                );
                if let Some(task) = st.transfers.get_task_mut(&transfer_id) {
                    task.folder_manifest = manifest;
                }
                (addr, st.device_id_bytes)
            };

            let payload = FileOfferPayload {
                transfer_id: transfer_id.clone(),
                file_name: folder_name,
                file_size: total_size,
                is_folder: true,
                file_count,
            };

            send_to_device(target_addr, PacketType::FileOffer, &self_device_id_bytes, &payload).await?;
            transfer_ids.push(transfer_id);
        } else {
            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let file_size = std::fs::metadata(path)?.len();
            let transfer_id = uuid::Uuid::new_v4().to_string();

            let (target_addr, self_device_id_bytes) = {
                let mut st = state.write().await;
                let device = st
                    .discovery
                    .get_device(&device_id)
                    .ok_or_else(|| PeacockError::DeviceNotFound(device_id.clone()))?;

                let addr: SocketAddr = format!("{}:{}", device.ip_addr, device.tcp_port)
                    .parse()
                    .map_err(|e| PeacockError::Network(format!("Invalid address: {}", e)))?;

                st.transfers.create_send_task(
                    transfer_id.clone(),
                    device_id.clone(),
                    file_name.clone(),
                    p.clone(),
                    file_size,
                    false,
                    1,
                );
                (addr, st.device_id_bytes)
            };

            let payload = FileOfferPayload {
                transfer_id: transfer_id.clone(),
                file_name,
                file_size,
                is_folder: false,
                file_count: 1,
            };

            send_to_device(target_addr, PacketType::FileOffer, &self_device_id_bytes, &payload).await?;
            transfer_ids.push(transfer_id);
        }
    }

    Ok(transfer_ids)
}

#[tauri::command]
pub async fn accept_transfer(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    app_handle: tauri::AppHandle,
    transfer_id: String,
) -> Result<(), PeacockError> {
    let (device_id, file_name, file_size, download_dir, is_folder) = {
        let st = state.read().await;
        let task = st
            .transfers
            .get_task(&transfer_id)
            .ok_or_else(|| PeacockError::Transfer("Transfer not found".into()))?;
        (
            task.device_id.clone(),
            task.file_name.clone(),
            task.file_size,
            st.download_dir.clone(),
            task.is_folder,
        )
    };

    // Start receiver - opens a random port
    let receiver_port = crate::transfer::receiver::start_receiving(
        state.inner().clone(),
        app_handle,
        transfer_id.clone(),
        file_name,
        file_size,
        download_dir,
        is_folder,
    )
    .await?;

    // Check for resume offset
    let resume_offset = {
        let st = state.read().await;
        st.transfers
            .get_task(&transfer_id)
            .map(|t| t.resume_offset)
            .unwrap_or(0)
    };

    // Send FILE_ACCEPT to the sender
    let (target_addr, self_device_id_bytes) = {
        let st = state.read().await;
        let device = st
            .discovery
            .get_device(&device_id)
            .ok_or_else(|| PeacockError::DeviceNotFound(device_id.clone()))?;

        let addr: SocketAddr = format!("{}:{}", device.ip_addr, device.tcp_port)
            .parse()
            .map_err(|e| PeacockError::Network(format!("Invalid address: {}", e)))?;

        (addr, st.device_id_bytes)
    };

    let accept_payload = FileAcceptPayload {
        transfer_id: transfer_id.clone(),
        receiver_port,
        resume_offset,
    };

    info!(
        "Accepting transfer {} on port {}, resume_offset={}",
        transfer_id, receiver_port, resume_offset
    );

    send_to_device(
        target_addr,
        PacketType::FileAccept,
        &self_device_id_bytes,
        &accept_payload,
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn accept_transfer_to_dir(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    app_handle: tauri::AppHandle,
    transfer_id: String,
    dir: String,
) -> Result<(), PeacockError> {
    let (device_id, file_name, file_size, is_folder) = {
        let st = state.read().await;
        let task = st
            .transfers
            .get_task(&transfer_id)
            .ok_or_else(|| PeacockError::Transfer("Transfer not found".into()))?;
        (
            task.device_id.clone(),
            task.file_name.clone(),
            task.file_size,
            task.is_folder,
        )
    };

    let download_dir = std::path::PathBuf::from(dir);

    let receiver_port = crate::transfer::receiver::start_receiving(
        state.inner().clone(),
        app_handle,
        transfer_id.clone(),
        file_name,
        file_size,
        download_dir,
        is_folder,
    )
    .await?;

    let resume_offset = {
        let st = state.read().await;
        st.transfers
            .get_task(&transfer_id)
            .map(|t| t.resume_offset)
            .unwrap_or(0)
    };

    let (target_addr, self_device_id_bytes) = {
        let st = state.read().await;
        let device = st
            .discovery
            .get_device(&device_id)
            .ok_or_else(|| PeacockError::DeviceNotFound(device_id.clone()))?;
        let addr: SocketAddr = format!("{}:{}", device.ip_addr, device.tcp_port)
            .parse()
            .map_err(|e| PeacockError::Network(format!("Invalid address: {}", e)))?;
        (addr, st.device_id_bytes)
    };

    let accept_payload = FileAcceptPayload {
        transfer_id: transfer_id.clone(),
        receiver_port,
        resume_offset,
    };

    info!(
        "Accepting transfer {} to custom dir, port {}, resume_offset={}",
        transfer_id, receiver_port, resume_offset
    );

    send_to_device(
        target_addr,
        PacketType::FileAccept,
        &self_device_id_bytes,
        &accept_payload,
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn get_download_dir(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<String, PeacockError> {
    let st = state.read().await;
    Ok(st.download_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn reject_transfer(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    transfer_id: String,
) -> Result<(), PeacockError> {
    let device_id = {
        let mut st = state.write().await;
        let task = st
            .transfers
            .get_task(&transfer_id)
            .ok_or_else(|| PeacockError::Transfer("Transfer not found".into()))?;
        let did = task.device_id.clone();
        st.transfers.set_status(&transfer_id, TransferStatus::Rejected);
        did
    };

    let (target_addr, self_device_id_bytes) = {
        let st = state.read().await;
        let device = st
            .discovery
            .get_device(&device_id)
            .ok_or_else(|| PeacockError::DeviceNotFound(device_id.clone()))?;
        let addr: SocketAddr = format!("{}:{}", device.ip_addr, device.tcp_port)
            .parse()
            .map_err(|e| PeacockError::Network(format!("Invalid address: {}", e)))?;
        (addr, st.device_id_bytes)
    };

    let reject_payload = FileRejectPayload {
        transfer_id: transfer_id.clone(),
    };

    send_to_device(
        target_addr,
        PacketType::FileReject,
        &self_device_id_bytes,
        &reject_payload,
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn pause_transfer(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    transfer_id: String,
) -> Result<(), PeacockError> {
    let mut st = state.write().await;
    st.transfers.set_status(&transfer_id, TransferStatus::Paused);
    Ok(())
}

#[tauri::command]
pub async fn resume_transfer(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    transfer_id: String,
) -> Result<(), PeacockError> {
    let mut st = state.write().await;
    st.transfers.set_status(&transfer_id, TransferStatus::Active);
    Ok(())
}

#[tauri::command]
pub async fn cancel_transfer(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    transfer_id: String,
) -> Result<(), PeacockError> {
    let mut st = state.write().await;
    st.transfers.set_status(&transfer_id, TransferStatus::Failed);
    Ok(())
}

#[tauri::command]
pub async fn get_active_transfers(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<Vec<crate::transfer::tracker::TransferTask>, PeacockError> {
    let st = state.read().await;
    Ok(st.transfers.get_all_tasks())
}
