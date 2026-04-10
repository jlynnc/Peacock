import Foundation

struct Snippet: Identifiable, Sendable {
    let id: String
    var title: String
    var content: String
    var tag: String
    var note: String
    var sortOrder: Int
    var copyCount: Int
    let createdAt: UInt64
    var updatedAt: UInt64

    var date: Date {
        Date(timeIntervalSince1970: TimeInterval(updatedAt) / 1000.0)
    }

    /// Parse [[text]] markers and return segments.
    var contentSegments: [ContentSegment] {
        var segments: [ContentSegment] = []
        let scanner = Scanner(string: content)
        scanner.charactersToBeSkipped = nil

        while !scanner.isAtEnd {
            if let text = scanner.scanUpToString("[[") {
                if !text.isEmpty {
                    segments.append(.plain(text))
                }
            }
            if scanner.scanString("[[") != nil {
                if let chip = scanner.scanUpToString("]]") {
                    segments.append(.chip(chip))
                    _ = scanner.scanString("]]")
                }
            } else if !scanner.isAtEnd {
                // No more [[ found, consume rest
                let rest = String(content[scanner.currentIndex...])
                if !rest.isEmpty {
                    segments.append(.plain(rest))
                }
                break
            }
        }
        return segments
    }
}

enum ContentSegment: Sendable {
    case plain(String)
    case chip(String)
}

struct SnippetOffer: Identifiable, Sendable {
    let id: String
    let title: String
    let content: String
    let tag: String
    let note: String
    let fromDeviceId: String
    let fromDeviceName: String
}
