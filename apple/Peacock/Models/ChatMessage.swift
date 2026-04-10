import Foundation

struct ChatMessage: Identifiable, Sendable {
    let id: String
    let deviceId: String
    let direction: MessageDirection
    let content: String
    let msgType: MessageType
    let timestamp: UInt64 // milliseconds since epoch
    var status: MessageStatus

    var date: Date {
        Date(timeIntervalSince1970: TimeInterval(timestamp) / 1000.0)
    }
}

enum MessageDirection: String, Sendable, Codable {
    case sent
    case received
}

enum MessageType: String, Sendable, Codable {
    case text
    case file
    case snippet
}

enum MessageStatus: String, Sendable, Codable {
    case sending
    case sent
    case failed
}

struct Conversation: Sendable {
    var messages: [ChatMessage] = []
    var unreadCount: Int = 0
    var lastMessage: ChatMessage? { messages.last }
}
