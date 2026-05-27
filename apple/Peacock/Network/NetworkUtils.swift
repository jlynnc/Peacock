import Foundation
import Network

enum NetworkConstants {
    static let udpPort: UInt16 = 52000
    static let tcpPort: UInt16 = 52001
    static let multicastAddress = "224.0.1.100"
    static let beaconInterval: TimeInterval = 10
    static let offlineTimeout: TimeInterval = 30
    static let timeoutCheckInterval: TimeInterval = 15
    static let fileChunkSize = 65536 // 64KB
    static let features: UInt32 = 0xFFFF
}

enum NetworkUtils {
    /// Get the local Wi-Fi IP address. Prefers en0 interface.
    static func getLocalIPAddress() -> String? {
        var address: String?
        var ifaddr: UnsafeMutablePointer<ifaddrs>?

        guard getifaddrs(&ifaddr) == 0, let firstAddr = ifaddr else { return nil }
        defer { freeifaddrs(ifaddr) }

        for ptr in sequence(first: firstAddr, next: { $0.pointee.ifa_next }) {
            let sa = ptr.pointee.ifa_addr.pointee
            guard sa.sa_family == UInt8(AF_INET) else { continue }

            let name = String(cString: ptr.pointee.ifa_name)
            let addr = ptr.pointee.ifa_addr.withMemoryRebound(to: sockaddr_in.self, capacity: 1) {
                String(cString: inet_ntoa($0.pointee.sin_addr))
            }

            // Skip loopback and link-local
            if addr.hasPrefix("127.") || addr.hasPrefix("169.254.") { continue }

            // Prefer en0 (Wi-Fi)
            if name == "en0" {
                return addr
            }
            if address == nil {
                address = addr
            }
        }
        return address
    }

    /// Compute the directed broadcast address from an IP.
    static func broadcastAddress(from ip: String) -> String? {
        let parts = ip.split(separator: ".").compactMap { UInt8($0) }
        guard parts.count == 4 else { return nil }
        return "\(parts[0]).\(parts[1]).\(parts[2]).255"
    }

    /// Convert UUID string to 16-byte array.
    static func uuidToBytes(_ uuidString: String) -> [UInt8] {
        guard let uuid = UUID(uuidString: uuidString) else {
            return Array(repeating: 0, count: 16)
        }
        let u = uuid.uuid
        return [u.0, u.1, u.2, u.3, u.4, u.5, u.6, u.7,
                u.8, u.9, u.10, u.11, u.12, u.13, u.14, u.15]
    }

    /// Convert 16-byte array to UUID string.
    static func bytesToUUID(_ bytes: [UInt8]) -> String {
        guard bytes.count == 16 else { return "" }
        let uuid = UUID(uuid: (bytes[0], bytes[1], bytes[2], bytes[3],
                                bytes[4], bytes[5], bytes[6], bytes[7],
                                bytes[8], bytes[9], bytes[10], bytes[11],
                                bytes[12], bytes[13], bytes[14], bytes[15]))
        return uuid.uuidString.lowercased()
    }

    /// Get the current platform string.
    static var currentPlatform: String {
        #if os(iOS)
        return "ios"
        #elseif os(macOS)
        return "macos"
        #else
        return "unknown"
        #endif
    }
}
