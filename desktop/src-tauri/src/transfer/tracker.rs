use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderEntry {
    pub relative_path: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTask {
    pub transfer_id: String,
    pub device_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub transferred_bytes: u64,
    pub status: TransferStatus,
    pub direction: TransferDirection,
    pub speed_bps: u64,
    pub is_folder: bool,
    pub file_count: u32,
    pub created_at: u64,
    /// For receiver: the temporary port opened to receive data
    pub receiver_port: Option<u16>,
    /// For resume: the offset to start from
    pub resume_offset: u64,
    /// For folder transfers: manifest of files inside the folder
    #[serde(default)]
    pub folder_manifest: Vec<FolderEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferStatus {
    Pending,
    Active,
    Paused,
    Completed,
    Failed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferDirection {
    Send,
    Receive,
}

#[derive(Debug)]
pub struct TransferManager {
    tasks: HashMap<String, TransferTask>,
}

impl TransferManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub fn create_send_task(
        &mut self,
        transfer_id: String,
        device_id: String,
        file_name: String,
        file_path: String,
        file_size: u64,
        is_folder: bool,
        file_count: u32,
    ) -> &TransferTask {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let task = TransferTask {
            transfer_id: transfer_id.clone(),
            device_id,
            file_name,
            file_path,
            file_size,
            transferred_bytes: 0,
            status: TransferStatus::Pending,
            direction: TransferDirection::Send,
            speed_bps: 0,
            is_folder,
            file_count,
            created_at: now,
            receiver_port: None,
            resume_offset: 0,
            folder_manifest: Vec::new(),
        };

        self.tasks.insert(transfer_id.clone(), task);
        self.tasks.get(&transfer_id).unwrap()
    }

    pub fn create_receive_task(
        &mut self,
        transfer_id: String,
        device_id: String,
        file_name: String,
        file_size: u64,
        is_folder: bool,
        file_count: u32,
    ) -> &TransferTask {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let task = TransferTask {
            transfer_id: transfer_id.clone(),
            device_id,
            file_name,
            file_path: String::new(), // Set when accepted
            file_size,
            transferred_bytes: 0,
            status: TransferStatus::Pending,
            direction: TransferDirection::Receive,
            speed_bps: 0,
            is_folder,
            file_count,
            created_at: now,
            receiver_port: None,
            resume_offset: 0,
            folder_manifest: Vec::new(),
        };

        self.tasks.insert(transfer_id.clone(), task);
        self.tasks.get(&transfer_id).unwrap()
    }

    pub fn get_task(&self, transfer_id: &str) -> Option<&TransferTask> {
        self.tasks.get(transfer_id)
    }

    pub fn get_task_mut(&mut self, transfer_id: &str) -> Option<&mut TransferTask> {
        self.tasks.get_mut(transfer_id)
    }

    pub fn update_progress(&mut self, transfer_id: &str, transferred: u64, speed: u64) {
        if let Some(task) = self.tasks.get_mut(transfer_id) {
            task.transferred_bytes = transferred;
            task.speed_bps = speed;
        }
    }

    pub fn set_status(&mut self, transfer_id: &str, status: TransferStatus) {
        if let Some(task) = self.tasks.get_mut(transfer_id) {
            task.status = status;
        }
    }

    pub fn get_active_tasks(&self) -> Vec<TransferTask> {
        self.tasks
            .values()
            .filter(|t| {
                matches!(
                    t.status,
                    TransferStatus::Active | TransferStatus::Pending | TransferStatus::Paused
                )
            })
            .cloned()
            .collect()
    }

    pub fn get_all_tasks(&self) -> Vec<TransferTask> {
        self.tasks.values().cloned().collect()
    }
}
