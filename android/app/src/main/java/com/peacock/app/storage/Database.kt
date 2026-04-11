package com.peacock.app.storage

import android.content.ContentValues
import android.content.Context
import android.database.sqlite.SQLiteDatabase
import android.database.sqlite.SQLiteOpenHelper
import com.peacock.app.models.Snippet
import java.util.UUID

class PeacockDatabase(context: Context) : SQLiteOpenHelper(context, "peacock.db", null, 1) {

    override fun onCreate(db: SQLiteDatabase) {
        db.execSQL("""
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
        """)
        db.execSQL("""
            CREATE TABLE IF NOT EXISTS snippets (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL DEFAULT '新建片段',
                content TEXT NOT NULL DEFAULT '',
                tag TEXT NOT NULL DEFAULT '',
                note TEXT NOT NULL DEFAULT '',
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
        """)
    }

    override fun onUpgrade(db: SQLiteDatabase, oldVersion: Int, newVersion: Int) {}

    // ── Settings ──

    fun getSetting(key: String): String? {
        val cursor = readableDatabase.rawQuery(
            "SELECT value FROM settings WHERE key = ?", arrayOf(key)
        )
        return cursor.use { if (it.moveToFirst()) it.getString(0) else null }
    }

    fun setSetting(key: String, value: String) {
        writableDatabase.execSQL(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            arrayOf(key, value)
        )
    }

    fun getOrCreateDeviceId(): String {
        return getSetting("device_id") ?: run {
            val id = UUID.randomUUID().toString()
            setSetting("device_id", id)
            id
        }
    }

    fun getDeviceName(default: String): String {
        return getSetting("device_name") ?: run {
            setSetting("device_name", default)
            default
        }
    }

    // ── Snippets ──

    fun getSnippets(): List<Snippet> {
        val list = mutableListOf<Snippet>()
        val cursor = readableDatabase.rawQuery(
            "SELECT * FROM snippets ORDER BY sort_order ASC, updated_at DESC", null
        )
        cursor.use {
            while (it.moveToNext()) {
                list.add(Snippet(
                    id = it.getString(it.getColumnIndexOrThrow("id")),
                    title = it.getString(it.getColumnIndexOrThrow("title")),
                    content = it.getString(it.getColumnIndexOrThrow("content")),
                    tag = it.getString(it.getColumnIndexOrThrow("tag")),
                    note = it.getString(it.getColumnIndexOrThrow("note")),
                    sortOrder = it.getInt(it.getColumnIndexOrThrow("sort_order")),
                    createdAt = it.getLong(it.getColumnIndexOrThrow("created_at")),
                    updatedAt = it.getLong(it.getColumnIndexOrThrow("updated_at"))
                ))
            }
        }
        return list
    }

    fun createSnippet(): Snippet {
        val now = System.currentTimeMillis() / 1000
        val snippet = Snippet(
            id = UUID.randomUUID().toString(),
            createdAt = now,
            updatedAt = now
        )
        val cv = ContentValues().apply {
            put("id", snippet.id)
            put("title", snippet.title)
            put("content", snippet.content)
            put("tag", snippet.tag)
            put("note", snippet.note)
            put("sort_order", snippet.sortOrder)
            put("created_at", snippet.createdAt)
            put("updated_at", snippet.updatedAt)
        }
        writableDatabase.insert("snippets", null, cv)
        return snippet
    }

    fun updateSnippet(id: String, title: String? = null, content: String? = null,
                      tag: String? = null, note: String? = null) {
        val cv = ContentValues().apply {
            title?.let { put("title", it) }
            content?.let { put("content", it) }
            tag?.let { put("tag", it) }
            note?.let { put("note", it) }
            put("updated_at", System.currentTimeMillis() / 1000)
        }
        writableDatabase.update("snippets", cv, "id = ?", arrayOf(id))
    }

    fun deleteSnippet(id: String) {
        writableDatabase.delete("snippets", "id = ?", arrayOf(id))
    }

    fun reorderSnippets(ids: List<String>) {
        val db = writableDatabase
        db.beginTransaction()
        try {
            ids.forEachIndexed { index, id ->
                db.execSQL("UPDATE snippets SET sort_order = ? WHERE id = ?",
                    arrayOf(index, id))
            }
            db.setTransactionSuccessful()
        } finally {
            db.endTransaction()
        }
    }
}
