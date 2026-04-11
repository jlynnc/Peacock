package com.peacock.app.models

data class ChatMessage(
    val id: String,
    val deviceId: String,
    val direction: String, // "sent" or "received"
    val content: String,
    val msgType: String = "text", // "text", "file", "snippet"
    val timestamp: Long = System.currentTimeMillis(),
    var status: String = "sent", // "sending", "sent", "failed"
    val transferId: String? = null,
    val fileName: String? = null,
    val fileSize: Long = 0
)
