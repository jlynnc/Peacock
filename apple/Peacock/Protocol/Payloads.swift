import Foundation

// MARK: - Announce

struct AnnouncePayload: Sendable {
    let deviceName: String
    let platform: String
    let tcpPort: UInt16
    let features: UInt32
    let restrictedPeers: [PeerInfo]

    func encode() -> Data {
        let enc = BincodeEncoder()
        enc.encodeString(deviceName)
        enc.encodeString(platform)
        enc.encodeU16(tcpPort)
        enc.encodeU32(features)
        enc.encodeU64(UInt64(restrictedPeers.count))
        for peer in restrictedPeers {
            peer.encode(to: enc)
        }
        return enc.data
    }

    static func decode(from data: Data) throws -> AnnouncePayload {
        let dec = BincodeDecoder(data: data)
        let deviceName = try dec.decodeString()
        let platform = try dec.decodeString()
        let tcpPort = try dec.decodeU16()
        let features = try dec.decodeU32()
        let peerCount = try dec.decodeU64()
        var peers: [PeerInfo] = []
        for _ in 0..<peerCount {
            peers.append(try PeerInfo.decode(from: dec))
        }
        return AnnouncePayload(deviceName: deviceName, platform: platform,
                               tcpPort: tcpPort, features: features,
                               restrictedPeers: peers)
    }
}

struct PeerInfo: Sendable {
    let deviceId: String
    let deviceName: String
    let ipAddr: String
    let tcpPort: UInt16
    let platform: String

    func encode(to enc: BincodeEncoder) {
        enc.encodeString(deviceId)
        enc.encodeString(deviceName)
        enc.encodeString(ipAddr)
        enc.encodeU16(tcpPort)
        enc.encodeString(platform)
    }

    static func decode(from dec: BincodeDecoder) throws -> PeerInfo {
        PeerInfo(
            deviceId: try dec.decodeString(),
            deviceName: try dec.decodeString(),
            ipAddr: try dec.decodeString(),
            tcpPort: try dec.decodeU16(),
            platform: try dec.decodeString()
        )
    }
}

// MARK: - Text

struct TextPayload: Sendable {
    let messageId: String
    let text: String
    let timestamp: UInt64

    func encode() -> Data {
        let enc = BincodeEncoder()
        enc.encodeString(messageId)
        enc.encodeString(text)
        enc.encodeU64(timestamp)
        return enc.data
    }

    static func decode(from data: Data) throws -> TextPayload {
        let dec = BincodeDecoder(data: data)
        return TextPayload(
            messageId: try dec.decodeString(),
            text: try dec.decodeString(),
            timestamp: try dec.decodeU64()
        )
    }
}

// MARK: - File Transfer

struct FileOfferPayload: Sendable {
    let transferId: String
    let fileName: String
    let fileSize: UInt64
    let isFolder: Bool
    let fileCount: UInt32

    func encode() -> Data {
        let enc = BincodeEncoder()
        enc.encodeString(transferId)
        enc.encodeString(fileName)
        enc.encodeU64(fileSize)
        enc.encodeBool(isFolder)
        enc.encodeU32(fileCount)
        return enc.data
    }

    static func decode(from data: Data) throws -> FileOfferPayload {
        let dec = BincodeDecoder(data: data)
        return FileOfferPayload(
            transferId: try dec.decodeString(),
            fileName: try dec.decodeString(),
            fileSize: try dec.decodeU64(),
            isFolder: try dec.decodeBool(),
            fileCount: try dec.decodeU32()
        )
    }
}

struct FileAcceptPayload: Sendable {
    let transferId: String
    let receiverPort: UInt16
    let resumeOffset: UInt64

    func encode() -> Data {
        let enc = BincodeEncoder()
        enc.encodeString(transferId)
        enc.encodeU16(receiverPort)
        enc.encodeU64(resumeOffset)
        return enc.data
    }

    static func decode(from data: Data) throws -> FileAcceptPayload {
        let dec = BincodeDecoder(data: data)
        return FileAcceptPayload(
            transferId: try dec.decodeString(),
            receiverPort: try dec.decodeU16(),
            resumeOffset: try dec.decodeU64()
        )
    }
}

struct FileRejectPayload: Sendable {
    let transferId: String

    func encode() -> Data {
        let enc = BincodeEncoder()
        enc.encodeString(transferId)
        return enc.data
    }

    static func decode(from data: Data) throws -> FileRejectPayload {
        let dec = BincodeDecoder(data: data)
        return FileRejectPayload(transferId: try dec.decodeString())
    }
}

// MARK: - Snippet

struct SnippetSharePayload: Sendable {
    let title: String
    let content: String
    let tag: String
    let note: String

    func encode() -> Data {
        let enc = BincodeEncoder()
        enc.encodeString(title)
        enc.encodeString(content)
        enc.encodeString(tag)
        enc.encodeString(note)
        return enc.data
    }

    static func decode(from data: Data) throws -> SnippetSharePayload {
        let dec = BincodeDecoder(data: data)
        return SnippetSharePayload(
            title: try dec.decodeString(),
            content: try dec.decodeString(),
            tag: try dec.decodeString(),
            note: try dec.decodeString()
        )
    }
}
