import Foundation

/// 32-byte binary packet header for the Peacock protocol.
/// All multi-byte integers are big-endian.
struct PacketHeader {
    static let size = 32
    static let magic: [UInt8] = [0x50, 0x43, 0x4F, 0x4B] // "PCOK"
    static let protocolVersion: UInt16 = 1

    let version: UInt16
    let packetType: PacketType
    let deviceId: [UInt8] // 16 bytes (UUID)
    let payloadLength: UInt32

    init(packetType: PacketType, deviceId: [UInt8], payloadLength: UInt32) {
        self.version = Self.protocolVersion
        self.packetType = packetType
        self.deviceId = deviceId
        self.payloadLength = payloadLength
    }

    func toData() -> Data {
        var data = Data(capacity: Self.size)
        data.append(contentsOf: Self.magic)
        withUnsafeBytes(of: version.bigEndian) { data.append(contentsOf: $0) }
        withUnsafeBytes(of: packetType.rawValue.bigEndian) { data.append(contentsOf: $0) }
        data.append(contentsOf: deviceId)
        withUnsafeBytes(of: payloadLength.bigEndian) { data.append(contentsOf: $0) }
        data.append(contentsOf: [0, 0, 0, 0]) // reserved
        return data
    }

    static func from(data: Data) -> PacketHeader? {
        guard data.count >= size else { return nil }
        let bytes = [UInt8](data)
        guard bytes[0] == magic[0], bytes[1] == magic[1],
              bytes[2] == magic[2], bytes[3] == magic[3] else { return nil }

        let version = UInt16(bytes[4]) << 8 | UInt16(bytes[5])
        let typeRaw = UInt16(bytes[6]) << 8 | UInt16(bytes[7])
        guard let packetType = PacketType(rawValue: typeRaw) else { return nil }
        let deviceId = Array(bytes[8..<24])
        let payloadLength = UInt32(bytes[24]) << 24 | UInt32(bytes[25]) << 16 |
                            UInt32(bytes[26]) << 8 | UInt32(bytes[27])

        return PacketHeader(packetType: packetType, deviceId: deviceId, payloadLength: payloadLength)
    }
}
