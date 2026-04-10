import Foundation

struct DeviceInfo: Identifiable, Sendable {
    let deviceId: String
    var deviceName: String
    var ipAddr: String
    var tcpPort: UInt16
    var platform: String
    var lastSeen: Date
    var isOnline: Bool

    var id: String { deviceId }

    var platformEmoji: String {
        switch platform {
        case "ios": return "📱"
        case "macos": return "💻"
        case "windows": return "🖥️"
        case "linux": return "🐧"
        case "android": return "🤖"
        default: return "❓"
        }
    }

    var platformIcon: String {
        switch platform {
        case "ios": return "iphone"
        case "macos": return "laptopcomputer"
        case "windows": return "desktopcomputer"
        case "linux": return "terminal"
        case "android": return "phone"
        default: return "questionmark.circle"
        }
    }
}
