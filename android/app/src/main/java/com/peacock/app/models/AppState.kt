package com.peacock.app.models

import android.content.Context
import android.os.Build
import android.os.Environment
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.ViewModel
import com.peacock.app.network.FileTransferService
import com.peacock.app.network.UDPService
import com.peacock.app.protocol.*
import com.peacock.app.storage.PeacockDatabase
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import java.io.File
import java.net.InetAddress
import java.net.InetSocketAddress
import java.util.UUID

class AppState(private val context: Context) : ViewModel() {
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
    val transfers = mutableStateMapOf<String, TransferTask>()

    // Pending file offers waiting for user decision
    val pendingOffers = mutableStateListOf<TransferTask>()

    val selectedDeviceId = mutableStateOf<String?>(null)

    private val transferScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    lateinit var fileTransferService: FileTransferService
    lateinit var udpService: UDPService

    val downloadDir: File
        get() {
            val dir = db.getSetting("download_dir")
            return if (dir != null) File(dir) else {
                val downloads = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS)
                File(downloads, "Peacock")
            }
        }

    fun init() {
        loadSnippets()
        fileTransferService = FileTransferService(transferScope)
        udpService = UDPService(
            deviceId = deviceIdBytes,
            deviceIdStr = deviceIdStr,
            deviceName = deviceName.value,
            onDeviceDiscovered = { device ->
                devices[device.deviceId] = device
            },
            onDeviceOffline = { id ->
                devices[id]?.let {
                    devices[id] = it.copy(isOnline = false)
                }
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
                handleFileOffer(senderId, payload, senderAddr)
            },
            onFileAccept = { senderId, payload, senderAddr ->
                handleFileAccept(senderId, payload, senderAddr)
            },
            onFileReject = { senderId, payload ->
                handleFileReject(senderId, payload)
            },
            onSnippetShare = { senderId, payload ->
                val id = UUID.randomUUID().toString()
                db.createSnippet().also {
                    db.updateSnippet(it.id, payload.title, payload.content, payload.tag, payload.note)
                }
                loadSnippets()

                // Also add a chat message about the shared snippet
                val msg = ChatMessage(
                    id = UUID.randomUUID().toString(),
                    deviceId = senderId,
                    direction = "received",
                    content = "[片段] ${payload.title}",
                    msgType = "snippet",
                    timestamp = System.currentTimeMillis()
                )
                getOrCreateMessages(senderId).add(msg)
            }
        )
        udpService.start()
    }

    fun stop() {
        if (::udpService.isInitialized) udpService.stop()
    }


    // ── Messages ──

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

        val payload = TextPayload(msgId, text, timestamp.toULong(), deviceId)
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

    // ── File Transfer ──

    private fun handleFileOffer(senderId: String, payload: FileOfferPayload, senderAddr: InetAddress) {
        val task = TransferTask(
            transferId = payload.transferId,
            deviceId = senderId,
            fileName = payload.fileName,
            fileSize = payload.fileSize.toLong(),
            isFolder = payload.isFolder,
            fileCount = payload.fileCount.toInt(),
            direction = "receive"
        )
        task.senderAddr = senderAddr
        transfers[payload.transferId] = task
        pendingOffers.add(task)

        // Add file message to chat
        val msg = ChatMessage(
            id = UUID.randomUUID().toString(),
            deviceId = senderId,
            direction = "received",
            content = payload.fileName,
            msgType = "file",
            timestamp = System.currentTimeMillis(),
            transferId = payload.transferId,
            fileName = payload.fileName,
            fileSize = payload.fileSize.toLong()
        )
        getOrCreateMessages(senderId).add(msg)
    }

    fun acceptFileOffer(transferId: String) {
        val task = transfers[transferId] ?: return
        pendingOffers.removeAll { it.transferId == transferId }

        val port = fileTransferService.startReceiver(
            task = task,
            downloadDir = downloadDir,
            onProgress = { bytes, speed ->
                task.bytesTransferred = bytes
                task.speedBps = speed
            },
            onComplete = { path ->
                task.localPath = path
                task.status = "completed"
            },
            onError = { _ ->
                task.status = "failed"
            }
        )

        task.receiverPort = port

        // Send FileAccept via UDP
        val device = devices[task.deviceId] ?: return
        val acceptPayload = FileAcceptPayload(
            transferId = transferId,
            receiverPort = port.toUShort(),
            resumeOffset = task.resumeOffset.toULong()
        )
        try {
            val targetIp = InetAddress.getByName(device.ipAddr)
            udpService.sendToDevice(targetIp, PacketType.FileAccept, acceptPayload.encode())
        } catch (e: Exception) {
            task.status = "failed"
        }
    }

    fun rejectFileOffer(transferId: String) {
        val task = transfers[transferId] ?: return
        pendingOffers.removeAll { it.transferId == transferId }
        task.status = "rejected"

        val device = devices[task.deviceId] ?: return
        val rejectPayload = FileRejectPayload(transferId)
        try {
            val targetIp = InetAddress.getByName(device.ipAddr)
            udpService.sendToDevice(targetIp, PacketType.FileReject, rejectPayload.encode())
        } catch (_: Exception) {}
    }

    private fun handleFileAccept(senderId: String, payload: FileAcceptPayload, senderAddr: InetAddress) {
        val task = transfers[payload.transferId] ?: return
        task.resumeOffset = payload.resumeOffset.toLong()

        val targetAddr = InetSocketAddress(senderAddr, payload.receiverPort.toInt())
        fileTransferService.startSender(
            task = task,
            targetAddr = targetAddr,
            onProgress = { bytes, speed ->
                task.bytesTransferred = bytes
                task.speedBps = speed
            },
            onComplete = {
                task.status = "completed"
            },
            onError = { _ ->
                task.status = "failed"
            }
        )
    }

    private fun handleFileReject(senderId: String, payload: FileRejectPayload) {
        val task = transfers[payload.transferId] ?: return
        task.status = "rejected"
    }

    fun sendFile(deviceId: String, filePath: String, fileName: String, fileSize: Long) {
        val device = devices[deviceId] ?: return
        val transferId = UUID.randomUUID().toString()
        val file = File(filePath)
        val isFolder = file.isDirectory

        val actualSize: Long
        val fileCount: Int
        if (isFolder) {
            val files = mutableListOf<File>()
            collectFiles(file, files)
            actualSize = files.sumOf { it.length() }
            fileCount = files.size
        } else {
            actualSize = fileSize
            fileCount = 1
        }

        val task = TransferTask(
            transferId = transferId,
            deviceId = deviceId,
            fileName = fileName,
            fileSize = actualSize,
            isFolder = isFolder,
            fileCount = fileCount,
            direction = "send",
            filePath = filePath
        )
        transfers[transferId] = task

        // Add file message to chat
        val msg = ChatMessage(
            id = UUID.randomUUID().toString(),
            deviceId = deviceId,
            direction = "sent",
            content = fileName,
            msgType = "file",
            timestamp = System.currentTimeMillis(),
            transferId = transferId,
            fileName = fileName,
            fileSize = actualSize
        )
        getOrCreateMessages(deviceId).add(msg)

        // Send FileOffer via UDP
        val offerPayload = FileOfferPayload(
            transferId = transferId,
            fileName = fileName,
            fileSize = actualSize.toULong(),
            isFolder = isFolder,
            fileCount = fileCount.toUInt()
        )
        try {
            val targetIp = InetAddress.getByName(device.ipAddr)
            udpService.sendToDevice(targetIp, PacketType.FileOffer, offerPayload.encode())
        } catch (e: Exception) {
            task.status = "failed"
        }
    }

    private fun collectFiles(dir: File, result: MutableList<File>) {
        val files = dir.listFiles() ?: return
        for (f in files.sortedBy { it.name }) {
            if (f.isDirectory) collectFiles(f, result)
            else result.add(f)
        }
    }

    // ── Snippet Sharing ──

    fun shareSnippet(snippetId: String, deviceId: String) {
        val snippet = snippets.find { it.id == snippetId } ?: return
        val device = devices[deviceId] ?: return

        val payload = SnippetSharePayload(
            title = snippet.title,
            content = snippet.content,
            tag = snippet.tag,
            note = snippet.note
        )
        try {
            val targetIp = InetAddress.getByName(device.ipAddr)
            udpService.sendToDevice(targetIp, PacketType.SnippetShare, payload.encode())

            // Add a chat message
            val msg = ChatMessage(
                id = UUID.randomUUID().toString(),
                deviceId = deviceId,
                direction = "sent",
                content = "[片段] ${snippet.title}",
                msgType = "snippet",
                timestamp = System.currentTimeMillis()
            )
            getOrCreateMessages(deviceId).add(msg)
        } catch (_: Exception) {}
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

    fun pinSnippetToTop(id: String) {
        val ids = snippets.map { it.id }.toMutableList()
        ids.remove(id)
        ids.add(0, id)
        db.reorderSnippets(ids)
        loadSnippets()
    }

    fun updateDeviceName(name: String) {
        deviceName.value = name
        db.setSetting("device_name", name)
    }
}
