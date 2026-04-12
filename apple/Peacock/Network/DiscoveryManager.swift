import Foundation

/// Manages device discovery state: tracks online devices, restricted peers, timeouts.
@MainActor
final class DiscoveryManager: ObservableObject {
    @Published var devices: [String: DeviceInfo] = [:]

    /// Devices that respond to our broadcasts but never broadcast themselves (iOS devices etc.)
    private var broadcastRestricted: Set<String> = []
    /// Devices we have seen broadcast
    private var hasBroadcast: Set<String> = []

    var onlineDevices: [DeviceInfo] {
        devices.values.filter(\.isOnline).sorted { $0.deviceName < $1.deviceName }
    }

    var onlineCount: Int {
        devices.values.filter(\.isOnline).count
    }

    /// Upsert a device. Returns true if the device is newly online.
    @discardableResult
    func upsertDevice(_ device: DeviceInfo) -> Bool {
        let wasOnline = devices[device.deviceId]?.isOnline ?? false
        var d = device
        d.isOnline = true
        d.lastSeen = Date()
        devices[device.deviceId] = d
        return !wasOnline
    }

    func markCanBroadcast(_ deviceId: String) {
        hasBroadcast.insert(deviceId)
        broadcastRestricted.remove(deviceId)
    }

    func markResponded(_ deviceId: String) {
        if !hasBroadcast.contains(deviceId) {
            broadcastRestricted.insert(deviceId)
        }
    }

    func getRestrictedPeers() -> [PeerInfo] {
        broadcastRestricted.compactMap { id in
            guard let d = devices[id], d.isOnline else { return nil }
            return PeerInfo(
                deviceId: d.deviceId,
                deviceName: d.deviceName,
                ipAddr: d.ipAddr,
                tcpPort: d.tcpPort,
                platform: d.platform
            )
        }
    }

    func mergeRestrictedPeers(_ peers: [PeerInfo], ownDeviceId: String) {
        for peer in peers {
            guard peer.deviceId != ownDeviceId else { continue }
            if devices[peer.deviceId] == nil || devices[peer.deviceId]?.isOnline == false {
                let device = DeviceInfo(
                    deviceId: peer.deviceId,
                    deviceName: peer.deviceName,
                    ipAddr: peer.ipAddr,
                    tcpPort: peer.tcpPort,
                    platform: peer.platform,
                    lastSeen: Date(),
                    isOnline: true
                )
                devices[peer.deviceId] = device
                broadcastRestricted.insert(peer.deviceId)
            }
        }
    }

    func markOffline(_ deviceId: String) {
        devices[deviceId]?.isOnline = false
        broadcastRestricted.remove(deviceId)
        hasBroadcast.remove(deviceId)
    }

    /// Check for timed-out devices. Returns IDs of newly-offline devices.
    func checkTimeouts() -> [String] {
        let now = Date()
        var offlineIds: [String] = []
        for (id, device) in devices where device.isOnline {
            if now.timeIntervalSince(device.lastSeen) > NetworkConstants.offlineTimeout {
                devices[id]?.isOnline = false
                broadcastRestricted.remove(id)
                hasBroadcast.remove(id)
                offlineIds.append(id)
            }
        }
        return offlineIds
    }

    func getDevice(_ deviceId: String) -> DeviceInfo? {
        devices[deviceId]
    }

    func isBroadcastRestricted(_ deviceId: String) -> Bool {
        broadcastRestricted.contains(deviceId)
    }
}
