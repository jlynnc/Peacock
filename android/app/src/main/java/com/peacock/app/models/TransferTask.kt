package com.peacock.app.models

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import java.net.InetAddress

class TransferTask(
    val transferId: String,
    val deviceId: String,
    val fileName: String,
    val fileSize: Long,
    val isFolder: Boolean,
    val fileCount: Int,
    val direction: String, // "send" or "receive"
    val filePath: String = "", // source path for send
) {
    var status by mutableStateOf("pending") // pending, active, completed, failed, rejected
    var bytesTransferred by mutableLongStateOf(0L)
    var speedBps by mutableLongStateOf(0L)
    var localPath by mutableStateOf(filePath) // final path for received files
    var receiverPort: Int = 0
    var resumeOffset: Long = 0
    var senderAddr: InetAddress? = null

    val progress: Float
        get() = if (fileSize > 0) bytesTransferred.toFloat() / fileSize.toFloat() else 0f
}
