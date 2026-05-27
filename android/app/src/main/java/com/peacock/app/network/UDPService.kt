package com.peacock.app.network

import android.util.Log
import com.peacock.app.models.DeviceInfo
import com.peacock.app.protocol.*
import kotlinx.coroutines.*
import java.net.*
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

/**
 * UDP discovery and messaging service.
 *
 * Device discovery rules:
 * 1. We broadcast → they respond (AnnounceResponse) → we add them
 * 2. We receive broadcast → we find ourselves in their restricted_peers → we add them
 * 3. Receiving a broadcast does NOT add the broadcaster (only send response)
 *
 * Restricted status: derived from last_broadcast_at on each device.
 * Refreshed before each beacon send.
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
        const val RESTRICTED_THRESHOLD_S = 30L // 3x beacon interval
    }

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private var socket: MulticastSocket? = null
    private val devices = ConcurrentHashMap<String, DeviceInfo>()

    fun start() {
        scope.launch { runListener() }
        scope.launch { runBeacon() }
        scope.launch { runTimeoutChecker() }
    }

    fun stop() {
        scope.cancel()
        socket?.close()
    }

    fun sendPacket(targetIp: InetAddress, packetType: PacketType, payload: ByteArray) {
        try {
            val header = PacketHeader(
                version = PacketHeader.PROTOCOL_VERSION,
                packetType = packetType.value,
                deviceId = deviceId,
                payloadLength = payload.size.toUInt()
            )
            val packet = header.toBytes() + payload
            socket?.send(DatagramPacket(packet, packet.size, targetIp, DISCOVERY_PORT))
        } catch (e: Exception) {
            Log.e(TAG, "Failed to send packet: ${e.message}")
        }
    }

    fun sendToDevice(targetIp: InetAddress, packetType: PacketType, payload: ByteArray) {
        scope.launch { sendPacket(targetIp, packetType, payload) }
    }

    // ── Listener ──

    private suspend fun runListener() {
        while (scope.isActive) {
            try {
                socket = MulticastSocket(null).apply {
                    reuseAddress = true
                    broadcast = true
                    bind(InetSocketAddress(DISCOVERY_PORT))
                    try { joinGroup(InetAddress.getByName(MULTICAST_ADDR)) }
                    catch (e: Exception) { Log.w(TAG, "Failed to join multicast: ${e.message}") }
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
        if (header.deviceId.contentEquals(deviceId)) return

        val senderId = uuidFromBytes(header.deviceId)
        val payload = data.copyOfRange(PacketHeader.SIZE, PacketHeader.SIZE + header.payloadLength.toInt())

        when (header.getPacketType()) {
            PacketType.Announce -> handleAnnounce(senderId, senderAddr, payload)
            PacketType.AnnounceResponse -> handleAnnounceResponse(senderId, senderAddr, payload)
            PacketType.Bye -> handleBye(senderId)
            PacketType.Text -> handleText(senderId, payload)
            PacketType.FileOffer -> handleFileOffer(senderId, payload, senderAddr)
            PacketType.FileAccept -> handleFileAccept(senderId, payload, senderAddr)
            PacketType.FileReject -> handleFileReject(senderId, payload)
            PacketType.SnippetShare -> handleSnippetShare(senderId, payload)
            else -> {}
        }
    }

    // ── Discovery handlers ──

    /**
     * Received a broadcast from another device.
     * Do NOT add to device list. Only:
     * 1) Send AnnounceResponse
     * 2) Check if we're in their restricted_peers (Rule 2)
     * 3) Update last_broadcast_at if device already known
     */
    private fun handleAnnounce(senderId: String, senderIp: InetAddress, data: ByteArray) {
        try {
            val payload = AnnouncePayload.decode(data)

            // Update last_broadcast_at if already known
            devices[senderId]?.let {
                it.lastBroadcastAt = System.currentTimeMillis() / 1000
                it.lastSeen = it.lastBroadcastAt
            }

            // Rule 2: check if WE are in their restricted_peers
            val selfInList = payload.restrictedPeers.any { it.deviceId == deviceIdStr }
            if (selfInList) {
                val isNew = upsertFromResponse(senderId, payload.deviceName,
                    senderIp.hostAddress ?: "", payload.tcpPort, payload.platform)
                if (isNew) {
                    Log.i(TAG, "Device discovered (self in restricted_peers): ${payload.deviceName}")
                    devices[senderId]?.let { onDeviceDiscovered(it) }
                }

                // Also add other devices from restricted_peers (peer discovery)
                for (peer in payload.restrictedPeers) {
                    if (peer.deviceId == deviceIdStr) continue
                    val existed = devices.containsKey(peer.deviceId)
                    if (existed) {
                        devices[peer.deviceId]?.lastSeen = System.currentTimeMillis() / 1000
                    } else {
                        upsertFromResponse(peer.deviceId, peer.deviceName,
                            peer.ipAddr, peer.tcpPort, peer.platform)
                        Log.i(TAG, "Device discovered (peer in restricted_peers): ${peer.deviceName}")
                        devices[peer.deviceId]?.let { onDeviceDiscovered(it) }
                    }
                }
            }

            // Send AnnounceResponse
            val response = AnnouncePayload(
                deviceName = deviceName,
                platform = "android",
                tcpPort = tcpPort,
                features = 0xFFFFu,
                restrictedPeers = emptyList()
            )
            sendPacket(senderIp, PacketType.AnnounceResponse, response.encode())
        } catch (e: Exception) {
            Log.e(TAG, "Failed to handle announce: ${e.message}")
        }
    }

    /**
     * Received AnnounceResponse — they replied to OUR broadcast (Rule 1).
     */
    private fun handleAnnounceResponse(senderId: String, senderIp: InetAddress, data: ByteArray) {
        try {
            val payload = AnnouncePayload.decode(data)
            val isNew = upsertFromResponse(senderId, payload.deviceName,
                senderIp.hostAddress ?: "", payload.tcpPort, payload.platform)
            if (isNew) {
                Log.i(TAG, "Device discovered (response): ${payload.deviceName}")
                devices[senderId]?.let { onDeviceDiscovered(it) }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to handle announce response: ${e.message}")
        }
    }

    // ── Message handlers ──

    private fun handleBye(senderId: String) {
        devices[senderId]?.let {
            it.isOnline = false
            it.lastBroadcastAt = 0
            onDeviceOffline(senderId)
        }
    }

    private fun handleText(senderId: String, data: ByteArray) {
        try { onTextMessage(senderId, TextPayload.decode(data)) }
        catch (e: Exception) { Log.e(TAG, "Failed to decode text: ${e.message}") }
    }

    private fun handleFileOffer(senderId: String, data: ByteArray, addr: InetAddress) {
        try { onFileOffer(senderId, FileOfferPayload.decode(data), addr) }
        catch (e: Exception) { Log.e(TAG, "Failed to decode file offer: ${e.message}") }
    }

    private fun handleFileAccept(senderId: String, data: ByteArray, addr: InetAddress) {
        try { onFileAccept(senderId, FileAcceptPayload.decode(data), addr) }
        catch (e: Exception) { Log.e(TAG, "Failed to decode file accept: ${e.message}") }
    }

    private fun handleFileReject(senderId: String, data: ByteArray) {
        try { onFileReject(senderId, FileRejectPayload.decode(data)) }
        catch (e: Exception) { Log.e(TAG, "Failed to decode file reject: ${e.message}") }
    }

    private fun handleSnippetShare(senderId: String, data: ByteArray) {
        try { onSnippetShare(senderId, SnippetSharePayload.decode(data)) }
        catch (e: Exception) { Log.e(TAG, "Failed to decode snippet share: ${e.message}") }
    }

    // ── Beacon ──

    private suspend fun runBeacon() {
        delay(1000)
        while (scope.isActive) {
            try {
                // Refresh restricted status before building packet
                refreshRestrictedStatus()

                val restrictedPeers = devices.values
                    .filter { it.isOnline && it.isRestricted }
                    .map { PeerInfo(it.deviceId, it.deviceName, it.ipAddr, it.tcpPort, it.platform) }

                val payload = AnnouncePayload(
                    deviceName = deviceName, platform = "android",
                    tcpPort = tcpPort, features = 0xFFFFu,
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
                    socket?.send(DatagramPacket(packet, packet.size,
                        InetAddress.getByName(MULTICAST_ADDR), DISCOVERY_PORT))
                } catch (_: Exception) {}

                // Broadcast
                try {
                    socket?.send(DatagramPacket(packet, packet.size,
                        InetAddress.getByName("255.255.255.255"), DISCOVERY_PORT))
                } catch (_: Exception) {}
            } catch (e: Exception) {
                Log.e(TAG, "Beacon error: ${e.message}")
            }
            delay(BEACON_INTERVAL_MS)
        }
    }

    private fun refreshRestrictedStatus() {
        val now = System.currentTimeMillis() / 1000
        devices.values.filter { it.isOnline }.forEach { device ->
            device.isRestricted = device.lastBroadcastAt == 0L ||
                (now - device.lastBroadcastAt) > RESTRICTED_THRESHOLD_S
        }
    }

    // ── Timeout checker ──

    private suspend fun runTimeoutChecker() {
        while (scope.isActive) {
            delay(OFFLINE_TIMEOUT_S * 500)
            val now = System.currentTimeMillis() / 1000
            devices.forEach { (id, device) ->
                if (device.isOnline && (now - device.lastSeen) > OFFLINE_TIMEOUT_S) {
                    device.isOnline = false
                    device.lastBroadcastAt = 0
                    onDeviceOffline(id)
                }
            }
        }
    }

    // ── Helpers ──

    private fun upsertFromResponse(id: String, name: String, ip: String, port: UShort, platform: String): Boolean {
        val now = System.currentTimeMillis() / 1000
        val existing = devices[id]
        if (existing != null) {
            existing.deviceName = name
            existing.ipAddr = ip
            existing.lastSeen = now
            existing.isOnline = true
            return false
        }
        devices[id] = DeviceInfo(id, name, ip, port, platform, now, true)
        return true
    }

    private fun uuidFromBytes(bytes: ByteArray): String {
        val bb = java.nio.ByteBuffer.wrap(bytes)
        return UUID(bb.long, bb.long).toString()
    }
}
