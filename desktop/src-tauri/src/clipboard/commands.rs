use crate::error::PeacockError;

// Phase 4 stubs

#[tauri::command]
pub async fn enable_clipboard_sync(
    _enabled: bool,
) -> Result<(), PeacockError> {
    Err(PeacockError::General("Clipboard sync not yet implemented".into()))
}

#[tauri::command]
pub async fn push_clipboard(
    _device_id: String,
) -> Result<(), PeacockError> {
    Err(PeacockError::General("Clipboard sync not yet implemented".into()))
}
