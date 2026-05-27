mod clipboard;
mod discovery;
mod error;
mod messaging;
pub mod net_util;
mod protocol;
pub mod state;
mod storage;
mod transfer;

use std::sync::Arc;
use tauri::Manager;
use tauri::Emitter;
#[cfg(desktop)]
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
#[cfg(desktop)]
use tauri::menu::{Menu, MenuItem};
use tokio::sync::RwLock;
use tracing::info;

use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "peacock=info".into()),
        )
        .init();

    let mut builder = tauri::Builder::default();

    // Desktop-only plugins
    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
                // Another instance was launched — bring existing window to front
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
                // Check for --send (legacy) or --send-pending (new: read from temp file)
                if let Some(pos) = argv.iter().position(|a| a == "--send") {
                    if let Some(file_path) = argv.get(pos + 1) {
                        let _ = app.emit("send-file-request", file_path.clone());
                    }
                }
                if argv.iter().any(|a| a == "--send-pending") {
                    let send_file = std::env::temp_dir().join("peacock_send.txt");
                    if let Ok(path) = std::fs::read_to_string(&send_file) {
                        let path = path.trim().to_string();
                        if !path.is_empty() {
                            let _ = app.emit("send-file-request", path);
                        }
                        let _ = std::fs::remove_file(&send_file);
                    }
                }
            }))
            .plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec![]),
            ));
    }

    builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(tauri::generate_handler![
            // Discovery
            discovery::commands::get_online_devices,
            discovery::commands::get_self_info,
            discovery::commands::udp_test,
            // Messaging
            messaging::commands::send_message,
            messaging::commands::get_message_history,
            // Transfer (Phase 3 stubs)
            transfer::commands::send_file,
            transfer::commands::send_file_bytes,
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
            settings_commands::set_max_concurrent,
            // File utilities
            file_commands::open_file_location,
            file_commands::delete_file,
            file_commands::check_file_exists,
            // Window utilities (desktop only, but safe no-ops on mobile)
            window_commands::flash_window,
            window_commands::stop_flash,
            context_menu_commands::register_context_menu,
            context_menu_commands::unregister_context_menu,
            context_menu_commands::is_context_menu_registered,
            // Debug
            debug_commands::get_debug_state,
            debug_commands::set_broadcast_enabled,
            // Rooms
            room_commands::create_room,
            room_commands::get_rooms,
            room_commands::delete_room,
            room_commands::send_room_message,
            room_commands::send_room_file,
            // Snippets
            snippet_commands::get_snippets,
            snippet_commands::create_snippet,
            snippet_commands::update_snippet,
            snippet_commands::delete_snippet,
            snippet_commands::copy_snippet,
            snippet_commands::share_snippet,
            snippet_commands::reorder_snippets,
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

            // ── Handle --send / --send-pending on first launch ──
            #[cfg(desktop)]
            {
                let args: Vec<String> = std::env::args().collect();
                let mut send_path: Option<String> = None;

                // Legacy --send <path>
                if let Some(pos) = args.iter().position(|a| a == "--send") {
                    if let Some(file_path) = args.get(pos + 1) {
                        send_path = Some(file_path.clone());
                    }
                }
                // New --send-pending: read path from temp file
                if args.iter().any(|a| a == "--send-pending") {
                    let send_file = std::env::temp_dir().join("peacock_send.txt");
                    if let Ok(path) = std::fs::read_to_string(&send_file) {
                        let path = path.trim().to_string();
                        if !path.is_empty() {
                            send_path = Some(path);
                        }
                        let _ = std::fs::remove_file(&send_file);
                    }
                }

                if let Some(fp) = send_path {
                    let handle = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        let _ = handle.emit("send-file-request", fp);
                    });
                }
            }

            // ── System tray ──
            #[cfg(desktop)]
            {
                let show = MenuItem::with_id(app, "show", "显示 Peacock", true, None::<&str>)?;
                let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show, &quit])?;

                let _tray = TrayIconBuilder::with_id("main")
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Minimize to tray on close instead of quitting
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                #[cfg(desktop)]
                {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
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

    /// Check if a file or directory exists
    #[tauri::command]
    pub async fn check_file_exists(path: String) -> Result<bool, PeacockError> {
        Ok(Path::new(&path).exists())
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
    pub async fn set_max_concurrent(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        max: usize,
    ) -> Result<(), PeacockError> {
        let mut state = state.write().await;
        state.transfer_semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max));
        state.db.set_setting("max_concurrent", &max.to_string())?;
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
    pub async fn reorder_snippets(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        ids: Vec<String>,
    ) -> Result<(), PeacockError> {
        let state = state.read().await;
        state.db.reorder_snippets(&ids)?;
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

mod window_commands {
    #[cfg(desktop)]
    use tauri::Manager;

    #[cfg(desktop)]
    use std::sync::atomic::{AtomicBool, Ordering};

    #[cfg(desktop)]
    static TRAY_FLASHING: AtomicBool = AtomicBool::new(false);

    /// Flash the taskbar icon and tray to alert the user
    #[tauri::command]
    pub async fn flash_window(#[allow(unused)] app: tauri::AppHandle) -> Result<(), String> {
        #[cfg(desktop)]
        {
            use tauri::image::Image;

            // Flash taskbar
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.request_user_attention(Some(tauri::UserAttentionType::Informational));
            }

            // Start continuous tray flash if not already flashing
            if TRAY_FLASHING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    let normal_icon = app_clone.default_window_icon().cloned();
                    let mut red_dot = vec![0u8; 16 * 16 * 4];
                    for y in 0..16u32 {
                        for x in 0..16u32 {
                            let idx = ((y * 16 + x) * 4) as usize;
                            let dx = x as f32 - 8.0;
                            let dy = y as f32 - 8.0;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist < 7.0 {
                                red_dot[idx] = 239;
                                red_dot[idx + 1] = 68;
                                red_dot[idx + 2] = 68;
                                red_dot[idx + 3] = 255;
                            }
                        }
                    }
                    let alert_icon = Image::new_owned(red_dot, 16, 16);

                    while TRAY_FLASHING.load(Ordering::SeqCst) {
                        if let Some(tray) = app_clone.tray_by_id("main") {
                            let _ = tray.set_icon(Some(alert_icon.clone()));
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                        if !TRAY_FLASHING.load(Ordering::SeqCst) { break; }
                        if let Some(ref normal) = normal_icon {
                            if let Some(tray) = app_clone.tray_by_id("main") {
                                let _ = tray.set_icon(Some(normal.clone()));
                            }
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                    }

                    if let Some(ref normal) = normal_icon {
                        if let Some(tray) = app_clone.tray_by_id("main") {
                            let _ = tray.set_icon(Some(normal.clone()));
                        }
                    }
                });
            }
        }
        Ok(())
    }

    /// Stop tray flashing
    #[tauri::command]
    pub async fn stop_flash(#[allow(unused)] app: tauri::AppHandle) -> Result<(), String> {
        #[cfg(desktop)]
        {
            TRAY_FLASHING.store(false, Ordering::SeqCst);
            if let Some(normal) = app.default_window_icon().cloned() {
                if let Some(tray) = app.tray_by_id("main") {
                    let _ = tray.set_icon(Some(normal));
                }
            }
        }
        Ok(())
    }
}

mod context_menu_commands {
    /// Register Windows Explorer right-click "Send to Peacock" menu
    #[tauri::command]
    pub async fn register_context_menu() -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let exe_path = std::env::current_exe()
                .map_err(|e| format!("Failed to get exe path: {}", e))?;
            let exe_str = exe_path.to_string_lossy().replace("\\", "\\\\");

            // Register for files: HKCU\Software\Classes\*\shell\Peacock
            let output = std::process::Command::new("reg")
                .args(["add", r"HKCU\Software\Classes\*\shell\Peacock", "/ve", "/d", "发送到 Peacock", "/f"])
                .output()
                .map_err(|e| format!("reg add failed: {}", e))?;
            if !output.status.success() {
                return Err("Failed to register file context menu".into());
            }

            // Set icon
            std::process::Command::new("reg")
                .args(["add", r"HKCU\Software\Classes\*\shell\Peacock", "/v", "Icon", "/d", &exe_str, "/f"])
                .output()
                .ok();

            // Set command
            let cmd = format!("\"{}\" --send \"%1\"", exe_path.to_string_lossy());
            std::process::Command::new("reg")
                .args(["add", r"HKCU\Software\Classes\*\shell\Peacock\command", "/ve", "/d", &cmd, "/f"])
                .output()
                .map_err(|e| format!("reg add command failed: {}", e))?;

            // Register for folders: HKCU\Software\Classes\Directory\shell\Peacock
            std::process::Command::new("reg")
                .args(["add", r"HKCU\Software\Classes\Directory\shell\Peacock", "/ve", "/d", "发送到 Peacock", "/f"])
                .output()
                .ok();

            std::process::Command::new("reg")
                .args(["add", r"HKCU\Software\Classes\Directory\shell\Peacock", "/v", "Icon", "/d", &exe_str, "/f"])
                .output()
                .ok();

            let folder_cmd = format!("\"{}\" --send \"%1\"", exe_path.to_string_lossy());
            std::process::Command::new("reg")
                .args(["add", r"HKCU\Software\Classes\Directory\shell\Peacock\command", "/ve", "/d", &folder_cmd, "/f"])
                .output()
                .ok();
        }

        Ok(())
    }

    /// Unregister the right-click context menu
    #[tauri::command]
    pub async fn unregister_context_menu() -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("reg")
                .args(["delete", r"HKCU\Software\Classes\*\shell\Peacock", "/f"])
                .output()
                .ok();
            std::process::Command::new("reg")
                .args(["delete", r"HKCU\Software\Classes\Directory\shell\Peacock", "/f"])
                .output()
                .ok();
        }
        Ok(())
    }

    /// Check if context menu is registered
    #[tauri::command]
    pub async fn is_context_menu_registered() -> Result<bool, String> {
        #[cfg(target_os = "windows")]
        {
            let output = std::process::Command::new("reg")
                .args(["query", r"HKCU\Software\Classes\*\shell\Peacock"])
                .output()
                .map_err(|e| format!("reg query failed: {}", e))?;
            return Ok(output.status.success());
        }
        #[cfg(not(target_os = "windows"))]
        Ok(false)
    }
}

mod room_commands {
    use std::net::SocketAddr;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::sync::RwLock;
    use tracing::info;

    use crate::error::PeacockError;
    use crate::messaging::client::send_to_device;
    use crate::protocol::types::*;
    use crate::state::AppState;

    #[tauri::command]
    pub async fn create_room(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        room_name: String,
        member_ids: Vec<String>,
    ) -> Result<String, PeacockError> {
        let room_id = uuid::Uuid::new_v4().to_string();

        let (self_id, self_id_bytes) = {
            let st = state.read().await;
            // Save to local DB
            let mut all_members = member_ids.clone();
            if !all_members.contains(&st.device_id) {
                all_members.push(st.device_id.clone());
            }
            st.db.create_room(&room_id, &room_name, &all_members)?;
            (st.device_id.clone(), st.device_id_bytes)
        };

        // Build full member list including self
        let mut all_members = member_ids.clone();
        if !all_members.contains(&self_id) {
            all_members.push(self_id.clone());
        }

        let payload = RoomCreatePayload {
            room_id: room_id.clone(),
            room_name: room_name.clone(),
            member_ids: all_members,
        };

        // Send to each member
        let st = state.read().await;
        for mid in &member_ids {
            if mid == &self_id { continue; }
            if let Some(device) = st.discovery.get_device(mid) {
                let addr: SocketAddr = format!("{}:{}", device.ip_addr, device.tcp_port)
                    .parse()
                    .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)));
                let _ = send_to_device(addr, PacketType::RoomCreate, &self_id_bytes, &payload).await;
            }
        }

        info!("Created room {} with {} members", room_name, member_ids.len() + 1);
        Ok(room_id)
    }

    #[tauri::command]
    pub async fn get_rooms(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
    ) -> Result<Vec<serde_json::Value>, PeacockError> {
        let st = state.read().await;
        st.db.get_rooms()
    }

    #[tauri::command]
    pub async fn delete_room(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        room_id: String,
    ) -> Result<(), PeacockError> {
        let st = state.read().await;
        st.db.delete_room(&room_id)
    }

    #[tauri::command]
    pub async fn send_room_message(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        room_id: String,
        text: String,
    ) -> Result<String, PeacockError> {
        let message_id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let (self_id, self_name, self_id_bytes, members) = {
            let st = state.read().await;
            let rooms = st.db.get_rooms()?;
            let room = rooms.iter()
                .find(|r| r["id"].as_str() == Some(&room_id))
                .ok_or_else(|| PeacockError::General("Room not found".into()))?;
            let member_ids: Vec<String> = room["member_ids"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            (st.device_id.clone(), st.device_name.clone(), st.device_id_bytes, member_ids)
        };

        let payload = RoomMessagePayload {
            room_id: room_id.clone(),
            message_id: message_id.clone(),
            sender_id: self_id.clone(),
            sender_name: self_name,
            text,
            timestamp: now,
        };

        // Send to each member except self
        let st = state.read().await;
        for mid in &members {
            if mid == &self_id { continue; }
            if let Some(device) = st.discovery.get_device(mid) {
                let addr: SocketAddr = format!("{}:{}", device.ip_addr, device.tcp_port)
                    .parse()
                    .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)));
                let _ = send_to_device(addr, PacketType::RoomMessage, &self_id_bytes, &payload).await;
            }
        }

        Ok(message_id)
    }

    #[tauri::command]
    pub async fn send_room_file(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        app_handle: tauri::AppHandle,
        room_id: String,
        file_path: String,
    ) -> Result<String, PeacockError> {
        let transfer_id = uuid::Uuid::new_v4().to_string();
        let path = std::path::Path::new(&file_path);
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.clone());

        let (file_size, is_folder, file_count) = if path.is_dir() {
            let mut count = 0u32;
            let mut size = 0u64;
            fn walk(dir: &std::path::Path, count: &mut u32, size: &mut u64) {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            walk(&p, count, size);
                        } else {
                            *count += 1;
                            *size += p.metadata().map(|m| m.len()).unwrap_or(0);
                        }
                    }
                }
            }
            walk(path, &mut count, &mut size);
            (size, true, count)
        } else {
            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
            (size, false, 1)
        };

        let (self_id, self_name, self_id_bytes, members) = {
            let st = state.read().await;
            let rooms = st.db.get_rooms()?;
            let room = rooms.iter()
                .find(|r| r["id"].as_str() == Some(&room_id))
                .ok_or_else(|| PeacockError::General("Room not found".into()))?;
            let member_ids: Vec<String> = room["member_ids"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            // Create send task
            let mut st_w = state.write().await;
            st_w.transfers.create_send_task(
                transfer_id.clone(),
                "room".to_string(), // room transfer
                file_name.clone(),
                file_path.clone(),
                file_size,
                is_folder,
                file_count,
            );

            (st_w.device_id.clone(), st_w.device_name.clone(), st_w.device_id_bytes, member_ids)
        };

        let offer = RoomFileOfferPayload {
            room_id,
            transfer_id: transfer_id.clone(),
            sender_id: self_id.clone(),
            sender_name: self_name,
            file_name,
            file_size,
            is_folder,
            file_count,
        };

        // Send offer to each member
        let st = state.read().await;
        for mid in &members {
            if mid == &self_id { continue; }
            if let Some(device) = st.discovery.get_device(mid) {
                let addr: SocketAddr = format!("{}:{}", device.ip_addr, device.tcp_port)
                    .parse()
                    .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)));
                let _ = send_to_device(addr, PacketType::RoomFileOffer, &self_id_bytes, &offer).await;
            }
        }

        Ok(transfer_id)
    }
}

mod debug_commands {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    use crate::error::PeacockError;
    use crate::state::AppState;

    #[tauri::command]
    pub async fn get_debug_state(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
    ) -> Result<serde_json::Value, PeacockError> {
        let st = state.read().await;
        let devices = st.discovery.get_debug_devices();
        let restricted_peers = st.discovery.get_restricted_peers();
        let rooms = st.db.get_rooms().unwrap_or_default();

        Ok(serde_json::json!({
            "self": {
                "device_id": st.device_id,
                "device_name": st.device_name,
                "ip_addr": st.ip_addr,
                "platform": st.platform,
            },
            "broadcast_enabled": st.broadcast_enabled,
            "devices": devices,
            "restricted_peers": restricted_peers,
            "rooms": rooms,
        }))
    }

    #[tauri::command]
    pub async fn set_broadcast_enabled(
        state: tauri::State<'_, Arc<RwLock<AppState>>>,
        enabled: bool,
    ) -> Result<(), PeacockError> {
        let mut st = state.write().await;
        st.broadcast_enabled = enabled;
        tracing::info!("Broadcast enabled: {}", enabled);
        Ok(())
    }
}
