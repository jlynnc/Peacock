package com.peacock.app.network

import android.util.Log
import com.peacock.app.models.DeviceInfo
import com.peacock.app.protocol.*
import kotlinx.coroutines.*
import java.net.*
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

/**
 * Handles all UDP communication:
 * - Beacon broadcasting (every 10s)
 * - Listening for Announce/AnnounceResponse/messages
 * - UDP unicast response
 * - Restricted peers list management
 */
class UDPService(
    private val deviceId: ByteArray,
    private val deviceIdStr: String,
    private val deviceName: String,
    private val tcpPort: UShort = 52001u,
    private val onDeviceDiscovered: (DeviceInfo) -> Unit,
    private val onDeviceOffline: (String) -> Unit,
    private val onTextMessage: (String, TextPayload) -> Unit,
    private val onFileOffer: (String, FileOfferPayload, InetAddress) -> Unit,
    private val onFileAccept: (String, FileAcceptPayload, InetAddress) -> Unit,
    private val onFileReject: (String, FileRejectPayload) -> Unit,
    private val onSnippetShare: (String, SnippetSharePayload) -> Unit
) {
    companion object {
        const val TAG = "PeacockUDP"
        const val DISCOVERY_PORT = 52000
        const val MULTICAST_ADDR = "224.0.1.100"
        const val BEACON_INTERVAL_MS = 10_000L
        const val OFFLINE_TIMEOUT_S = 30L
    }

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private var socket: MulticastSocket? = null
    private val devices = ConcurrentHashMap<String, DeviceInfo>()
    private val broadcastRestricted = ConcurrentHashMap.newKeySet<String>()
    private val hasBroadcast = ConcurrentHashMap.newKeySet<String>()

    fun start() {
        scope.launch { runListener() }
        scope.launch { runBeacon() }
        scope.launch { runTimeoutChecker() }
    }

    fun stop() {
        scope.cancel()
        socket?.close()
    }

    fun getOnlineDevices(): List<DeviceInfo> = devices.values.filter { it.isOnline }

    fun getDevice(deviceId: String): DeviceInfo? = devices[deviceId]

    fun sendPacket(targetIp: InetAddress, packetType: PacketType, payload: ByteArray) {
        try {
            val header = PacketHeader(
                version = PacketHeader.PROTOCOL_VERSION,
                packetType = packetType.value,
                deviceId = deviceId,
                payloadLength = payload.size.toUInt()
            )
            val packet = header.toBytes() + payload
            val dgram = DatagramPacket(packet, packet.size, targetIp, DISCOVERY_PORT)
            socket?.send(dgram)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to send packet: ${e.message}")
        }
    }

    fun sendToDevice(targetIp: InetAddress, packetType: PacketType, payload: ByteArray) {
        scope.launch {
            sendPacket(targetIp, packetType, payload)
        }
    }

    private suspend fun runListener() {
        while (scope.isActive) {
            try {
                socket = MulticastSocket(null).apply {
                    reuseAddress = true
                    broadcast = true
                    bind(InetSocketAddress(DISCOVERY_PORT))
                    try {
                        joinGroup(InetAddress.getByName(MULTICAST_ADDR))
                    } catch (e: Exception) {
                        Log.w(TAG, "Failed to join multicast: ${e.message}")
                    }
                }

                Log.i(TAG, "UDP listener started on port $DISCOVERY_PORT")
                val buf = ByteArray(4096)

                while (scope.isActive) {
                    val dgram = DatagramPacket(buf, buf.size)
                    socket?.receive(dgram)
                    handlePacket(dgram.data.copyOf(dgram.length), dgram.address)
                }
            } catch (e: Exception) {
                if (scope.isActive) {
                    Log.w(TAG, "Listener error: ${e.message}, restarting in 3s...")
                    delay(3000)
                }
            }
        }
    }

    private fun handlePacket(data: ByteArray, senderAddr: InetAddress) {
        if (data.size < PacketHeader.SIZE) return
        val header = PacketHeader.fromBytes(data) ?: return
        if (!header.isValid()) return
        if (header.deviceId.contentEquals(deviceId)) return // ignore own packets

        val senderIdStr = uuidFromBytes(header.deviceId)
        val payloadData = data.copyOfRange(PacketHeader.SIZE, PacketHeader.SIZE + header.payloadLength.toInt())

        when (header.getPacketType()) {
            PacketType.Announce -> handleAnnounce(senderIdStr, senderAddr, payloadData, isBroadcast = true)
            PacketType.AnnounceResponse -> handleAnnounce(senderIdStr, senderAddr, payloadData, isBroadcast = false)
            PacketType.Bye -> handleBye(senderIdStr)
            PacketType.Text -> handleText(senderIdStr, payloadData)
            PacketType.FileOffer -> handleFileOffer(senderIdStr, payloadData, senderAddr)
            PacketType.FileAccept -> handleFileAccept(senderIdStr, payloadData, senderAddr)
            PacketType.FileReject -> handleFileReject(senderIdStr, payloadData)
            PacketType.SnippetShare -> handleSnippetShare(senderIdStr, payloadData)
            else -> {}
        }
    }

    private fun handleAnnounce(senderId: String, senderIp: InetAddress, payloadData: ByteArray, isBroadcast: Boolean) {
        try {
            val payload = AnnouncePayload.decode(payloadData)
            val isNew = upsertDevice(senderId, payload.deviceName, senderIp.hostAddress ?: "", payload.tcpPort, payload.platform)

            if (isBroadcast) {
                hasBroadcast.add(senderId)
                broadcastRestricted.remove(senderId)

                // Process restricted peers
                for (peer in payload.restrictedPeers) {
                    if (peer.deviceId == deviceIdStr) continue
                    val peerIsNew = upsertDevice(peer.deviceId, peer.deviceName, peer.ipAddr, peer.tcpPort, peer.platform)
                    broadcastRestricted.add(peer.deviceId)
                    if (peerIsNew) {
                        devices[peer.deviceId]?.let { onDeviceDiscovered(it) }
                    }
                }

                // Send unicast response
                val response = AnnouncePayload(
                    deviceName = deviceName,
                    platform = "android",
                    tcpPort = tcpPort,
                    features = 0xFFFFu,
                    restrictedPeers = emptyList()
                )
                sendPacket(senderIp, PacketType.AnnounceResponse, response.encode())
            } else {
                // Response — check if this device is broadcast-restricted
                if (!hasBroadcast.contains(senderId)) {
                    broadcastRestricted.add(senderId)
                }
            }

            if (isNew) {
                devices[senderId]?.let { onDeviceDiscovered(it) }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to decode announce: ${e.message}")
        }
    }

    private fun handleBye(senderId: String) {
        devices[senderId]?.let {
            it.isOnline = false
            onDeviceOffline(senderId)
        }
    }

    private fun handleText(senderId: String, data: ByteArray) {
        try {
            val payload = TextPayload.decode(data)
            onTextMessage(senderId, payload)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to decode text: ${e.message}")
        }
    }

    private fun handleFileOffer(senderId: String, data: ByteArray, senderAddr: InetAddress) {
        try {
            val payload = FileOfferPayload.decode(data)
            onFileOffer(senderId, payload, senderAddr)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to decode file offer: ${e.message}")
        }
    }

    private fun handleFileAccept(senderId: String, data: ByteArray, senderAddr: InetAddress) {
        try {
            val payload = FileAcceptPayload.decode(data)
            onFileAccept(senderId, payload, senderAddr)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to decode file accept: ${e.message}")
        }
    }

    private fun handleFileReject(senderId: String, data: ByteArray) {
        try {
            val payload = FileRejectPayload.decode(data)
            onFileReject(senderId, payload)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to decode file reject: ${e.message}")
        }
    }

    private fun handleSnippetShare(senderId: String, data: ByteArray) {
        try {
            val payload = SnippetSharePayload.decode(data)
            onSnippetShare(senderId, payload)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to decode snippet share: ${e.message}")
        }
    }

    private suspend fun runBeacon() {
        delay(1000) // wait for listener to start
        while (scope.isActive) {
            try {
                val restrictedPeers = broadcastRestricted.mapNotNull { id ->
                    devices[id]?.let {
                        PeerInfo(it.deviceId, it.deviceName, it.ipAddr, it.tcpPort, it.platform)
                    }
                }

                val payload = AnnouncePayload(
                    deviceName = deviceName,
                    platform = "android",
                    tcpPort = tcpPort,
                    features = 0xFFFFu,
                    restrictedPeers = restrictedPeers
                )
                val payloadBytes = payload.encode()
                val header = PacketHeader(
                    version = PacketHeader.PROTOCOL_VERSION,
                    packetType = PacketType.Announce.value,
                    deviceId = deviceId,
                    payloadLength = payloadBytes.size.toUInt()
                )
                val packet = header.toBytes() + payloadBytes

                // Multicast
                try {
                    val multicastAddr = InetAddress.getByName(MULTICAST_ADDR)
                    socket?.send(DatagramPacket(packet, packet.size, multicastAddr, DISCOVERY_PORT))
                } catch (e: Exception) {
                    Log.d(TAG, "Multicast send failed: ${e.message}")
                }

                // Broadcast
                try {
                    val broadcastAddr = InetAddress.getByName("255.255.255.255")
                    socket?.send(DatagramPacket(packet, packet.size, broadcastAddr, DISCOVERY_PORT))
                } catch (e: Exception) {
                    Log.d(TAG, "Broadcast send failed: ${e.message}")
                }
            } catch (e: Exception) {
                Log.e(TAG, "Beacon error: ${e.message}")
            }
            delay(BEACON_INTERVAL_MS)
        }
    }

    private suspend fun runTimeoutChecker() {
        while (scope.isActive) {
            delay(OFFLINE_TIMEOUT_S * 500) // check every half timeout
            val now = System.currentTimeMillis() / 1000
            devices.forEach { (id, device) ->
                if (device.isOnline && (now - device.lastSeen) > OFFLINE_TIMEOUT_S) {
                    device.isOnline = false
                    broadcastRestricted.remove(id)
                    hasBroadcast.remove(id)
                    onDeviceOffline(id)
                }
            }
        }
    }

    private fun upsertDevice(id: String, name: String, ip: String, port: UShort, platform: String): Boolean {
        val now = System.currentTimeMillis() / 1000
        val existing = devices[id]
        if (existing != null) {
            existing.lastSeen = now
            existing.isOnline = true
            return false
        }
        devices[id] = DeviceInfo(id, name, ip, port, platform, now, true)
        return true
    }

    private fun uuidFromBytes(bytes: ByteArray): String {
        val bb = java.nio.ByteBuffer.wrap(bytes)
        val high = bb.long
        val low = bb.long
        return UUID(high, low).toString()
    }
}
