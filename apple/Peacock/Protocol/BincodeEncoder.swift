import Foundation

/// Encodes values in Rust bincode 1.x format.
/// Strings: 8-byte LE length prefix (u64) + UTF-8 bytes.
/// Integers: little-endian.
/// Bool: 1 byte.
/// Vec<T>: 8-byte LE count (u64) + concatenated elements.
final class BincodeEncoder {
    private(set) var data = Data()

    func encodeString(_ value: String) {
        let utf8 = Array(value.utf8)
        encodeU64(UInt64(utf8.count))
        data.append(contentsOf: utf8)
    }

    func encodeU16(_ value: UInt16) {
        withUnsafeBytes(of: value.littleEndian) { data.append(contentsOf: $0) }
    }

    func encodeU32(_ value: UInt32) {
        withUnsafeBytes(of: value.littleEndian) { data.append(contentsOf: $0) }
    }

    func encodeU64(_ value: UInt64) {
        withUnsafeBytes(of: value.littleEndian) { data.append(contentsOf: $0) }
    }

    func encodeBool(_ value: Bool) {
        data.append(value ? 1 : 0)
    }

    func encodeBytes(_ bytes: [UInt8]) {
        data.append(contentsOf: bytes)
    }
}
