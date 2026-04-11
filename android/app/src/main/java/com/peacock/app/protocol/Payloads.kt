package com.peacock.app.protocol

/**
 * All payload types — field order must match Rust's bincode serialization exactly.
 */

data class PeerInfo(
    val deviceId: String,
    val deviceName: String,
    val ipAddr: String,
    val tcpPort: UShort,
    val platform: String
) {
    fun encode(): ByteArray = BincodeEncoder()
        .encodeString(deviceId)
        .encodeString(deviceName)
        .encodeString(ipAddr)
        .encodeU16(tcpPort)
        .encodeString(platform)
        .toByteArray()

    companion object {
        fun decode(decoder: BincodeDecoder): PeerInfo = PeerInfo(
            deviceId = decoder.decodeString(),
            deviceName = decoder.decodeString(),
            ipAddr = decoder.decodeString(),
            tcpPort = decoder.decodeU16(),
            platform = decoder.decodeString()
        )
    }
}

data class AnnouncePayload(
    val deviceName: String,
    val platform: String,
    val tcpPort: UShort,
    val features: UInt,
    val restrictedPeers: List<PeerInfo> = emptyList()
) {
    fun encode(): ByteArray {
        val enc = BincodeEncoder()
            .encodeString(deviceName)
            .encodeString(platform)
            .encodeU16(tcpPort)
            .encodeU32(features)
            .encodeU64(restrictedPeers.size.toULong())
        for (peer in restrictedPeers) {
            enc.encodeBytes(peer.encode())
        }
        return enc.toByteArray()
    }

    companion object {
        fun decode(data: ByteArray): AnnouncePayload {
            val d = BincodeDecoder(data)
            val deviceName = d.decodeString()
            val platform = d.decodeString()
            val tcpPort = d.decodeU16()
            val features = d.decodeU32()
            val peerCount = d.decodeU64().toInt()
            val peers = (0 until peerCount).map { PeerInfo.decode(d) }
            return AnnouncePayload(deviceName, platform, tcpPort, features, peers)
        }
    }
}

data class TextPayload(
    val messageId: String,
    val text: String,
    val timestamp: ULong
) {
    fun encode(): ByteArray = BincodeEncoder()
        .encodeString(messageId)
        .encodeString(text)
        .encodeU64(timestamp)
        .toByteArray()

    companion object {
        fun decode(data: ByteArray): TextPayload {
            val d = BincodeDecoder(data)
            return TextPayload(d.decodeString(), d.decodeString(), d.decodeU64())
        }
    }
}

data class FileOfferPayload(
    val transferId: String,
    val fileName: String,
    val fileSize: ULong,
    val isFolder: Boolean,
    val fileCount: UInt
) {
    fun encode(): ByteArray = BincodeEncoder()
        .encodeString(transferId)
        .encodeString(fileName)
        .encodeU64(fileSize)
        .encodeBool(isFolder)
        .encodeU32(fileCount)
        .toByteArray()

    companion object {
        fun decode(data: ByteArray): FileOfferPayload {
            val d = BincodeDecoder(data)
            return FileOfferPayload(
                d.decodeString(), d.decodeString(), d.decodeU64(),
                d.decodeBool(), d.decodeU32()
            )
        }
    }
}

data class FileAcceptPayload(
    val transferId: String,
    val receiverPort: UShort,
    val resumeOffset: ULong
) {
    fun encode(): ByteArray = BincodeEncoder()
        .encodeString(transferId)
        .encodeU16(receiverPort)
        .encodeU64(resumeOffset)
        .toByteArray()

    companion object {
        fun decode(data: ByteArray): FileAcceptPayload {
            val d = BincodeDecoder(data)
            return FileAcceptPayload(d.decodeString(), d.decodeU16(), d.decodeU64())
        }
    }
}

data class FileRejectPayload(
    val transferId: String
) {
    fun encode(): ByteArray = BincodeEncoder()
        .encodeString(transferId)
        .toByteArray()

    companion object {
        fun decode(data: ByteArray): FileRejectPayload {
            val d = BincodeDecoder(data)
            return FileRejectPayload(d.decodeString())
        }
    }
}

data class SnippetSharePayload(
    val title: String,
    val content: String,
    val tag: String,
    val note: String
) {
    fun encode(): ByteArray = BincodeEncoder()
        .encodeString(title)
        .encodeString(content)
        .encodeString(tag)
        .encodeString(note)
        .toByteArray()

    companion object {
        fun decode(data: ByteArray): SnippetSharePayload {
            val d = BincodeDecoder(data)
            return SnippetSharePayload(
                d.decodeString(), d.decodeString(),
                d.decodeString(), d.decodeString()
            )
        }
    }
}
