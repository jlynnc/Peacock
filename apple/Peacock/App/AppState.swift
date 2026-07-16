import Foundation
import SwiftUI
import Combine
import os.log

private let log = Logger(subsystem: "com.peacock.app", category: "AppState")

/// Central app state that coordinates networking, storage, and UI.
@MainActor
final class AppState: ObservableObject {
    // Identity
    let deviceId: String
    let deviceIdBytes: [UInt8]
    @Published var deviceName: String

    // Managers
    let discovery: DiscoveryManager
    let transferManager: FileTransferManager
    let db: PeacockDatabase
    private var cancellables = Set<AnyCancellable>()

    // Chat
    @Published var conversations: [String: Conversation] = [:]
    @Published var selectedDeviceId: String?

    // Snippets
    @Published var snippets: [Snippet] = []
    @Published var selectedSnippetId: String?

    // Snippet offers pending acceptance
    @Published var pendingSnippetOffers: [SnippetOffer] = []

    // File offers pending acceptance
    @Published var pendingFileOffers: [FileOfferInfo] = []

    // Files handed over by the Share Extension, awaiting a device choice
    @Published var pendingSharedFiles: [URL] = []

    // Settings
    @Published var autoAccept: Bool = false
    @Published var maxConcurrent: Int = 10
    @Published var theme: AppTheme = .system
    let locale: LocaleManager

    // Network
    var udpService: UDPService!
    private var beaconTimer: Timer?
    private var timeoutTimer: Timer?

    var selectedDevice: DeviceInfo? {
        guard let id = selectedDeviceId else { return nil }
        return discovery.getDevice(id)
    }

    var totalUnread: Int {
        conversations.values.reduce(0) { $0 + $1.unreadCount }
    }

    init() {
        let db: PeacockDatabase
        do {
            db = try PeacockDatabase()
        } catch {
            fatalError("Failed to open database: \(error)")
        }
        self.db = db

        // Load or create device ID (lowercase to match Windows/Android wire format)
        let storedId = try? db.getSetting("device_id")
        let id = (storedId ?? UUID().uuidString).lowercased()
        if storedId == nil {
            try? db.setSetting("device_id", value: id)
        }
        self.deviceId = id
        self.deviceIdBytes = NetworkUtils.uuidToBytes(id)

        // Load device name.
        // iOS 16+ returns a generic model name ("iPhone") from UIDevice.name for
        // privacy, so every iPhone looks identical on other devices. When the name
        // is generic, append a short suffix from the device ID so they're unique.
        // A real custom name (older iOS / with entitlement) is kept as-is.
        let suffix = String(id.replacingOccurrences(of: "-", with: "").suffix(4)).uppercased()
        #if os(iOS)
        let genericName = UIDevice.current.model        // "iPhone" / "iPad"
        let rawName = UIDevice.current.name
        let uniqueName = (rawName == genericName) ? "\(rawName)-\(suffix)" : rawName
        #else
        let genericName = "Mac"
        let uniqueName = Host.current().localizedName ?? "Mac-\(suffix)"
        #endif
        let storedName = try? db.getSetting("device_name")
        let resolvedName: String
        if let stored = storedName, stored != genericName {
            resolvedName = stored                       // real/custom name → keep
        } else {
            resolvedName = uniqueName                    // fresh install or bare generic → make unique
            try? db.setSetting("device_name", value: resolvedName)
        }
        self.deviceName = resolvedName

        let disc = DiscoveryManager()
        self.discovery = disc
        let xfer = FileTransferManager()
        self.transferManager = xfer

        // Load settings
        self.locale = LocaleManager()
        if let localeStr = try? db.getSetting("locale"), let loc = AppLocale(rawValue: localeStr) {
            self.locale.current = loc
        }
        if let themeStr = try? db.getSetting("theme"), let th = AppTheme(rawValue: themeStr) {
            self.theme = th
        }
        self.autoAccept = (try? db.getSetting("auto_accept")) == "true"
        if let maxStr = try? db.getSetting("max_concurrent"), let m = Int(maxStr) {
            self.maxConcurrent = m
        }

        // Load snippets
        self.snippets = (try? db.getAllSnippets()) ?? []

        // Setup UDP
        self.udpService = UDPService(
            deviceIdBytes: deviceIdBytes,
            deviceIdString: deviceId
        ) { [weak self] header, payload, sourceIP in
            guard let self else { return }
            Task { @MainActor in
                self.handlePacket(header: header, payload: payload, sourceIP: sourceIP)
            }
        }

        // Forward child objectWillChange → AppState objectWillChange
        disc.objectWillChange.sink { [weak self] _ in
            self?.objectWillChange.send()
        }.store(in: &cancellables)

        xfer.objectWillChange.sink { [weak self] _ in
            self?.objectWillChange.send()
        }.store(in: &cancellables)
    }

    // MARK: - Lifecycle

    func start() {
        guard !isStarted else { return }
        isStarted = true
        udpService.startListening()
        startBeacon()
        startTimeoutChecker()

        // Restart UDP listener when app returns to foreground (iOS suspends sockets on lock)
        #if os(iOS)
        NotificationCenter.default.addObserver(
            forName: UIApplication.willEnterForegroundNotification,
            object: nil, queue: .main
        ) { [weak self] _ in
            guard let self else { return }
            self.udpService.stopListening()
            self.udpService.startListening()
            // Send an immediate announce so others discover us quickly
            self.sendAnnounce()
        }
        #endif
    }
    private var isStarted = false

    func stop() {
        beaconTimer?.invalidate()
        timeoutTimer?.invalidate()
        #if os(iOS)
        NotificationCenter.default.removeObserver(self, name: UIApplication.willEnterForegroundNotification, object: nil)
        #endif
        udpService.sendBye()
        udpService.stopListening()
    }

    // MARK: - Beacon

    private func startBeacon() {
        sendAnnounce()
        beaconTimer = Timer.scheduledTimer(withTimeInterval: NetworkConstants.beaconInterval, repeats: true) { [weak self] _ in
            guard let self else { return }
            Task { @MainActor in self.sendAnnounce() }
        }
    }

    private func sendAnnounce() {
        let payload = AnnouncePayload(
            deviceName: deviceName,
            platform: NetworkUtils.currentPlatform,
            tcpPort: NetworkConstants.tcpPort,
            features: NetworkConstants.features,
            restrictedPeers: discovery.getRestrictedPeers()
        )
        udpService.sendAnnounce(payload: payload)
    }

    private func startTimeoutChecker() {
        timeoutTimer = Timer.scheduledTimer(withTimeInterval: NetworkConstants.timeoutCheckInterval, repeats: true) { [weak self] _ in
            guard let self else { return }
            Task { @MainActor in
                _ = self.discovery.checkTimeouts()
            }
        }
    }

    // MARK: - Packet Handling

    private func handlePacket(header: PacketHeader, payload: Data, sourceIP: String) {
        let senderDeviceId = NetworkUtils.bytesToUUID(header.deviceId)

        switch header.packetType {
        case .announce:
            handleAnnounce(payload: payload, sourceIP: sourceIP, senderDeviceId: senderDeviceId)
        case .announceResponse:
            handleAnnounceResponse(payload: payload, sourceIP: sourceIP, senderDeviceId: senderDeviceId)
        case .bye:
            discovery.markOffline(senderDeviceId)
        case .text:
            handleText(payload: payload, senderDeviceId: senderDeviceId)
        case .fileOffer:
            handleFileOffer(payload: payload, senderDeviceId: senderDeviceId)
        case .fileAccept:
            handleFileAccept(payload: payload, senderDeviceId: senderDeviceId)
        case .fileReject:
            handleFileReject(payload: payload)
        case .snippetShare:
            handleSnippetShare(payload: payload, senderDeviceId: senderDeviceId)
        default:
            break
        }
    }

    private func handleAnnounce(payload: Data, sourceIP: String, senderDeviceId: String) {
        guard let announce = try? AnnouncePayload.decode(from: payload) else {
            log.error("[Peacock] Failed to decode Announce from \(sourceIP)")
            return
        }

        // Record that we received their broadcast (for restricted status)
        discovery.noteReceivedBroadcast(from: senderDeviceId)

        // If device is already in our list, update lastSeen
        discovery.touchDevice(senderDeviceId)

        let broadcaster = DeviceInfo(
            deviceId: senderDeviceId,
            deviceName: announce.deviceName,
            ipAddr: sourceIP,
            tcpPort: announce.tcpPort,
            platform: announce.platform,
            lastSeen: Date(),
            isOnline: true
        )

        // Rule 2: Check if MY device ID is in their restricted_peers → add them
        let added = discovery.checkSelfInRestrictedPeers(
            broadcaster: broadcaster,
            restrictedPeers: announce.restrictedPeers,
            ownDeviceId: deviceId
        )
        if added {
            try? db.saveKnownDevice(broadcaster)

            // Learn about other restricted peers so two restricted devices
            // can discover each other through this shared broadcaster.
            for peer in announce.restrictedPeers where peer.deviceId != deviceId {
                if discovery.addPeerFromRestrictedList(peer),
                   let d = discovery.getDevice(peer.deviceId) {
                    try? db.saveKnownDevice(d)
                }
            }
        }

        // Always send AnnounceResponse so they can discover us via Rule 1
        let response = AnnouncePayload(
            deviceName: deviceName,
            platform: NetworkUtils.currentPlatform,
            tcpPort: NetworkConstants.tcpPort,
            features: NetworkConstants.features,
            restrictedPeers: []
        )
        udpService.sendAnnounceResponse(payload: response, to: sourceIP)
    }

    private func handleAnnounceResponse(payload: Data, sourceIP: String, senderDeviceId: String) {
        guard let announce = try? AnnouncePayload.decode(from: payload) else { return }

        let device = DeviceInfo(
            deviceId: senderDeviceId,
            deviceName: announce.deviceName,
            ipAddr: sourceIP,
            tcpPort: announce.tcpPort,
            platform: announce.platform,
            lastSeen: Date(),
            isOnline: true
        )

        // Rule 1: Someone responded to our broadcast → add them
        let isNew = discovery.addDeviceFromResponse(device)
        if isNew {
            try? db.saveKnownDevice(device)
        }
    }

    private func handleText(payload: Data, senderDeviceId: String) {
        guard let text = try? TextPayload.decode(from: payload) else { return }

        let message = ChatMessage(
            id: text.messageId,
            deviceId: senderDeviceId,
            direction: .received,
            content: text.text,
            msgType: .text,
            timestamp: text.timestamp > 0 ? text.timestamp : UInt64(Date().timeIntervalSince1970 * 1000),
            status: .sent
        )

        addMessage(message, for: senderDeviceId)
        try? db.storeMessage(message)
    }

    private func handleFileOffer(payload: Data, senderDeviceId: String) {
        guard let offer = try? FileOfferPayload.decode(from: payload) else { return }

        let _ = transferManager.createReceiveTask(
            transferId: offer.transferId,
            deviceId: senderDeviceId,
            fileName: offer.fileName,
            fileSize: offer.fileSize,
            isFolder: offer.isFolder,
            fileCount: offer.fileCount
        )

        let deviceName = discovery.getDevice(senderDeviceId)?.deviceName ?? "Unknown"
        pendingFileOffers.append(FileOfferInfo(
            transferId: offer.transferId,
            fileName: offer.fileName,
            fileSize: offer.fileSize,
            isFolder: offer.isFolder,
            fileCount: offer.fileCount,
            fromDeviceId: senderDeviceId,
            fromDeviceName: deviceName
        ))

        // Add file message to chat
        let message = ChatMessage(
            id: UUID().uuidString,
            deviceId: senderDeviceId,
            direction: .received,
            content: offer.transferId,
            msgType: .file,
            timestamp: UInt64(Date().timeIntervalSince1970 * 1000),
            status: .sent
        )
        addMessage(message, for: senderDeviceId)
    }

    private func handleFileAccept(payload: Data, senderDeviceId: String) {
        guard let accept = try? FileAcceptPayload.decode(from: payload) else {
            log.error("[Peacock] handleFileAccept: decode failed")
            return
        }
        guard let device = discovery.getDevice(senderDeviceId) else {
            log.error("[Peacock] handleFileAccept: device not found")
            return
        }

        transferManager.startSending(
            transferId: accept.transferId,
            toHost: device.ipAddr,
            port: accept.receiverPort,
            resumeOffset: accept.resumeOffset
        )
    }

    private func handleFileReject(payload: Data) {
        guard let reject = try? FileRejectPayload.decode(from: payload) else { return }
        transferManager.tasks[reject.transferId]?.status = .rejected
    }

    private func handleSnippetShare(payload: Data, senderDeviceId: String) {
        guard let snippet = try? SnippetSharePayload.decode(from: payload) else { return }

        let deviceName = discovery.getDevice(senderDeviceId)?.deviceName ?? "Unknown"
        let offer = SnippetOffer(
            id: UUID().uuidString,
            title: snippet.title,
            content: snippet.content,
            tag: snippet.tag,
            note: snippet.note,
            fromDeviceId: senderDeviceId,
            fromDeviceName: deviceName
        )
        pendingSnippetOffers.append(offer)

        // Also add as chat message
        let message = ChatMessage(
            id: UUID().uuidString,
            deviceId: senderDeviceId,
            direction: .received,
            content: offer.id,
            msgType: .snippet,
            timestamp: UInt64(Date().timeIntervalSince1970 * 1000),
            status: .sent
        )
        addMessage(message, for: senderDeviceId)
    }

    // MARK: - Actions

    func sendMessage(to deviceId: String, text: String) {
        guard let device = discovery.getDevice(deviceId) else { return }

        let messageId = UUID().uuidString
        let timestamp = UInt64(Date().timeIntervalSince1970 * 1000)
        let payload = TextPayload(messageId: messageId, text: text, timestamp: timestamp, targetDeviceId: deviceId)

        udpService.sendText(payload, to: device.ipAddr)

        let message = ChatMessage(
            id: messageId,
            deviceId: deviceId,
            direction: .sent,
            content: text,
            msgType: .text,
            timestamp: timestamp,
            status: .sent
        )

        addMessage(message, for: deviceId)
        try? db.storeMessage(message)
    }

    func sendFile(to deviceId: String, url: URL) {
        guard let device = discovery.getDevice(deviceId) else { return }

        guard let attrs = try? FileManager.default.attributesOfItem(atPath: url.path) else {
            log.error("[Peacock] sendFile: can't read file attrs at \(url.path)")
            return
        }
        let fileSize = (attrs[.size] as? UInt64) ?? 0
        let fileName = url.lastPathComponent

        let task = transferManager.createSendTask(
            deviceId: deviceId,
            fileName: fileName,
            filePath: url.path,
            fileSize: fileSize
        )

        let offer = FileOfferPayload(
            transferId: task.transferId,
            fileName: fileName,
            fileSize: fileSize,
            isFolder: false,
            fileCount: 1
        )
        udpService.sendFileOffer(offer, to: device.ipAddr)

        // Add file message to chat
        let message = ChatMessage(
            id: UUID().uuidString,
            deviceId: deviceId,
            direction: .sent,
            content: task.transferId,
            msgType: .file,
            timestamp: UInt64(Date().timeIntervalSince1970 * 1000),
            status: .sent
        )
        addMessage(message, for: deviceId)
    }

    // MARK: - Share Extension inbox

    func loadSharedInbox() {
        pendingSharedFiles = ShareInbox.pendingFiles()
    }

    /// Send everything the Share Extension dropped off to the chosen device.
    func sendSharedFiles(to deviceId: String) {
        for src in pendingSharedFiles {
            // Copy out of the App Group container first — the transfer reads the file
            // asynchronously, so it must outlive clearing the inbox.
            let dest = FileManager.default.temporaryDirectory
                .appendingPathComponent(src.lastPathComponent)
            try? FileManager.default.removeItem(at: dest)
            guard (try? FileManager.default.copyItem(at: src, to: dest)) != nil else { continue }
            sendFile(to: deviceId, url: dest)
        }
        ShareInbox.clear()
        pendingSharedFiles = []
    }

    func discardSharedFiles() {
        ShareInbox.clear()
        pendingSharedFiles = []
    }

    func acceptTransfer(_ transferId: String) {
        guard let task = transferManager.getTask(transferId),
              let device = discovery.getDevice(task.deviceId) else {
            log.error("[Peacock] acceptTransfer: task or device not found for \(transferId)")
            return
        }

        Task {
            do {
                let port = try await transferManager.startReceiving(transferId: transferId)
                let accept = FileAcceptPayload(
                    transferId: transferId,
                    receiverPort: port,
                    resumeOffset: task.resumeOffset
                )
                udpService.sendFileAccept(accept, to: device.ipAddr)
            } catch {
                log.error("[Peacock] Accept failed: \(error.localizedDescription)")
                task.status = .failed
            }
        }

        pendingFileOffers.removeAll { $0.transferId == transferId }
    }

    func rejectTransfer(_ transferId: String) {
        guard let task = transferManager.getTask(transferId),
              let device = discovery.getDevice(task.deviceId) else { return }

        task.status = .rejected
        let reject = FileRejectPayload(transferId: transferId)
        udpService.sendFileReject(reject, to: device.ipAddr)
        pendingFileOffers.removeAll { $0.transferId == transferId }
    }

    func shareSnippet(to deviceId: String, snippet: Snippet) {
        guard let device = discovery.getDevice(deviceId) else { return }

        let payload = SnippetSharePayload(
            title: snippet.title,
            content: snippet.content,
            tag: snippet.tag,
            note: snippet.note
        )
        udpService.sendSnippetShare(payload, to: device.ipAddr)

        // Add message to chat
        let message = ChatMessage(
            id: UUID().uuidString,
            deviceId: deviceId,
            direction: .sent,
            content: "snippet",
            msgType: .snippet,
            timestamp: UInt64(Date().timeIntervalSince1970 * 1000),
            status: .sent
        )
        addMessage(message, for: deviceId)
    }

    func acceptSnippetOffer(_ offerId: String) {
        guard let offer = pendingSnippetOffers.first(where: { $0.id == offerId }) else { return }
        let id = UUID().uuidString
        try? db.createSnippet(id: id, title: offer.title, content: offer.content,
                              tag: offer.tag, note: offer.note)
        snippets = (try? db.getAllSnippets()) ?? snippets
        pendingSnippetOffers.removeAll { $0.id == offerId }
    }

    func rejectSnippetOffer(_ offerId: String) {
        pendingSnippetOffers.removeAll { $0.id == offerId }
    }

    // MARK: - Snippet CRUD

    func createSnippet() {
        let id = UUID().uuidString
        try? db.createSnippet(id: id, title: "新建片段", content: "", tag: "", note: "")
        snippets = (try? db.getAllSnippets()) ?? snippets
        selectedSnippetId = id
    }

    func updateSnippet(_ snippet: Snippet) {
        try? db.updateSnippet(id: snippet.id, title: snippet.title,
                              content: snippet.content, tag: snippet.tag, note: snippet.note)
        if let idx = snippets.firstIndex(where: { $0.id == snippet.id }) {
            var updated = snippet
            updated.updatedAt = UInt64(Date().timeIntervalSince1970 * 1000)
            snippets[idx] = updated
        }
    }

    func deleteSnippet(_ id: String) {
        try? db.deleteSnippet(id: id)
        snippets.removeAll { $0.id == id }
        if selectedSnippetId == id {
            selectedSnippetId = nil
        }
    }

    func renameSnippet(_ id: String, title: String) {
        if let idx = snippets.firstIndex(where: { $0.id == id }) {
            snippets[idx].title = title
            snippets[idx].updatedAt = UInt64(Date().timeIntervalSince1970 * 1000)
            try? db.updateSnippet(id: id, title: title,
                                  content: snippets[idx].content,
                                  tag: snippets[idx].tag, note: snippets[idx].note)
        }
    }

    func pinSnippetToTop(_ id: String) {
        // Set sort_order to -1 (lower = higher priority), then reorder all
        guard let idx = snippets.firstIndex(where: { $0.id == id }) else { return }
        let snippet = snippets.remove(at: idx)
        snippets.insert(snippet, at: 0)
        let ids = snippets.map(\.id)
        try? db.reorderSnippets(ids: ids)
        // Refresh sort_order values
        for (i, _) in snippets.enumerated() {
            snippets[i].sortOrder = i
        }
    }

    func updateDeviceName(_ name: String) {
        deviceName = name
        try? db.setSetting("device_name", value: name)
    }

    func updateAutoAccept(_ value: Bool) {
        autoAccept = value
        try? db.setSetting("auto_accept", value: value ? "true" : "false")
    }

    func updateMaxConcurrent(_ value: Int) {
        maxConcurrent = value
        try? db.setSetting("max_concurrent", value: "\(value)")
    }

    func updateTheme(_ value: AppTheme) {
        theme = value
        try? db.setSetting("theme", value: value.rawValue)
    }

    func updateLocale(_ value: AppLocale) {
        locale.current = value
        try? db.setSetting("locale", value: value.rawValue)
    }

    // MARK: - Helpers

    func addMessage(_ message: ChatMessage, for deviceId: String) {
        if conversations[deviceId] == nil {
            conversations[deviceId] = Conversation()
        }
        conversations[deviceId]?.messages.append(message)
        if message.direction == .received && selectedDeviceId != deviceId {
            conversations[deviceId]?.unreadCount += 1
        }
    }

    func loadHistory(for deviceId: String) {
        guard conversations[deviceId] == nil || conversations[deviceId]?.messages.isEmpty == true else { return }
        if let messages = try? db.getMessages(deviceId: deviceId) {
            conversations[deviceId] = Conversation(messages: messages)
        }
    }

    func clearUnread(for deviceId: String) {
        conversations[deviceId]?.unreadCount = 0
    }
}

struct FileOfferInfo: Identifiable, Sendable {
    let transferId: String
    let fileName: String
    let fileSize: UInt64
    let isFolder: Bool
    let fileCount: UInt32
    let fromDeviceId: String
    let fromDeviceName: String

    var id: String { transferId }
}
