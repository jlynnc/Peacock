mod clipboard;
mod discovery;
mod error;
mod messaging;
mod protocol;
mod state;
mod storage;
mod transfer;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;
use tracing::info;

use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "peacock_lib=info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(tauri::generate_handler![
            // Discovery
            discovery::commands::get_online_devices,
            discovery::commands::get_self_info,
            // Messaging
            messaging::commands::send_message,
            messaging::commands::get_message_history,
            // Transfer (Phase 3 stubs)
            transfer::commands::send_file,
            transfer::commands::send_folder,
            transfer::commands::send_paths,
            transfer::commands::accept_transfer,
            transfer::commands::accept_transfer_to_dir,
            transfer::commands::get_download_dir,
            transfer::commands::reject_transfer,
            transfer::commands::pause_transfer,
            transfer::commands::resume_transfer,
            transfer::commands::cancel_transfer,
            transfer::commands::get_active_transfers,
            // Clipboard (Phase 4 stubs)
            clipboard::commands::enable_clipboard_sync,
            clipboard::commands::push_clipboard,
            // Settings
            settings_commands::update_device_name,
            settings_commands::update_download_dir,
            // File utilities
            file_commands::open_file_location,
            file_commands::delete_file,
            // Snippets
            snippet_commands::get_snippets,
            snippet_commands::create_snippet,
            snippet_commands::update_snippet,
            snippet_commands::delete_snippet,
            snippet_commands::copy_snippet,
            snippet_commands::share_snippet,
        ])
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");

            info!("Peacock starting, data dir: {:?}", data_dir);

            let app_state = AppState::new(data_dir).expect("Failed to initialize app state");

            info!(
                "Device: {} ({}), IP: {}, Platform: {}",
                app_state.device_name, app_state.device_id, app_state.ip_addr, app_state.platform
            );

            let state = Arc::new(RwLock::new(app_state));

            // Register state for Tauri commands
            app.manage(state.clone());

            let app_handle = app.handle().clone();

            // Spawn background tasks
            discovery::beacon::spawn_beacon(state.clone());
            discovery::listener::spawn_listener(state.clone(), app_handle.clone());
            discovery::probe::spawn_probe(state.clone(), app_handle.clone());
            messaging::server::spawn_server(state.clone(), app_handle.clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// File utility commands
mod file_commands {
    use crate::error::PeacockError;
    use std::path::Path;

    /// Open the folder containing the file in the system file manager
    #[tauri::command]
    pub async fn open_file_location(path: String) -> Result<(), PeacockError> {
        let file_path = Path::new(&path);
        let dir = if file_path.is_dir() {
            file_path.to_path_buf()
        } else {
            file_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| file_path.to_path_buf())
        };

        if !dir.exists() {
            return Err(PeacockError::General(format!(
                "Directory not found: {}",
                dir.display()
            )));
        }

        #[cfg(target_os = "windows")]
        {
            if file_path.exists() && file_path.is_file() {
                // On Windows, use explorer /select to highlight the file
                std::process::Command::new("explorer")
                    .arg("/select,")
                    .arg(&path)
                    .spawn()
                    .map_err(|e| PeacockError::General(format!("Failed to open explorer: {}", e)))?;
            } else {
                std::process::Command::new("explorer")
                    .arg(dir.as_os_str())
                    .spawn()
                    .map_err(|e| PeacockError::General(format!("Failed to open explorer: {}", e)))?;
            }
        }

        #[cfg(target_os = "macos")]
        {
            if file_path.exists() && file_path.is_file() {
                std::process::Command::new("open")
                    .arg("-R")
                    .arg(&path)
                    .spawn()
                    .map_err(|e| PeacockError::General(format!("Failed to open Finder: {}", e)))?;
            } else {
                std::process::Command::new("open")
                    .arg(dir.as_os_str())
                    .spawn()
                    .map_err(|e| PeacockError::General(format!("Failed to open Finder: {}", e)))?;
            }
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(dir.as_os_str())
                .spawn()
                .map_err(|e| PeacockError::General(format!("Failed to open file manager: {}", e)))?;
        }

        Ok(())
    }

    /// Delete a received file or directory from disk
    #[tauri::command]
    pub async fn delete_file(path: String) -> Result<(), PeacockError> {
        let file_path = Path::new(&path);
        if !file_path.exists() {
            return Err(PeacockError::General("File not found".into()));
        }
        if file_path.is_dir() {
            std::fs::remove_dir_all(file_path)
                .map_err(|e| PeacockError::General(format!("Failed to delete directory: {}", e)))?;
        } else {
            std::fs::remove_file(file_path)
                .map_err(|e| PeacockError::General(format!("Failed to delete file: {}", e)))?;
        }
        Ok(())
    }
}

// Settings commands defined inline since they're simple
mod settings_commands {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    use crate::error::PeacockError;
    use crate::state::AppState;

    #[tauri::command]
    pub async fn update_device_name(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        name: String,
    ) -> Result<(), PeacockError> {
        let mut state = state.write().await;
        state.device_name = name.clone();
        state.db.set_setting("device_name", &name)?;
        Ok(())
    }

    #[tauri::command]
    pub async fn update_download_dir(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        path: String,
    ) -> Result<(), PeacockError> {
        let mut state = state.write().await;
        state.db.set_setting("download_dir", &path)?;
        state.download_dir = std::path::PathBuf::from(&path);
        Ok(())
    }
}

mod snippet_commands {
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    use crate::error::PeacockError;
    use crate::messaging::client::send_to_device;
    use crate::protocol::types::{PacketType, SnippetSharePayload};
    use crate::state::AppState;

    #[tauri::command]
    pub async fn get_snippets(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
    ) -> Result<Vec<serde_json::Value>, PeacockError> {
        let state = state.read().await;
        let snippets = state.db.get_all_snippets()?;
        Ok(snippets)
    }

    #[tauri::command]
    pub async fn create_snippet(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        title: String,
        content: String,
        tag: String,
        note: String,
    ) -> Result<String, PeacockError> {
        let id = uuid::Uuid::new_v4().to_string();
        let state = state.read().await;
        state.db.create_snippet(&id, &title, &content, &tag, &note)?;
        Ok(id)
    }

    #[tauri::command]
    pub async fn update_snippet(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        id: String,
        title: String,
        content: String,
        tag: String,
        note: String,
    ) -> Result<(), PeacockError> {
        let state = state.read().await;
        state.db.update_snippet(&id, &title, &content, &tag, &note)?;
        Ok(())
    }

    #[tauri::command]
    pub async fn delete_snippet(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        id: String,
    ) -> Result<(), PeacockError> {
        let state = state.read().await;
        state.db.delete_snippet(&id)?;
        Ok(())
    }

    #[tauri::command]
    pub async fn copy_snippet(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        id: String,
    ) -> Result<(), PeacockError> {
        let state = state.read().await;
        state.db.increment_snippet_copy_count(&id)?;
        Ok(())
    }

    #[tauri::command]
    pub async fn share_snippet(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        device_id: String,
        title: String,
        content: String,
        tag: String,
        note: String,
    ) -> Result<(), PeacockError> {
        let (target_addr, self_device_id_bytes) = {
            let state = state.read().await;
            let device = state
                .discovery
                .get_device(&device_id)
                .ok_or_else(|| PeacockError::DeviceNotFound(device_id.clone()))?;
            let addr: SocketAddr = format!("{}:{}", device.ip_addr, device.tcp_port)
                .parse()
                .map_err(|e| PeacockError::Network(format!("Invalid address: {}", e)))?;
            (addr, state.device_id_bytes)
        };

        let payload = SnippetSharePayload {
            title,
            content,
            tag,
            note,
        };

        send_to_device(target_addr, PacketType::SnippetShare, &self_device_id_bytes, &payload)
            .await?;
        Ok(())
    }
}
