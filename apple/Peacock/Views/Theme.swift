import SwiftUI

extension Color {
    static let peacockTeal = Color(red: 13/255, green: 148/255, blue: 136/255)
    static let peacockTealDark = Color(red: 15/255, green: 118/255, blue: 110/255)
    static let peacockTealLight = Color(red: 13/255, green: 148/255, blue: 136/255).opacity(0.06)

    static let bubbleSent = Color(red: 230/255, green: 255/255, blue: 250/255)
    static let bubbleSentText = Color(red: 19/255, green: 78/255, blue: 74/255)
    static let bubbleReceived = Color(.systemBackground)

    static let textSecondary = Color(.secondaryLabel)
    static let textMuted = Color(.tertiaryLabel)

    static let onlineGreen = Color(red: 34/255, green: 197/255, blue: 94/255)
    static let dangerRed = Color(red: 239/255, green: 68/255, blue: 68/255)
}

extension Font {
    static let chatBody = Font.system(size: 15)
    static let chatTime = Font.system(size: 11)
    static let chatSender = Font.system(size: 12)
}

struct FormatUtils {
    static func fileSize(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter.string(fromByteCount: Int64(bytes))
    }

    static func speed(_ bytesPerSecond: UInt64) -> String {
        fileSize(bytesPerSecond) + "/s"
    }

    static func relativeTime(_ date: Date) -> String {
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter.localizedString(for: date, relativeTo: Date())
    }

    static func fileIcon(for fileName: String) -> String {
        let ext = (fileName as NSString).pathExtension.lowercased()
        switch ext {
        case "jpg", "jpeg", "png", "gif", "webp", "heic", "bmp", "svg":
            return "photo"
        case "mp4", "mov", "avi", "mkv", "wmv":
            return "film"
        case "mp3", "wav", "flac", "aac", "m4a":
            return "music.note"
        case "pdf":
            return "doc.richtext"
        case "doc", "docx":
            return "doc.text"
        case "xls", "xlsx", "csv":
            return "tablecells"
        case "ppt", "pptx":
            return "rectangle.stack"
        case "zip", "rar", "7z", "tar", "gz":
            return "doc.zipper"
        case "txt", "md", "rtf":
            return "doc.plaintext"
        case "swift", "rs", "ts", "js", "py", "java", "c", "cpp", "h":
            return "chevron.left.forwardslash.chevron.right"
        default:
            return "doc"
        }
    }
}
