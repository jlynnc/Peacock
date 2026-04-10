import Foundation

/// Decodes values from Rust bincode 1.x format.
/// Uses byte-by-byte reading to avoid alignment issues with Data slices.
final class BincodeDecoder {
    private let bytes: [UInt8]
    private(set) var offset: Int = 0

    init(data: Data) {
        self.bytes = [UInt8](data)
    }

    var remaining: Int { bytes.count - offset }

    func decodeString() throws -> String {
        let length = try decodeU64()
        let len = Int(length)
        guard remaining >= len else {
            throw BincodeError.unexpectedEnd
        }
        guard let str = String(bytes: bytes[offset..<(offset + len)], encoding: .utf8) else {
            throw BincodeError.invalidUTF8
        }
        offset += len
        return str
    }

    func decodeU16() throws -> UInt16 {
        guard remaining >= 2 else { throw BincodeError.unexpectedEnd }
        let value = UInt16(bytes[offset]) | UInt16(bytes[offset + 1]) << 8
        offset += 2
        return value
    }

    func decodeU32() throws -> UInt32 {
        guard remaining >= 4 else { throw BincodeError.unexpectedEnd }
        let value = UInt32(bytes[offset])
            | UInt32(bytes[offset + 1]) << 8
            | UInt32(bytes[offset + 2]) << 16
            | UInt32(bytes[offset + 3]) << 24
        offset += 4
        return value
    }

    func decodeU64() throws -> UInt64 {
        guard remaining >= 8 else { throw BincodeError.unexpectedEnd }
        let value = UInt64(bytes[offset])
            | UInt64(bytes[offset + 1]) << 8
            | UInt64(bytes[offset + 2]) << 16
            | UInt64(bytes[offset + 3]) << 24
            | UInt64(bytes[offset + 4]) << 32
            | UInt64(bytes[offset + 5]) << 40
            | UInt64(bytes[offset + 6]) << 48
            | UInt64(bytes[offset + 7]) << 56
        offset += 8
        return value
    }

    func decodeBool() throws -> Bool {
        guard remaining >= 1 else { throw BincodeError.unexpectedEnd }
        let value = bytes[offset]
        offset += 1
        return value != 0
    }
}

enum BincodeError: Error {
    case unexpectedEnd
    case invalidUTF8
    case invalidData
}
