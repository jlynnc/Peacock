use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::info;

use crate::error::Result;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Current database schema version. Increment this when adding new migrations.
    const CURRENT_DB_VERSION: i64 = 3;
    pub fn new(data_dir: &PathBuf) -> Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let db_path = data_dir.join("peacock.db");
        info!("Opening database at {:?}", db_path);

        let conn = Connection::open(db_path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_tables()?;
        db.run_migrations()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                device_id TEXT NOT NULL,
                direction TEXT NOT NULL,
                content TEXT NOT NULL,
                msg_type TEXT NOT NULL DEFAULT 'text',
                timestamp INTEGER NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            );

            CREATE INDEX IF NOT EXISTS idx_messages_device ON messages(device_id, timestamp);

            CREATE TABLE IF NOT EXISTS known_devices (
                device_id TEXT PRIMARY KEY,
                device_name TEXT NOT NULL,
                ip_addr TEXT NOT NULL,
                tcp_port INTEGER NOT NULL,
                platform TEXT NOT NULL,
                last_seen INTEGER NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            );

            CREATE TABLE IF NOT EXISTS transfers (
                id TEXT PRIMARY KEY,
                device_id TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                status TEXT NOT NULL,
                direction TEXT NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER DEFAULT (strftime('%s', 'now'))
            );

            CREATE TABLE IF NOT EXISTS clipboard_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_hash TEXT NOT NULL,
                content_preview TEXT NOT NULL,
                source_device TEXT NOT NULL,
                content_type TEXT NOT NULL DEFAULT 'text',
                timestamp INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS snippets (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                tag TEXT NOT NULL DEFAULT '',
                note TEXT NOT NULL DEFAULT '',
                copy_count INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );

            CREATE INDEX IF NOT EXISTS idx_snippets_tag ON snippets(tag);
            CREATE INDEX IF NOT EXISTS idx_snippets_updated ON snippets(updated_at DESC);
            ",
        )?;
        Ok(())
    }

    fn get_db_version(&self) -> i64 {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT value FROM settings WHERE key = 'db_version'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0)
    }

    fn set_db_version(&self, version: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('db_version', ?1)",
            params![version.to_string()],
        )?;
        Ok(())
    }

    fn run_migrations(&self) -> Result<()> {
        let current = self.get_db_version();
        if current >= Self::CURRENT_DB_VERSION {
            info!("Database is up to date (version {})", current);
            return Ok(());
        }

        info!(
            "Running database migrations from version {} to {}",
            current,
            Self::CURRENT_DB_VERSION
        );

        let conn = self.conn.lock().unwrap();

        // Migration 1: Initial schema (baseline for existing databases)
        if current < 1 {
            info!("Migration 1: Setting baseline version");
            // Tables already created by init_tables, just mark version
        }

        // Migration 2: Add sort_order to snippets for drag-drop reordering
        if current < 2 {
            info!("Migration 2: Add sort_order to snippets");
            conn.execute_batch(
                "ALTER TABLE snippets ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;"
            )?;
        }

        // Migration 3: Rooms table for group chat
        if current < 3 {
            info!("Migration 3: Create rooms table");
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS rooms (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    member_ids TEXT NOT NULL,
                    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
                );"
            )?;
        }

        // ── Future migrations go here ──

        drop(conn);
        self.set_db_version(Self::CURRENT_DB_VERSION)?;
        info!("Database migrated to version {}", Self::CURRENT_DB_VERSION);
        Ok(())
    }

    pub fn store_message(
        &self,
        id: &str,
        device_id: &str,
        direction: &str,
        content: &str,
        msg_type: &str,
        timestamp: u64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO messages (id, device_id, direction, content, msg_type, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, device_id, direction, content, msg_type, timestamp],
        )?;
        Ok(())
    }

    pub fn get_messages(
        &self,
        device_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, device_id, direction, content, msg_type, timestamp
             FROM messages
             WHERE device_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2 OFFSET ?3",
        )?;

        let rows = stmt
            .query_map(params![device_id, limit, offset], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "device_id": row.get::<_, String>(1)?,
                    "direction": row.get::<_, String>(2)?,
                    "content": row.get::<_, String>(3)?,
                    "msg_type": row.get::<_, String>(4)?,
                    "timestamp": row.get::<_, u64>(5)?,
                    "status": "sent"
                }))
            })?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();

        // Reverse to get chronological order
        let mut messages = rows;
        messages.reverse();
        Ok(messages)
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let result = stmt
            .query_row(params![key], |row| row.get::<_, String>(0))
            .ok();
        Ok(result)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    // ── Snippets CRUD ──

    pub fn create_snippet(
        &self,
        id: &str,
        title: &str,
        content: &str,
        tag: &str,
        note: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO snippets (id, title, content, tag, note) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, title, content, tag, note],
        )?;
        Ok(())
    }

    pub fn update_snippet(
        &self,
        id: &str,
        title: &str,
        content: &str,
        tag: &str,
        note: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE snippets SET title=?2, content=?3, tag=?4, note=?5, updated_at=strftime('%s','now') WHERE id=?1",
            params![id, title, content, tag, note],
        )?;
        Ok(())
    }

    pub fn delete_snippet(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM snippets WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn increment_snippet_copy_count(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE snippets SET copy_count = copy_count + 1 WHERE id=?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn get_all_snippets(&self) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, content, tag, note, copy_count, created_at, updated_at, sort_order
             FROM snippets ORDER BY sort_order ASC, updated_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "title": row.get::<_, String>(1)?,
                    "content": row.get::<_, String>(2)?,
                    "tag": row.get::<_, String>(3)?,
                    "note": row.get::<_, String>(4)?,
                    "copy_count": row.get::<_, i64>(5)?,
                    "created_at": row.get::<_, i64>(6)?,
                    "updated_at": row.get::<_, i64>(7)?,
                    "sort_order": row.get::<_, i64>(8)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    pub fn reorder_snippets(&self, ids: &[String]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        for (i, id) in ids.iter().enumerate() {
            conn.execute(
                "UPDATE snippets SET sort_order = ?1 WHERE id = ?2",
                params![i as i64, id],
            )?;
        }
        Ok(())
    }

    pub fn save_known_device(
        &self,
        device_id: &str,
        device_name: &str,
        ip_addr: &str,
        tcp_port: u16,
        platform: &str,
        last_seen: u64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO known_devices (device_id, device_name, ip_addr, tcp_port, platform, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![device_id, device_name, ip_addr, tcp_port, platform, last_seen],
        )?;
        Ok(())
    }

    // ── Rooms ──

    pub fn create_room(&self, id: &str, name: &str, member_ids: &[String]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let members_json = serde_json::to_string(member_ids).unwrap_or_default();
        conn.execute(
            "INSERT OR REPLACE INTO rooms (id, name, member_ids) VALUES (?1, ?2, ?3)",
            params![id, name, members_json],
        )?;
        Ok(())
    }

    pub fn get_rooms(&self) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, member_ids, created_at FROM rooms ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                let members_str: String = row.get(2)?;
                let member_ids: Vec<String> =
                    serde_json::from_str(&members_str).unwrap_or_default();
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "member_ids": member_ids,
                    "created_at": row.get::<_, i64>(3)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    pub fn delete_room(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM rooms WHERE id = ?1", params![id])?;
        Ok(())
    }
}
