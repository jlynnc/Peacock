package com.peacock.app.models

data class DeviceInfo(
    val deviceId: String,
    var deviceName: String,
    var ipAddr: String,
    val tcpPort: UShort,
    val platform: String,
    var lastSeen: Long = System.currentTimeMillis() / 1000,
    var isOnline: Boolean = true,
    var lastBroadcastAt: Long = 0, // 0 = never received broadcast
    var isRestricted: Boolean = false, // derived from lastBroadcastAt
)
