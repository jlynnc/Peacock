package com.peacock.app.protocol

import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * Bincode 1.x compatible encoder.
 * - Strings: 8-byte LE length prefix (u64) + UTF-8 bytes
 * - u16/u32/u64: little-endian
 * - bool: 1 byte (0 or 1)
 * - Vec<T>: 8-byte LE length prefix (u64) + concatenated elements
 */
class BincodeEncoder {
    private val out = ByteArrayOutputStream()

    fun encodeString(s: String): BincodeEncoder {
        val bytes = s.toByteArray(Charsets.UTF_8)
        encodeU64(bytes.size.toULong())
        out.write(bytes)
        return this
    }

    fun encodeU16(v: UShort): BincodeEncoder {
        val buf = ByteBuffer.allocate(2).order(ByteOrder.LITTLE_ENDIAN)
        buf.putShort(v.toShort())
        out.write(buf.array())
        return this
    }

    fun encodeU32(v: UInt): BincodeEncoder {
        val buf = ByteBuffer.allocate(4).order(ByteOrder.LITTLE_ENDIAN)
        buf.putInt(v.toInt())
        out.write(buf.array())
        return this
    }

    fun encodeU64(v: ULong): BincodeEncoder {
        val buf = ByteBuffer.allocate(8).order(ByteOrder.LITTLE_ENDIAN)
        buf.putLong(v.toLong())
        out.write(buf.array())
        return this
    }

    fun encodeBool(v: Boolean): BincodeEncoder {
        out.write(if (v) 1 else 0)
        return this
    }

    fun encodeBytes(bytes: ByteArray): BincodeEncoder {
        out.write(bytes)
        return this
    }

    fun toByteArray(): ByteArray = out.toByteArray()
}
