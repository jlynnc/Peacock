package com.peacock.app.models

data class DeviceInfo(
    val deviceId: String,
    val deviceName: String,
    val ipAddr: String,
    val tcpPort: UShort,
    val platform: String,
    var lastSeen: Long = System.currentTimeMillis() / 1000,
    var isOnline: Boolean = true
)
