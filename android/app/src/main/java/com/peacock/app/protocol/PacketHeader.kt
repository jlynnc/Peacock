package com.peacock.app.protocol

import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * 32-byte packet header — binary compatible with Rust/Swift implementations.
 * All multi-byte fields are big-endian.
 */
data class PacketHeader(
    val version: UShort,
    val packetType: UShort,
    val deviceId: ByteArray,  // 16 bytes
    val payloadLength: UInt
) {
    companion object {
        const val SIZE = 32
        val MAGIC = byteArrayOf(0x50, 0x43, 0x4F, 0x4B) // "PCOK"
        const val PROTOCOL_VERSION: UShort = 1u

        fun fromBytes(data: ByteArray): PacketHeader? {
            if (data.size < SIZE) return null
            // Check magic
            if (data[0] != MAGIC[0] || data[1] != MAGIC[1] ||
                data[2] != MAGIC[2] || data[3] != MAGIC[3]) return null

            val buf = ByteBuffer.wrap(data).order(ByteOrder.BIG_ENDIAN)
            buf.position(4)
            val version = buf.short.toUShort()
            val packetType = buf.short.toUShort()
            val deviceId = ByteArray(16)
            buf.get(deviceId)
            val payloadLength = buf.int.toUInt()

            return PacketHeader(version, packetType, deviceId, payloadLength)
        }
    }

    fun toBytes(): ByteArray {
        val buf = ByteBuffer.allocate(SIZE).order(ByteOrder.BIG_ENDIAN)
        buf.put(MAGIC)
        buf.putShort(version.toShort())
        buf.putShort(packetType.toShort())
        buf.put(deviceId)
        buf.putInt(payloadLength.toInt())
        buf.put(ByteArray(4)) // reserved
        return buf.array()
    }

    fun isValid(): Boolean = version == PROTOCOL_VERSION

    fun getPacketType(): PacketType? = PacketType.fromValue(packetType)

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is PacketHeader) return false
        return version == other.version && packetType == other.packetType &&
            deviceId.contentEquals(other.deviceId) && payloadLength == other.payloadLength
    }

    override fun hashCode(): Int = deviceId.contentHashCode()
}
