package com.peacock.app.protocol

import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * Bincode 1.x compatible decoder.
 */
class BincodeDecoder(private val data: ByteArray) {
    private var pos = 0

    fun decodeString(): String {
        val len = decodeU64().toInt()
        if (pos + len > data.size) throw IllegalStateException("String length $len exceeds data at pos $pos")
        val s = String(data, pos, len, Charsets.UTF_8)
        pos += len
        return s
    }

    fun decodeU16(): UShort {
        if (pos + 2 > data.size) throw IllegalStateException("Not enough data for u16 at pos $pos")
        val buf = ByteBuffer.wrap(data, pos, 2).order(ByteOrder.LITTLE_ENDIAN)
        pos += 2
        return buf.short.toUShort()
    }

    fun decodeU32(): UInt {
        if (pos + 4 > data.size) throw IllegalStateException("Not enough data for u32 at pos $pos")
        val buf = ByteBuffer.wrap(data, pos, 4).order(ByteOrder.LITTLE_ENDIAN)
        pos += 4
        return buf.int.toUInt()
    }

    fun decodeU64(): ULong {
        if (pos + 8 > data.size) throw IllegalStateException("Not enough data for u64 at pos $pos")
        val buf = ByteBuffer.wrap(data, pos, 8).order(ByteOrder.LITTLE_ENDIAN)
        pos += 8
        return buf.long.toULong()
    }

    fun decodeBool(): Boolean {
        if (pos >= data.size) throw IllegalStateException("Not enough data for bool at pos $pos")
        val v = data[pos].toInt() != 0
        pos += 1
        return v
    }

    fun remaining(): Int = data.size - pos
}
