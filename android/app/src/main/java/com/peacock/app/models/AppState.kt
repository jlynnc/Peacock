package com.peacock.app.models

import android.content.Context
import android.os.Build
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.ViewModel
import com.peacock.app.network.UDPService
import com.peacock.app.protocol.*
import com.peacock.app.storage.PeacockDatabase
import java.net.InetAddress
import java.util.UUID

class AppState(context: Context) : ViewModel() {
    val db = PeacockDatabase(context)
    val deviceIdStr = db.getOrCreateDeviceId()
    val deviceIdBytes: ByteArray = run {
        val uuid = UUID.fromString(deviceIdStr)
        val bb = java.nio.ByteBuffer.allocate(16)
        bb.putLong(uuid.mostSignificantBits)
        bb.putLong(uuid.leastSignificantBits)
        bb.array()
    }
    val deviceName = mutableStateOf(db.getDeviceName(Build.MODEL))
    val platform = "android"

    val devices = mutableStateMapOf<String, DeviceInfo>()
    val messages = mutableStateMapOf<String, MutableList<ChatMessage>>()
    val snippets = mutableStateListOf<Snippet>()

    val selectedDeviceId = mutableStateOf<String?>(null)

    lateinit var udpService: UDPService

    fun init() {
        loadSnippets()
        udpService = UDPService(
            deviceId = deviceIdBytes,
            deviceIdStr = deviceIdStr,
            deviceName = deviceName.value,
            onDeviceDiscovered = { device ->
                devices[device.deviceId] = device
            },
            onDeviceOffline = { id ->
                devices[id]?.isOnline = false
                devices[id] = devices[id]!!.copy(isOnline = false)
            },
            onTextMessage = { senderId, payload ->
                val msg = ChatMessage(
                    id = payload.messageId,
                    deviceId = senderId,
                    direction = "received",
                    content = payload.text,
                    timestamp = payload.timestamp.toLong()
                )
                getOrCreateMessages(senderId).add(msg)
            },
            onFileOffer = { senderId, payload, senderAddr ->
                // TODO: handle file offer
            },
            onFileAccept = { senderId, payload, senderAddr ->
                // TODO: handle file accept
            },
            onFileReject = { senderId, payload ->
                // TODO: handle file reject
            },
            onSnippetShare = { senderId, payload ->
                val snippet = Snippet(
                    id = UUID.randomUUID().toString(),
                    title = payload.title,
                    content = payload.content,
                    tag = payload.tag,
                    note = payload.note
                )
                db.createSnippet().also {
                    db.updateSnippet(it.id, payload.title, payload.content, payload.tag, payload.note)
                }
                loadSnippets()
            }
        )
        udpService.start()
    }

    fun stop() {
        if (::udpService.isInitialized) udpService.stop()
    }

    fun sendMessage(deviceId: String, text: String) {
        val device = devices[deviceId] ?: return
        val msgId = UUID.randomUUID().toString()
        val timestamp = System.currentTimeMillis()

        val msg = ChatMessage(
            id = msgId,
            deviceId = deviceId,
            direction = "sent",
            content = text,
            timestamp = timestamp,
            status = "sent"
        )
        getOrCreateMessages(deviceId).add(msg)

        val payload = TextPayload(msgId, text, timestamp.toULong())
        try {
            val targetIp = InetAddress.getByName(device.ipAddr)
            udpService.sendToDevice(targetIp, PacketType.Text, payload.encode())
        } catch (e: Exception) {
            msg.status = "failed"
        }
    }

    fun getOrCreateMessages(deviceId: String): MutableList<ChatMessage> {
        return messages.getOrPut(deviceId) { mutableStateListOf() }
    }

    // ── Snippets ──

    fun loadSnippets() {
        snippets.clear()
        snippets.addAll(db.getSnippets())
    }

    fun createSnippet(): Snippet {
        val s = db.createSnippet()
        loadSnippets()
        return s
    }

    fun updateSnippet(id: String, title: String? = null, content: String? = null,
                      tag: String? = null, note: String? = null) {
        db.updateSnippet(id, title, content, tag, note)
        loadSnippets()
    }

    fun deleteSnippet(id: String) {
        db.deleteSnippet(id)
        loadSnippets()
    }

    fun updateDeviceName(name: String) {
        deviceName.value = name
        db.setSetting("device_name", name)
    }
}
