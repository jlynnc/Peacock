import Foundation

enum PacketBuilder {
    /// Build a complete packet: 32-byte header + payload.
    static func build(type: PacketType, deviceId: [UInt8], payload: Data) -> Data {
        let header = PacketHeader(
            packetType: type,
            deviceId: deviceId,
            payloadLength: UInt32(payload.count)
        )
        var packet = header.toData()
        packet.append(payload)
        return packet
    }

    /// Build a header-only packet (no payload), e.g. Bye.
    static func buildHeaderOnly(type: PacketType, deviceId: [UInt8]) -> Data {
        PacketHeader(packetType: type, deviceId: deviceId, payloadLength: 0).toData()
    }

    /// Parse a raw packet into header + payload data.
    static func parse(_ data: Data) -> (header: PacketHeader, payload: Data)? {
        guard let header = PacketHeader.from(data: data) else { return nil }
        let payloadStart = PacketHeader.size
        let payloadEnd = payloadStart + Int(header.payloadLength)
        guard data.count >= payloadEnd else { return nil }
        let payload = data[payloadStart..<payloadEnd]
        return (header, Data(payload))
    }
}
