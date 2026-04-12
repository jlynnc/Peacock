import Foundation

/// Manages device discovery with converged two-rule design:
/// Rule 1: I broadcast → someone responds (AnnounceResponse) → I add them
/// Rule 2: I receive broadcast → my ID is in their restricted_peers → I add them
///
/// Restricted (yellow dot) status is derived from last_broadcast_at:
///   - 0 or older than BEACON_INTERVAL * 3 → restricted
///   - otherwise → normal (green dot)
@MainActor
final class DiscoveryManager: ObservableObject {
    @Published var devices: [String: DeviceInfo] = [:]

    /// Last time we received an Announce broadcast from each device.
    /// Key = deviceId, Value = timestamp (0 = never received)
    private var lastBroadcastAt: [String: Date] = [:]

    /// Threshold: if no broadcast received within this window, device is restricted.
    private let restrictedThreshold: TimeInterval = NetworkConstants.beaconInterval * 3 // 30s

    var onlineDevices: [DeviceInfo] {
        devices.values.filter(\.isOnline).sorted { $0.deviceName < $1.deviceName }
    }

    var onlineCount: Int {
        devices.values.filter(\.isOnline).count
    }

    // MARK: - Rule 1: Add device from AnnounceResponse

    /// Someone responded to our broadcast. Add them to device list.
    /// Returns true if newly online.
    @discardableResult
    func addDeviceFromResponse(_ device: DeviceInfo) -> Bool {
        let wasOnline = devices[device.deviceId]?.isOnline ?? false
        var d = device
        d.isOnline = true
        d.lastSeen = Date()
        devices[device.deviceId] = d
        return !wasOnline
    }

    // MARK: - Rule 2: Check if self is in broadcaster's restricted_peers

    /// Called when we receive a broadcast. Check if our device ID is in
    /// the broadcaster's restricted_peers list. If yes, add the broadcaster.
    /// Returns true if we added the broadcaster.
    @discardableResult
    func checkSelfInRestrictedPeers(broadcaster: DeviceInfo, restrictedPeers: [PeerInfo], ownDeviceId: String) -> Bool {
        let isMeRestricted = restrictedPeers.contains { $0.deviceId == ownDeviceId }
        if isMeRestricted {
            var d = broadcaster
            d.isOnline = true
            d.lastSeen = Date()
            devices[broadcaster.deviceId] = d
            return true
        }
        return false
    }

    // MARK: - Broadcast tracking

    /// Record that we received a broadcast from this device.
    func noteReceivedBroadcast(from deviceId: String) {
        lastBroadcastAt[deviceId] = Date()
    }

    /// Update lastSeen for a device already in our list.
    func touchDevice(_ deviceId: String) {
        if devices[deviceId] != nil {
            devices[deviceId]?.lastSeen = Date()
        }
    }

    // MARK: - Restricted status (derived from lastBroadcastAt)

    /// A device is restricted if we've never received their broadcast,
    /// or haven't received one within BEACON_INTERVAL * 3 (30s).
    func isBroadcastRestricted(_ deviceId: String) -> Bool {
        guard let lastBroadcast = lastBroadcastAt[deviceId] else {
            return true // never received
        }
        return Date().timeIntervalSince(lastBroadcast) > restrictedThreshold
    }

    /// Get our restricted peers list to include in our broadcasts.
    /// Called before each beacon send to get the current list.
    func getRestrictedPeers() -> [PeerInfo] {
        devices.values
            .filter { $0.isOnline && isBroadcastRestricted($0.deviceId) }
            .map { PeerInfo(deviceId: $0.deviceId, deviceName: $0.deviceName,
                            ipAddr: $0.ipAddr, tcpPort: $0.tcpPort, platform: $0.platform) }
    }

    // MARK: - Offline

    func markOffline(_ deviceId: String) {
        devices[deviceId]?.isOnline = false
        lastBroadcastAt.removeValue(forKey: deviceId)
    }

    /// Check for timed-out devices. Returns IDs of newly-offline devices.
    func checkTimeouts() -> [String] {
        let now = Date()
        var offlineIds: [String] = []
        for (id, device) in devices where device.isOnline {
            if now.timeIntervalSince(device.lastSeen) > NetworkConstants.offlineTimeout {
                devices[id]?.isOnline = false
                lastBroadcastAt.removeValue(forKey: id)
                offlineIds.append(id)
            }
        }
        return offlineIds
    }

    // MARK: - Lookup

    func getDevice(_ deviceId: String) -> DeviceInfo? {
        devices[deviceId]
    }
}
