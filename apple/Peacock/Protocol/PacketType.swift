import Foundation

enum PacketType: UInt16, Sendable {
    case announce = 1
    case bye = 2
    case announceResponse = 3
    case text = 10
    case fileOffer = 20
    case fileAccept = 21
    case fileReject = 22
    case fileChunk = 23
    case clipboard = 30
    case snippetShare = 31
    case ack = 99
}
