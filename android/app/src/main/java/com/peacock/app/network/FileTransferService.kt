package com.peacock.app.network

import android.util.Log
import com.peacock.app.models.TransferTask
import kotlinx.coroutines.*
import org.json.JSONArray
import org.json.JSONObject
import java.io.*
import java.net.InetSocketAddress
import java.net.ServerSocket
import java.net.Socket

/**
 * Handles TCP file transfer - both sending and receiving.
 * Protocol-compatible with desktop Rust and iOS Swift implementations.
 *
 * Single file: raw bytes (64KB chunks), size known from FileOffer
 * Folder: [u64 LE manifest_len][JSON manifest][file1 bytes][file2 bytes]...
 */
class FileTransferService(private val scope: CoroutineScope) {

    companion object {
        const val TAG = "PeacockTransfer"
        const val CHUNK_SIZE = 262144 // 256KB
        const val CONNECT_TIMEOUT_MS = 30_000

        fun getUniquePath(file: File): File {
            if (!file.exists()) return file

            val name = file.nameWithoutExtension
            val ext = if (file.extension.isNotEmpty()) ".${file.extension}" else ""
            val parent = file.parentFile ?: return file

            var i = 1
            while (true) {
                val newFile = if (file.isDirectory || ext.isEmpty()) {
                    File(parent, "$name($i)")
                } else {
                    File(parent, "$name($i)$ext")
                }
                if (!newFile.exists()) return newFile
                i++
            }
        }
    }

    /**
     * Start receiving a file. Opens a ServerSocket on a random port.
     * Returns the port number to send in FileAccept.
     */
    fun startReceiver(
        task: TransferTask,
        downloadDir: File,
        onProgress: (Long, Long) -> Unit,
        onComplete: (String) -> Unit,
        onError: (String) -> Unit
    ): Int {
        val server = ServerSocket(0)
        val port = server.localPort

        scope.launch {
            try {
                server.soTimeout = CONNECT_TIMEOUT_MS
                Log.i(TAG, "Receiver listening on port $port for ${task.transferId}")

                val socket = withContext(Dispatchers.IO) { server.accept() }
                socket.tcpNoDelay = true
                socket.receiveBufferSize = 512 * 1024
                Log.i(TAG, "Sender connected from ${socket.inetAddress} for ${task.transferId}")

                task.status = "active"

                if (task.isFolder) {
                    receiveFolder(socket, task, downloadDir, onProgress, onComplete)
                } else {
                    receiveSingleFile(socket, task, downloadDir, onProgress, onComplete)
                }
            } catch (e: Exception) {
                Log.e(TAG, "Receive failed for ${task.transferId}: ${e.message}")
                task.status = "failed"
                onError(e.message ?: "Unknown error")
            } finally {
                withContext(Dispatchers.IO) { server.close() }
            }
        }

        return port
    }

    private suspend fun receiveSingleFile(
        socket: Socket,
        task: TransferTask,
        downloadDir: File,
        onProgress: (Long, Long) -> Unit,
        onComplete: (String) -> Unit
    ) = withContext(Dispatchers.IO) {
        downloadDir.mkdirs()
        val partFile = File(downloadDir, "${task.fileName}.part")
        val finalFile = getUniquePath(File(downloadDir, task.fileName))

        val resumeOffset = if (partFile.exists()) partFile.length() else 0L
        task.resumeOffset = resumeOffset

        val fos = BufferedOutputStream(
            if (resumeOffset > 0) FileOutputStream(partFile, true)
            else FileOutputStream(partFile),
            CHUNK_SIZE
        )

        val input = BufferedInputStream(socket.getInputStream(), CHUNK_SIZE)
        val buf = ByteArray(CHUNK_SIZE)
        var totalReceived = resumeOffset
        val startTime = System.currentTimeMillis()
        var lastReport = System.currentTimeMillis()

        try {
            while (true) {
                val n = input.read(buf)
                if (n <= 0) break

                fos.write(buf, 0, n)
                totalReceived += n

                val now = System.currentTimeMillis()
                if (now - lastReport >= 100) {
                    val elapsed = ((now - startTime) / 1000.0).coerceAtLeast(0.001)
                    val speed = ((totalReceived - resumeOffset) / elapsed).toLong()
                    task.bytesTransferred = totalReceived
                    task.speedBps = speed
                    onProgress(totalReceived, speed)
                    lastReport = now
                }
            }

            fos.flush()
            fos.close()

            partFile.renameTo(finalFile)

            task.bytesTransferred = task.fileSize
            task.speedBps = 0
            task.status = "completed"
            task.localPath = finalFile.absolutePath
            onComplete(finalFile.absolutePath)

            Log.i(TAG, "File receive completed: ${task.transferId} -> ${finalFile.absolutePath}")
        } finally {
            try { fos.close() } catch (_: Exception) {}
            try { socket.close() } catch (_: Exception) {}
        }
    }

    private suspend fun receiveFolder(
        socket: Socket,
        task: TransferTask,
        downloadDir: File,
        onProgress: (Long, Long) -> Unit,
        onComplete: (String) -> Unit
    ) = withContext(Dispatchers.IO) {
        val input = DataInputStream(BufferedInputStream(socket.getInputStream(), CHUNK_SIZE))

        // 1. Read manifest: [u64 LE manifest_len][JSON manifest]
        val manifestLen = java.lang.Long.reverseBytes(input.readLong())
        val manifestBuf = ByteArray(manifestLen.toInt())
        input.readFully(manifestBuf)

        val manifestJson = JSONArray(String(manifestBuf, Charsets.UTF_8))
        Log.i(TAG, "Folder manifest: ${manifestJson.length()} files")

        // Create folder
        val folderPath = getUniquePath(File(downloadDir, task.fileName))
        folderPath.mkdirs()

        // 2. Receive each file
        val buf = ByteArray(CHUNK_SIZE)
        var totalReceived = 0L
        val startTime = System.currentTimeMillis()
        var lastReport = System.currentTimeMillis()

        for (i in 0 until manifestJson.length()) {
            val entry = manifestJson.getJSONObject(i)
            val relativePath = entry.getString("relative_path")
            val fileSize = entry.getLong("size")

            val filePath = File(folderPath, relativePath)
            filePath.parentFile?.mkdirs()

            val fos = BufferedOutputStream(FileOutputStream(filePath), CHUNK_SIZE)
            var remaining = fileSize

            while (remaining > 0) {
                val toRead = remaining.coerceAtMost(CHUNK_SIZE.toLong()).toInt()
                val n = input.read(buf, 0, toRead)
                if (n <= 0) throw IOException("Connection closed before all data received")

                fos.write(buf, 0, n)
                remaining -= n
                totalReceived += n

                val now = System.currentTimeMillis()
                if (now - lastReport >= 100) {
                    val elapsed = ((now - startTime) / 1000.0).coerceAtLeast(0.001)
                    val speed = (totalReceived / elapsed).toLong()
                    task.bytesTransferred = totalReceived
                    task.speedBps = speed
                    onProgress(totalReceived, speed)
                    lastReport = now
                }
            }

            fos.flush()
            fos.close()
        }

        try { socket.close() } catch (_: Exception) {}

        task.bytesTransferred = task.fileSize
        task.speedBps = 0
        task.status = "completed"
        task.localPath = folderPath.absolutePath
        onComplete(folderPath.absolutePath)

        Log.i(TAG, "Folder receive completed: ${task.transferId} -> ${folderPath.absolutePath}")
    }

    /**
     * Send a file to the receiver's port.
     */
    fun startSender(
        task: TransferTask,
        targetAddr: InetSocketAddress,
        onProgress: (Long, Long) -> Unit,
        onComplete: () -> Unit,
        onError: (String) -> Unit
    ) {
        scope.launch {
            try {
                task.status = "active"

                if (task.isFolder) {
                    sendFolder(task, targetAddr, onProgress)
                } else {
                    sendSingleFile(task, targetAddr, onProgress)
                }

                task.bytesTransferred = task.fileSize
                task.speedBps = 0
                task.status = "completed"
                onComplete()

                Log.i(TAG, "File send completed: ${task.transferId}")
            } catch (e: Exception) {
                Log.e(TAG, "Send failed for ${task.transferId}: ${e.message}")
                task.status = "failed"
                onError(e.message ?: "Unknown error")
            }
        }
    }

    private suspend fun sendSingleFile(
        task: TransferTask,
        targetAddr: InetSocketAddress,
        onProgress: (Long, Long) -> Unit
    ) = withContext(Dispatchers.IO) {
        val file = File(task.filePath)
        val fis = BufferedInputStream(FileInputStream(file), CHUNK_SIZE)
        val resumeOffset = task.resumeOffset

        if (resumeOffset > 0) {
            fis.skip(resumeOffset)
        }

        val socket = Socket()
        socket.tcpNoDelay = true
        socket.sendBufferSize = 512 * 1024
        socket.connect(targetAddr, CONNECT_TIMEOUT_MS)
        val output = BufferedOutputStream(socket.getOutputStream(), CHUNK_SIZE * 4)

        val buf = ByteArray(CHUNK_SIZE)
        var totalSent = resumeOffset
        val startTime = System.currentTimeMillis()
        var lastReport = System.currentTimeMillis()

        try {
            while (true) {
                val n = fis.read(buf)
                if (n <= 0) break

                output.write(buf, 0, n)
                totalSent += n

                val now = System.currentTimeMillis()
                if (now - lastReport >= 100) {
                    val elapsed = ((now - startTime) / 1000.0).coerceAtLeast(0.001)
                    val speed = ((totalSent - resumeOffset) / elapsed).toLong()
                    task.bytesTransferred = totalSent
                    task.speedBps = speed
                    onProgress(totalSent, speed)
                    lastReport = now
                }
            }

            output.flush()
        } finally {
            try { fis.close() } catch (_: Exception) {}
            try { socket.close() } catch (_: Exception) {}
        }
    }

    private suspend fun sendFolder(
        task: TransferTask,
        targetAddr: InetSocketAddress,
        onProgress: (Long, Long) -> Unit
    ) = withContext(Dispatchers.IO) {
        val baseDir = File(task.filePath)
        val socket = Socket()
        socket.tcpNoDelay = true
        socket.sendBufferSize = 512 * 1024
        socket.connect(targetAddr, CONNECT_TIMEOUT_MS)
        val output = DataOutputStream(BufferedOutputStream(socket.getOutputStream(), CHUNK_SIZE * 4))

        // Build manifest
        val manifest = JSONArray()
        val files = mutableListOf<Pair<String, Long>>()
        collectFiles(baseDir, baseDir, files)

        for ((relPath, size) in files) {
            val entry = JSONObject()
            entry.put("relative_path", relPath)
            entry.put("size", size)
            manifest.put(entry)
        }

        // 1. Write manifest: [u64 LE manifest_len][JSON manifest]
        val manifestBytes = manifest.toString().toByteArray(Charsets.UTF_8)
        output.writeLong(java.lang.Long.reverseBytes(manifestBytes.size.toLong()))
        output.write(manifestBytes)

        // 2. Send each file
        val buf = ByteArray(CHUNK_SIZE)
        var totalSent = 0L
        val startTime = System.currentTimeMillis()
        var lastReport = System.currentTimeMillis()

        for ((relPath, _) in files) {
            val file = File(baseDir, relPath)
            val fis = FileInputStream(file)

            while (true) {
                val n = fis.read(buf)
                if (n <= 0) break

                output.write(buf, 0, n)
                totalSent += n

                val now = System.currentTimeMillis()
                if (now - lastReport >= 100) {
                    val elapsed = ((now - startTime) / 1000.0).coerceAtLeast(0.001)
                    val speed = (totalSent / elapsed).toLong()
                    task.bytesTransferred = totalSent
                    task.speedBps = speed
                    onProgress(totalSent, speed)
                    lastReport = now
                }
            }

            fis.close()
        }

        output.flush()
        try { socket.close() } catch (_: Exception) {}
    }

    private fun collectFiles(baseDir: File, currentDir: File, result: MutableList<Pair<String, Long>>) {
        val files = currentDir.listFiles() ?: return
        for (file in files.sortedBy { it.name }) {
            if (file.isDirectory) {
                collectFiles(baseDir, file, result)
            } else {
                val relPath = file.relativeTo(baseDir).path.replace('\\', '/')
                result.add(relPath to file.length())
            }
        }
    }
}
