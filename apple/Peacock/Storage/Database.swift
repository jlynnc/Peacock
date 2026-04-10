import Foundation
import GRDB

final class PeacockDatabase: Sendable {
    let dbQueue: DatabaseQueue

    init() throws {
        let dir = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
            .appendingPathComponent("Peacock", isDirectory: true)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let dbPath = dir.appendingPathComponent("peacock.db").path
        dbQueue = try DatabaseQueue(path: dbPath)
        try migrate()
    }

    private func migrate() throws {
        try dbQueue.write { db in
            // Messages
            try db.execute(sql: """
                CREATE TABLE IF NOT EXISTS messages (
                    id TEXT PRIMARY KEY,
                    device_id TEXT NOT NULL,
                    direction TEXT NOT NULL,
                    content TEXT NOT NULL,
                    msg_type TEXT NOT NULL DEFAULT 'text',
                    timestamp INTEGER NOT NULL,
                    status TEXT NOT NULL DEFAULT 'sent'
                )
            """)
            try db.execute(sql: """
                CREATE INDEX IF NOT EXISTS idx_messages_device_ts ON messages(device_id, timestamp)
            """)

            // Settings
            try db.execute(sql: """
                CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                )
            """)

            // Snippets
            try db.execute(sql: """
                CREATE TABLE IF NOT EXISTS snippets (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL DEFAULT '新建片段',
                    content TEXT NOT NULL DEFAULT '',
                    tag TEXT NOT NULL DEFAULT '',
                    note TEXT NOT NULL DEFAULT '',
                    copy_count INTEGER NOT NULL DEFAULT 0,
                    sort_order INTEGER NOT NULL DEFAULT 0,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                )
            """)

            // Known devices
            try db.execute(sql: """
                CREATE TABLE IF NOT EXISTS known_devices (
                    device_id TEXT PRIMARY KEY,
                    device_name TEXT NOT NULL,
                    ip_addr TEXT NOT NULL,
                    tcp_port INTEGER NOT NULL,
                    platform TEXT NOT NULL,
                    last_seen INTEGER NOT NULL
                )
            """)
        }
    }

    // MARK: - Settings

    func getSetting(_ key: String) throws -> String? {
        try dbQueue.read { db in
            try String.fetchOne(db, sql: "SELECT value FROM settings WHERE key = ?", arguments: [key])
        }
    }

    func setSetting(_ key: String, value: String) throws {
        try dbQueue.write { db in
            try db.execute(
                sql: "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
                arguments: [key, value]
            )
        }
    }

    // MARK: - Messages

    func storeMessage(_ message: ChatMessage) throws {
        try dbQueue.write { db in
            try db.execute(
                sql: """
                    INSERT OR REPLACE INTO messages (id, device_id, direction, content, msg_type, timestamp, status)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                arguments: [message.id, message.deviceId, message.direction.rawValue,
                            message.content, message.msgType.rawValue,
                            message.timestamp, message.status.rawValue]
            )
        }
    }

    func getMessages(deviceId: String, offset: Int = 0, limit: Int = 50) throws -> [ChatMessage] {
        try dbQueue.read { db in
            let rows = try Row.fetchAll(db, sql: """
                SELECT * FROM messages WHERE device_id = ?
                ORDER BY timestamp DESC LIMIT ? OFFSET ?
            """, arguments: [deviceId, limit, offset])

            return rows.reversed().map { row in
                ChatMessage(
                    id: row["id"],
                    deviceId: row["device_id"],
                    direction: MessageDirection(rawValue: row["direction"]) ?? .received,
                    content: row["content"],
                    msgType: MessageType(rawValue: row["msg_type"]) ?? .text,
                    timestamp: UInt64(row["timestamp"] as Int64),
                    status: MessageStatus(rawValue: row["status"]) ?? .sent
                )
            }
        }
    }

    // MARK: - Snippets

    func getAllSnippets() throws -> [Snippet] {
        try dbQueue.read { db in
            let rows = try Row.fetchAll(db, sql: """
                SELECT * FROM snippets ORDER BY sort_order ASC, updated_at DESC
            """)
            return rows.map { row in
                Snippet(
                    id: row["id"],
                    title: row["title"],
                    content: row["content"],
                    tag: row["tag"],
                    note: row["note"],
                    sortOrder: row["sort_order"],
                    copyCount: row["copy_count"],
                    createdAt: UInt64(row["created_at"] as Int64),
                    updatedAt: UInt64(row["updated_at"] as Int64)
                )
            }
        }
    }

    func createSnippet(id: String, title: String, content: String, tag: String, note: String) throws {
        let now = UInt64(Date().timeIntervalSince1970 * 1000)
        try dbQueue.write { db in
            try db.execute(
                sql: """
                    INSERT INTO snippets (id, title, content, tag, note, sort_order, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, 0, ?, ?)
                """,
                arguments: [id, title, content, tag, note, now, now]
            )
        }
    }

    func updateSnippet(id: String, title: String, content: String, tag: String, note: String) throws {
        let now = UInt64(Date().timeIntervalSince1970 * 1000)
        try dbQueue.write { db in
            try db.execute(
                sql: """
                    UPDATE snippets SET title = ?, content = ?, tag = ?, note = ?, updated_at = ?
                    WHERE id = ?
                """,
                arguments: [title, content, tag, note, now, id]
            )
        }
    }

    func deleteSnippet(id: String) throws {
        try dbQueue.write { db in
            try db.execute(sql: "DELETE FROM snippets WHERE id = ?", arguments: [id])
        }
    }

    func incrementSnippetCopyCount(id: String) throws {
        try dbQueue.write { db in
            try db.execute(
                sql: "UPDATE snippets SET copy_count = copy_count + 1 WHERE id = ?",
                arguments: [id]
            )
        }
    }

    func reorderSnippets(ids: [String]) throws {
        try dbQueue.write { db in
            for (index, id) in ids.enumerated() {
                try db.execute(
                    sql: "UPDATE snippets SET sort_order = ? WHERE id = ?",
                    arguments: [index, id]
                )
            }
        }
    }

    // MARK: - Known Devices

    func saveKnownDevice(_ device: DeviceInfo) throws {
        try dbQueue.write { db in
            try db.execute(
                sql: """
                    INSERT OR REPLACE INTO known_devices
                    (device_id, device_name, ip_addr, tcp_port, platform, last_seen)
                    VALUES (?, ?, ?, ?, ?, ?)
                """,
                arguments: [device.deviceId, device.deviceName, device.ipAddr,
                            Int(device.tcpPort), device.platform,
                            Int64(device.lastSeen.timeIntervalSince1970)]
            )
        }
    }
}
