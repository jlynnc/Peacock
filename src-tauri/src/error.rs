use thiserror::Error;

#[derive(Error, Debug)]
pub enum PeacockError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Transfer error: {0}")]
    Transfer(String),

    #[error("{0}")]
    General(String),
}

// Make it serializable for Tauri IPC
impl serde::Serialize for PeacockError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, PeacockError>;
