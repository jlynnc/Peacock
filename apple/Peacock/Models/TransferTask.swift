import Foundation

final class TransferTask: Identifiable, ObservableObject, @unchecked Sendable {
    let transferId: String
    let deviceId: String
    let fileName: String
    var filePath: String
    let fileSize: UInt64
    @Published var transferredBytes: UInt64 = 0
    @Published var status: TransferStatus = .pending
    let direction: TransferDirection
    @Published var speedBps: UInt64 = 0
    let isFolder: Bool
    let fileCount: UInt32
    let createdAt: UInt64
    var receiverPort: UInt16?
    var resumeOffset: UInt64 = 0
    var folderManifest: [FolderEntry] = []

    var id: String { transferId }

    var progress: Double {
        guard fileSize > 0 else { return 0 }
        return Double(transferredBytes) / Double(fileSize)
    }

    init(transferId: String, deviceId: String, fileName: String, filePath: String,
         fileSize: UInt64, direction: TransferDirection, isFolder: Bool = false,
         fileCount: UInt32 = 1) {
        self.transferId = transferId
        self.deviceId = deviceId
        self.fileName = fileName
        self.filePath = filePath
        self.fileSize = fileSize
        self.direction = direction
        self.isFolder = isFolder
        self.fileCount = fileCount
        self.createdAt = UInt64(Date().timeIntervalSince1970 * 1000)
    }
}

enum TransferStatus: String, Sendable {
    case pending
    case active
    case paused
    case completed
    case failed
    case rejected
}

enum TransferDirection: String, Sendable {
    case send
    case receive
}

struct FolderEntry: Codable, Sendable {
    let relativePath: String
    let size: UInt64

    enum CodingKeys: String, CodingKey {
        case relativePath = "relative_path"
        case size
    }
}
