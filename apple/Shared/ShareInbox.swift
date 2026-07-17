import Foundation

/// Shared drop-box between the Share Extension and the main app.
/// The extension copies incoming attachments here, then launches the app
/// via the `peacock://share` URL scheme; the app drains the inbox and sends.
enum ShareInbox {
    static let appGroupID = "group.com.peacock.app"
    static let urlScheme = "peacock"
    static let shareHost = "share"

    /// Directory inside the shared App Group container holding pending files.
    static var directory: URL? {
        guard let container = FileManager.default
            .containerURL(forSecurityApplicationGroupIdentifier: appGroupID) else { return nil }
        let dir = container.appendingPathComponent("SharedInbox", isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir
    }

    /// Copy a file into the inbox, keeping its name unique.
    @discardableResult
    static func store(_ source: URL, preferredName: String? = nil) -> URL? {
        guard let dir = directory else { return nil }
        let name = preferredName ?? source.lastPathComponent
        var dest = dir.appendingPathComponent(name)
        var counter = 1
        let base = dest.deletingPathExtension().lastPathComponent
        let ext = dest.pathExtension
        while FileManager.default.fileExists(atPath: dest.path) {
            let newName = ext.isEmpty ? "\(base)(\(counter))" : "\(base)(\(counter)).\(ext)"
            dest = dir.appendingPathComponent(newName)
            counter += 1
        }
        do {
            try FileManager.default.copyItem(at: source, to: dest)
            return dest
        } catch {
            return nil
        }
    }

    /// Write raw data (e.g. shared text) into the inbox.
    @discardableResult
    static func store(data: Data, name: String) -> URL? {
        guard let dir = directory else { return nil }
        let dest = dir.appendingPathComponent(name)
        do {
            try data.write(to: dest)
            return dest
        } catch {
            return nil
        }
    }

    /// All files currently waiting to be sent.
    static func pendingFiles() -> [URL] {
        guard let dir = directory else { return [] }
        let urls = (try? FileManager.default.contentsOfDirectory(
            at: dir, includingPropertiesForKeys: nil, options: [.skipsHiddenFiles])) ?? []
        return urls.sorted { $0.lastPathComponent < $1.lastPathComponent }
    }

    static func remove(_ url: URL) {
        try? FileManager.default.removeItem(at: url)
    }

    static func clear() {
        for url in pendingFiles() { remove(url) }
    }
}
