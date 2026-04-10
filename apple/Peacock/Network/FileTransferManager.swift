import Foundation
import Network
import os.log

private let xferLog = Logger(subsystem: "com.jlynnc.peacock", category: "Transfer")

/// Manages file transfer tasks: sending and receiving files via TCP.
@MainActor
final class FileTransferManager: ObservableObject {
    @Published var tasks: [String: TransferTask] = [:]

    private let maxConcurrent: Int
    private let sendQueue = DispatchQueue(label: "peacock.transfer.send", attributes: .concurrent)

    var downloadDir: URL {
        FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
            .appendingPathComponent("Peacock Downloads", isDirectory: true)
    }

    init(maxConcurrent: Int = 10) {
        self.maxConcurrent = maxConcurrent

        // Ensure download dir exists
        try? FileManager.default.createDirectory(at: downloadDir, withIntermediateDirectories: true)
    }

    func createSendTask(deviceId: String, fileName: String, filePath: String,
                        fileSize: UInt64, isFolder: Bool = false,
                        fileCount: UInt32 = 1) -> TransferTask {
        let task = TransferTask(
            transferId: UUID().uuidString,
            deviceId: deviceId,
            fileName: fileName,
            filePath: filePath,
            fileSize: fileSize,
            direction: .send,
            isFolder: isFolder,
            fileCount: fileCount
        )
        tasks[task.transferId] = task
        return task
    }

    func createReceiveTask(transferId: String, deviceId: String, fileName: String,
                           fileSize: UInt64, isFolder: Bool = false,
                           fileCount: UInt32 = 1) -> TransferTask {
        let task = TransferTask(
            transferId: transferId,
            deviceId: deviceId,
            fileName: fileName,
            filePath: "",
            fileSize: fileSize,
            direction: .receive,
            isFolder: isFolder,
            fileCount: fileCount
        )
        task.status = .pending
        tasks[transferId] = task
        return task
    }

    func getTask(_ id: String) -> TransferTask? { tasks[id] }

    var activeTasks: [TransferTask] {
        tasks.values.filter { [.pending, .active, .paused].contains($0.status) }
    }

    // MARK: - Receiving

    /// Start receiving: open a TCP listener on a random port, return the port.
    func startReceiving(transferId: String) async throws -> UInt16 {
        guard let task = tasks[transferId] else {
            xferLog.error("[Transfer] startReceiving: task not found \(transferId)")
            throw PeacockError.transferNotFound
        }
        xferLog.error("[Transfer] startReceiving: \(task.fileName) size=\(task.fileSize)")

        let filePath: URL
        if task.isFolder {
            filePath = uniquePath(for: downloadDir.appendingPathComponent(task.fileName, isDirectory: true))
        } else {
            filePath = uniquePath(for: downloadDir.appendingPathComponent(task.fileName))
        }
        task.filePath = filePath.path

        // Check for .part file (resume)
        let partPath = filePath.appendingPathExtension("part")
        if !task.isFolder, FileManager.default.fileExists(atPath: partPath.path) {
            let attrs = try FileManager.default.attributesOfItem(atPath: partPath.path)
            if let size = attrs[.size] as? UInt64 {
                task.resumeOffset = size
                task.transferredBytes = size
            }
        }

        // Create TCP listener using BSD socket (NWListener unreliable on simulator)
        let listenFd = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP)
        guard listenFd >= 0 else {
            throw PeacockError.networkError("Failed to create TCP socket: \(errno)")
        }

        var yes: Int32 = 1
        setsockopt(listenFd, SOL_SOCKET, SO_REUSEADDR, &yes, socklen_t(MemoryLayout<Int32>.size))

        var addr = sockaddr_in()
        addr.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
        addr.sin_family = sa_family_t(AF_INET)
        addr.sin_port = 0 // random port
        addr.sin_addr.s_addr = INADDR_ANY

        let bindResult = withUnsafePointer(to: &addr) { ptr in
            ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockPtr in
                bind(listenFd, sockPtr, socklen_t(MemoryLayout<sockaddr_in>.size))
            }
        }
        guard bindResult == 0 else {
            close(listenFd)
            throw PeacockError.networkError("TCP bind failed: \(errno)")
        }

        guard Darwin.listen(listenFd, 1) == 0 else {
            close(listenFd)
            throw PeacockError.networkError("TCP listen failed: \(errno)")
        }

        // Get assigned port
        var boundAddr = sockaddr_in()
        var boundLen = socklen_t(MemoryLayout<sockaddr_in>.size)
        withUnsafeMutablePointer(to: &boundAddr) { ptr in
            ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockPtr in
                getsockname(listenFd, sockPtr, &boundLen)
            }
        }
        let port = UInt16(bigEndian: boundAddr.sin_port)

        xferLog.error("[Transfer] TCP listener ready on port \(port)")

        let capturedTask = task
        let transferIdCopy = transferId

        // Accept connection in background
        Task.detached { [weak self] in
            guard let self else { close(listenFd); return }

            // Set accept timeout
            var timeout = timeval(tv_sec: 30, tv_usec: 0)
            setsockopt(listenFd, SOL_SOCKET, SO_RCVTIMEO, &timeout, socklen_t(MemoryLayout<timeval>.size))

            var clientAddr = sockaddr_in()
            var clientLen = socklen_t(MemoryLayout<sockaddr_in>.size)
            let clientFd = withUnsafeMutablePointer(to: &clientAddr) { ptr in
                ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockPtr in
                    accept(listenFd, sockPtr, &clientLen)
                }
            }
            close(listenFd)

            guard clientFd >= 0 else {
                xferLog.error("[Transfer] accept() failed: \(errno)")
                await MainActor.run { capturedTask.status = .failed }
                return
            }

            xferLog.error("[Transfer] Incoming TCP connection accepted (fd=\(clientFd))")

            await MainActor.run {
                self.tasks[transferIdCopy]?.status = .active
                self.tasks[transferIdCopy]?.receiverPort = port
            }

            // Receive file using raw fd
            await self.doReceiveWithFd(transferId: transferIdCopy, fd: clientFd, task: capturedTask)
        }

        return port
    }

    private func doReceiveWithFd(transferId: String, fd: Int32, task: TransferTask) async {
        let filePath = URL(fileURLWithPath: task.filePath)

        if task.isFolder {
            await doReceiveFolderWithFd(transferId: transferId, fd: fd, task: task, destDir: filePath)
        } else {
            await doReceiveSingleFileWithFd(transferId: transferId, fd: fd, task: task, filePath: filePath)
        }
    }

    private func doReceiveSingleFileWithFd(transferId: String, fd: Int32, task: TransferTask, filePath: URL) async {
        xferLog.error("[Transfer] doReceiveSingleFileWithFd: \(filePath.lastPathComponent)")
        let partPath = filePath.appendingPathExtension("part")

        if !FileManager.default.fileExists(atPath: partPath.path) {
            FileManager.default.createFile(atPath: partPath.path, contents: nil)
        }
        guard let handle = FileHandle(forWritingAtPath: partPath.path) else {
            await MainActor.run { task.status = .failed }
            close(fd)
            return
        }

        if task.resumeOffset > 0 {
            handle.seekToEndOfFile()
        }

        var totalReceived = task.resumeOffset
        let startTime = Date()
        var buffer = [UInt8](repeating: 0, count: NetworkConstants.fileChunkSize)

        while totalReceived < task.fileSize {
            let n = recv(fd, &buffer, buffer.count, 0)
            if n <= 0 { break }

            handle.write(Data(bytes: buffer, count: n))
            totalReceived += UInt64(n)

            let elapsed = Date().timeIntervalSince(startTime)
            let speed = elapsed > 0 ? UInt64(Double(totalReceived - task.resumeOffset) / elapsed) : 0
            await MainActor.run {
                task.transferredBytes = totalReceived
                task.speedBps = speed
            }
        }

        handle.closeFile()
        close(fd)

        try? FileManager.default.moveItem(at: partPath, to: filePath)

        await MainActor.run {
            task.transferredBytes = totalReceived
            task.status = totalReceived >= task.fileSize ? .completed : .failed
            task.filePath = filePath.path
        }
        xferLog.error("[Transfer] Receive done: \(totalReceived)/\(task.fileSize)")
    }

    private func doReceiveFolderWithFd(transferId: String, fd: Int32, task: TransferTask, destDir: URL) async {
        // Read manifest: [u64 LE manifest_len][manifest JSON]
        var lenBuf = [UInt8](repeating: 0, count: 8)
        let lenRead = recvAll(fd: fd, buffer: &lenBuf, count: 8)
        guard lenRead == 8 else {
            xferLog.error("[Transfer] Failed to read folder manifest length")
            await MainActor.run { task.status = .failed }
            close(fd)
            return
        }
        let manifestLen = lenBuf.withUnsafeBytes { $0.load(as: UInt64.self) }
        let manifestLength = UInt64(littleEndian: manifestLen)

        var manifestBuf = [UInt8](repeating: 0, count: Int(manifestLength))
        let mRead = recvAll(fd: fd, buffer: &manifestBuf, count: Int(manifestLength))
        guard mRead == Int(manifestLength) else {
            await MainActor.run { task.status = .failed }
            close(fd)
            return
        }

        guard let entries = try? JSONDecoder().decode([FolderEntry].self, from: Data(manifestBuf)) else {
            await MainActor.run { task.status = .failed }
            close(fd)
            return
        }

        try? FileManager.default.createDirectory(at: destDir, withIntermediateDirectories: true)

        var totalReceived: UInt64 = 0
        let startTime = Date()
        var buffer = [UInt8](repeating: 0, count: NetworkConstants.fileChunkSize)

        for entry in entries {
            let fileURL = destDir.appendingPathComponent(entry.relativePath)
            try? FileManager.default.createDirectory(at: fileURL.deletingLastPathComponent(), withIntermediateDirectories: true)
            FileManager.default.createFile(atPath: fileURL.path, contents: nil)
            guard let handle = FileHandle(forWritingAtPath: fileURL.path) else { continue }

            var fileReceived: UInt64 = 0
            while fileReceived < entry.size {
                let toRead = min(buffer.count, Int(entry.size - fileReceived))
                let n = recv(fd, &buffer, toRead, 0)
                if n <= 0 { break }
                handle.write(Data(bytes: buffer, count: n))
                fileReceived += UInt64(n)
                totalReceived += UInt64(n)

                let elapsed = Date().timeIntervalSince(startTime)
                let speed = elapsed > 0 ? UInt64(Double(totalReceived) / elapsed) : 0
                await MainActor.run {
                    task.transferredBytes = totalReceived
                    task.speedBps = speed
                }
            }
            handle.closeFile()
        }

        close(fd)
        await MainActor.run {
            task.transferredBytes = totalReceived
            task.status = .completed
        }
    }

    private func recvAll(fd: Int32, buffer: UnsafeMutablePointer<UInt8>, count: Int) -> Int {
        var received = 0
        while received < count {
            let n = recv(fd, buffer + received, count - received, 0)
            if n <= 0 { return received }
            received += n
        }
        return received
    }

    private func doReceive(transferId: String, connection: NWConnection, task: TransferTask) async {
        connection.start(queue: .global(qos: .userInitiated))

        let filePath = URL(fileURLWithPath: task.filePath)

        if task.isFolder {
            await doReceiveFolder(transferId: transferId, connection: connection, task: task, destDir: filePath)
        } else {
            await doReceiveSingleFile(transferId: transferId, connection: connection, task: task, filePath: filePath)
        }
    }

    private func doReceiveSingleFile(transferId: String, connection: NWConnection,
                                     task: TransferTask, filePath: URL) async {
        xferLog.error("[Transfer] doReceiveSingleFile: \(filePath.lastPathComponent) to \(filePath.path)")
        let partPath = filePath.appendingPathExtension("part")

        // Create or open .part file
        if !FileManager.default.fileExists(atPath: partPath.path) {
            FileManager.default.createFile(atPath: partPath.path, contents: nil)
        }
        guard let handle = FileHandle(forWritingAtPath: partPath.path) else {
            await MainActor.run { task.status = .failed }
            connection.cancel()
            return
        }

        if task.resumeOffset > 0 {
            handle.seekToEndOfFile()
        }

        var totalReceived = task.resumeOffset
        var lastProgressTime = Date()

        // Receive loop
        while true {
            let result: Data? = await withCheckedContinuation { cont in
                connection.receive(minimumIncompleteLength: 1, maximumLength: NetworkConstants.fileChunkSize) { data, _, isComplete, error in
                    if let data, !data.isEmpty {
                        cont.resume(returning: data)
                    } else if isComplete || error != nil {
                        cont.resume(returning: nil)
                    } else {
                        cont.resume(returning: nil)
                    }
                }
            }

            guard let chunk = result else { break }

            handle.write(chunk)
            totalReceived += UInt64(chunk.count)

            let now = Date()
            if now.timeIntervalSince(lastProgressTime) >= 0.1 {
                let speed = UInt64(Double(totalReceived - task.resumeOffset) /
                                   max(now.timeIntervalSince(lastProgressTime), 0.001))
                await MainActor.run {
                    task.transferredBytes = totalReceived
                    task.speedBps = speed
                }
                lastProgressTime = now
            }

            if totalReceived >= task.fileSize { break }
        }

        handle.closeFile()

        // Rename .part to final
        try? FileManager.default.moveItem(at: partPath, to: filePath)

        await MainActor.run {
            task.transferredBytes = totalReceived
            task.status = .completed
            task.filePath = filePath.path
        }
        connection.cancel()
    }

    private func doReceiveFolder(transferId: String, connection: NWConnection,
                                 task: TransferTask, destDir: URL) async {
        // First, read manifest: [u64 LE manifest_len][manifest JSON]
        guard let lenData = await receiveExact(connection: connection, count: 8) else {
            await MainActor.run { task.status = .failed }
            connection.cancel()
            return
        }
        let manifestLen = lenData.withUnsafeBytes { $0.load(as: UInt64.self) }
        let manifestLength = UInt64(littleEndian: manifestLen)

        guard let manifestData = await receiveExact(connection: connection, count: Int(manifestLength)) else {
            await MainActor.run { task.status = .failed }
            connection.cancel()
            return
        }

        guard let entries = try? JSONDecoder().decode([FolderEntry].self, from: manifestData) else {
            await MainActor.run { task.status = .failed }
            connection.cancel()
            return
        }

        task.folderManifest = entries

        // Create destination directory
        try? FileManager.default.createDirectory(at: destDir, withIntermediateDirectories: true)

        var totalReceived: UInt64 = 0
        let startTime = Date()

        for entry in entries {
            let fileURL = destDir.appendingPathComponent(entry.relativePath)
            let parentDir = fileURL.deletingLastPathComponent()
            try? FileManager.default.createDirectory(at: parentDir, withIntermediateDirectories: true)

            FileManager.default.createFile(atPath: fileURL.path, contents: nil)
            guard let handle = FileHandle(forWritingAtPath: fileURL.path) else { continue }

            var fileReceived: UInt64 = 0
            while fileReceived < entry.size {
                let toRead = min(UInt64(NetworkConstants.fileChunkSize), entry.size - fileReceived)
                guard let chunk = await receiveExact(connection: connection, count: Int(toRead)) else {
                    break
                }
                handle.write(chunk)
                fileReceived += UInt64(chunk.count)
                totalReceived += UInt64(chunk.count)

                let elapsed = Date().timeIntervalSince(startTime)
                let speed = elapsed > 0 ? UInt64(Double(totalReceived) / elapsed) : 0
                await MainActor.run {
                    task.transferredBytes = totalReceived
                    task.speedBps = speed
                }
            }
            handle.closeFile()
        }

        await MainActor.run {
            task.transferredBytes = totalReceived
            task.status = .completed
        }
        connection.cancel()
    }

    // MARK: - Sending

    func startSending(transferId: String, toHost host: String, port: UInt16,
                      resumeOffset: UInt64 = 0) {
        guard let task = tasks[transferId] else { return }
        task.resumeOffset = resumeOffset

        Task.detached { [weak self] in
            guard let self else { return }

            if task.isFolder {
                await self.doSendFolder(task: task, host: host, port: port)
            } else {
                await self.doSendFile(task: task, host: host, port: port, resumeOffset: resumeOffset)
            }
        }
    }

    private func doSendFile(task: TransferTask, host: String, port: UInt16, resumeOffset: UInt64) async {
        await MainActor.run { task.status = .active }

        let endpoint = NWEndpoint.hostPort(
            host: NWEndpoint.Host(host),
            port: NWEndpoint.Port(rawValue: port)!
        )
        let params = NWParameters.tcp
        #if !targetEnvironment(simulator)
        params.requiredInterfaceType = .wifi
        #endif
        let connection = NWConnection(to: endpoint, using: params)

        let connected = await withCheckedContinuation { (cont: CheckedContinuation<Bool, Never>) in
            nonisolated(unsafe) var resumed = false
            connection.stateUpdateHandler = { state in
                guard !resumed else { return }
                switch state {
                case .ready:
                    resumed = true
                    cont.resume(returning: true)
                case .failed, .cancelled:
                    resumed = true
                    cont.resume(returning: false)
                default:
                    break
                }
            }
            connection.start(queue: .global(qos: .userInitiated))
        }

        guard connected else {
            await MainActor.run { task.status = .failed }
            return
        }

        guard let handle = FileHandle(forReadingAtPath: task.filePath) else {
            await MainActor.run { task.status = .failed }
            connection.cancel()
            return
        }

        if resumeOffset > 0 {
            handle.seek(toFileOffset: resumeOffset)
        }

        var totalSent = resumeOffset
        let startTime = Date()

        while true {
            let status = await MainActor.run { task.status }
            if status == .paused {
                try? await Task.sleep(nanoseconds: 500_000_000)
                continue
            }
            if status == .failed { break }

            let chunk = handle.readData(ofLength: NetworkConstants.fileChunkSize)
            if chunk.isEmpty { break }

            let sendResult = await withCheckedContinuation { (cont: CheckedContinuation<Bool, Never>) in
                connection.send(content: chunk, completion: .contentProcessed { error in
                    cont.resume(returning: error == nil)
                })
            }

            guard sendResult else {
                await MainActor.run { task.status = .failed }
                break
            }

            totalSent += UInt64(chunk.count)
            let elapsed = Date().timeIntervalSince(startTime)
            let speed = elapsed > 0 ? UInt64(Double(totalSent - resumeOffset) / elapsed) : 0

            await MainActor.run {
                task.transferredBytes = totalSent
                task.speedBps = speed
            }
        }

        handle.closeFile()
        connection.cancel()

        if totalSent >= task.fileSize {
            await MainActor.run { task.status = .completed }
        }
    }

    private func doSendFolder(task: TransferTask, host: String, port: UInt16) async {
        await MainActor.run { task.status = .active }

        let endpoint = NWEndpoint.hostPort(
            host: NWEndpoint.Host(host),
            port: NWEndpoint.Port(rawValue: port)!
        )
        let params = NWParameters.tcp
        #if !targetEnvironment(simulator)
        params.requiredInterfaceType = .wifi
        #endif
        let connection = NWConnection(to: endpoint, using: params)

        let connected = await withCheckedContinuation { (cont: CheckedContinuation<Bool, Never>) in
            nonisolated(unsafe) var resumed = false
            connection.stateUpdateHandler = { state in
                guard !resumed else { return }
                switch state {
                case .ready:
                    resumed = true
                    cont.resume(returning: true)
                case .failed, .cancelled:
                    resumed = true
                    cont.resume(returning: false)
                default: break
                }
            }
            connection.start(queue: .global(qos: .userInitiated))
        }

        guard connected else {
            await MainActor.run { task.status = .failed }
            return
        }

        // Write manifest
        guard let manifestData = try? JSONEncoder().encode(task.folderManifest) else {
            await MainActor.run { task.status = .failed }
            connection.cancel()
            return
        }

        var lenBytes = Data(count: 8)
        lenBytes.withUnsafeMutableBytes { ptr in
            ptr.storeBytes(of: UInt64(manifestData.count).littleEndian, as: UInt64.self)
        }

        await sendData(lenBytes, on: connection)
        await sendData(manifestData, on: connection)

        var totalSent: UInt64 = 0
        let basePath = URL(fileURLWithPath: task.filePath)
        let startTime = Date()

        for entry in task.folderManifest {
            let fileURL = basePath.appendingPathComponent(entry.relativePath)
            guard let handle = FileHandle(forReadingAtPath: fileURL.path) else { continue }

            var fileSent: UInt64 = 0
            while fileSent < entry.size {
                let chunk = handle.readData(ofLength: NetworkConstants.fileChunkSize)
                if chunk.isEmpty { break }

                let ok = await sendData(chunk, on: connection)
                if !ok {
                    await MainActor.run { task.status = .failed }
                    handle.closeFile()
                    connection.cancel()
                    return
                }

                fileSent += UInt64(chunk.count)
                totalSent += UInt64(chunk.count)

                let elapsed = Date().timeIntervalSince(startTime)
                let speed = elapsed > 0 ? UInt64(Double(totalSent) / elapsed) : 0
                await MainActor.run {
                    task.transferredBytes = totalSent
                    task.speedBps = speed
                }
            }
            handle.closeFile()
        }

        await MainActor.run { task.status = .completed }
        connection.cancel()
    }

    // MARK: - Helpers

    @discardableResult
    private func sendData(_ data: Data, on connection: NWConnection) async -> Bool {
        await withCheckedContinuation { cont in
            connection.send(content: data, completion: .contentProcessed { error in
                cont.resume(returning: error == nil)
            })
        }
    }

    private func receiveExact(connection: NWConnection, count: Int) async -> Data? {
        var buffer = Data()
        while buffer.count < count {
            let remaining = count - buffer.count
            let chunk: Data? = await withCheckedContinuation { cont in
                connection.receive(minimumIncompleteLength: 1, maximumLength: remaining) { data, _, _, error in
                    if let data, !data.isEmpty {
                        cont.resume(returning: data)
                    } else {
                        cont.resume(returning: nil)
                    }
                }
            }
            guard let chunk else { return nil }
            buffer.append(chunk)
        }
        return buffer
    }

    private func uniquePath(for url: URL) -> URL {
        var result = url
        var counter = 1
        let fm = FileManager.default

        if url.hasDirectoryPath {
            while fm.fileExists(atPath: result.path) {
                result = url.deletingLastPathComponent()
                    .appendingPathComponent("\(url.lastPathComponent)(\(counter))", isDirectory: true)
                counter += 1
            }
        } else {
            let name = url.deletingPathExtension().lastPathComponent
            let ext = url.pathExtension
            while fm.fileExists(atPath: result.path) {
                if ext.isEmpty {
                    result = url.deletingLastPathComponent()
                        .appendingPathComponent("\(name)(\(counter))")
                } else {
                    result = url.deletingLastPathComponent()
                        .appendingPathComponent("\(name)(\(counter)).\(ext)")
                }
                counter += 1
            }
        }
        return result
    }
}

enum PeacockError: Error, LocalizedError {
    case transferNotFound
    case deviceNotFound
    case networkError(String)

    var errorDescription: String? {
        switch self {
        case .transferNotFound: return "Transfer not found"
        case .deviceNotFound: return "Device not found"
        case .networkError(let msg): return msg
        }
    }
}
