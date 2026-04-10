import Foundation
import Network
import os.log

private let udpLog = Logger(subsystem: "com.jlynnc.peacock", category: "UDP")

/// Manages UDP communication: sending and receiving discovery + messaging packets.
/// Uses BSD sockets for listening (reliable broadcast/multicast reception)
/// and NWConnection for sending (handles iOS interface routing).
final class UDPService: @unchecked Sendable {
    private var listenFd: Int32 = -1
    private var listenThread: Thread?
    private var isRunning = false

    let deviceIdBytes: [UInt8]
    let deviceIdString: String
    private let onPacketReceived: @Sendable (PacketHeader, Data, String) -> Void

    init(deviceIdBytes: [UInt8], deviceIdString: String,
         onPacketReceived: @escaping @Sendable (PacketHeader, Data, String) -> Void) {
        self.deviceIdBytes = deviceIdBytes
        self.deviceIdString = deviceIdString
        self.onPacketReceived = onPacketReceived
    }

    // MARK: - Listening (BSD Socket)

    func startListening() {
        guard !isRunning else { return }

        listenFd = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP)
        guard listenFd >= 0 else {
            udpLog.error("[UDP] Failed to create socket: \(errno)")
            return
        }

        // SO_REUSEADDR + SO_REUSEPORT
        var yes: Int32 = 1
        setsockopt(listenFd, SOL_SOCKET, SO_REUSEADDR, &yes, socklen_t(MemoryLayout<Int32>.size))
        setsockopt(listenFd, SOL_SOCKET, SO_REUSEPORT, &yes, socklen_t(MemoryLayout<Int32>.size))

        // Bind to 0.0.0.0:52000
        var addr = sockaddr_in()
        addr.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
        addr.sin_family = sa_family_t(AF_INET)
        addr.sin_port = NetworkConstants.udpPort.bigEndian
        addr.sin_addr.s_addr = INADDR_ANY

        let bindResult = withUnsafePointer(to: &addr) { ptr in
            ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockPtr in
                bind(listenFd, sockPtr, socklen_t(MemoryLayout<sockaddr_in>.size))
            }
        }
        guard bindResult == 0 else {
            udpLog.error("[UDP] Bind failed: \(errno)")
            close(listenFd)
            listenFd = -1
            return
        }

        // Join multicast group 224.0.1.100
        var mreq = ip_mreq()
        mreq.imr_multiaddr.s_addr = inet_addr(NetworkConstants.multicastAddress)
        mreq.imr_interface.s_addr = INADDR_ANY
        setsockopt(listenFd, IPPROTO_IP, IP_ADD_MEMBERSHIP, &mreq, socklen_t(MemoryLayout<ip_mreq>.size))

        // SO_BROADCAST
        setsockopt(listenFd, SOL_SOCKET, SO_BROADCAST, &yes, socklen_t(MemoryLayout<Int32>.size))

        isRunning = true
        udpLog.error("[UDP] Listener ready on port \(NetworkConstants.udpPort)")

        // Start receive loop on background thread
        let fd = listenFd
        let ownId = deviceIdString
        let idBytes = deviceIdBytes
        let callback = onPacketReceived

        listenThread = Thread {
            var buffer = [UInt8](repeating: 0, count: 4096)
            var srcAddr = sockaddr_in()
            var srcAddrLen = socklen_t(MemoryLayout<sockaddr_in>.size)

            while true {
                let n = withUnsafeMutablePointer(to: &srcAddr) { srcPtr in
                    srcPtr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockPtr in
                        recvfrom(fd, &buffer, buffer.count, 0, sockPtr, &srcAddrLen)
                    }
                }

                guard n > 0 else {
                    if errno == EBADF { break } // Socket closed
                    continue
                }

                let data = Data(bytes: buffer, count: n)

                // Extract source IP early for logging
                var ipBuf = [CChar](repeating: 0, count: Int(INET_ADDRSTRLEN))
                var addrCopy = srcAddr.sin_addr
                inet_ntop(AF_INET, &addrCopy, &ipBuf, socklen_t(INET_ADDRSTRLEN))
                let sourceIP = String(cString: ipBuf)

                guard data.count >= PacketHeader.size else {
                    udpLog.error("[UDP] Packet too small (\(n) bytes) from \(sourceIP)")
                    continue
                }
                guard let header = PacketHeader.from(data: data) else {
                    udpLog.error("[UDP] Invalid header from \(sourceIP)")
                    continue
                }

                // Ignore own packets
                let senderId = NetworkUtils.bytesToUUID(header.deviceId)
                guard senderId != ownId else { continue }

                udpLog.error("[UDP] Received type=\(header.packetType.rawValue) from \(sourceIP) sender=\(senderId.prefix(8))")

                // Extract payload
                let payloadStart = PacketHeader.size
                let payloadEnd = payloadStart + Int(header.payloadLength)
                let payload: Data
                if data.count >= payloadEnd {
                    payload = Data(data[payloadStart..<payloadEnd])
                } else {
                    payload = Data()
                }

                callback(header, payload, sourceIP)
            }
        }
        listenThread?.qualityOfService = .userInitiated
        listenThread?.name = "peacock.udp.listener"
        listenThread?.start()
    }

    func stopListening() {
        isRunning = false
        if listenFd >= 0 {
            close(listenFd)
            listenFd = -1
        }
        listenThread = nil
    }

    // MARK: - Sending (NWConnection)

    func sendPacket(_ data: Data, to host: String, port: UInt16 = NetworkConstants.udpPort) {
        let endpoint = NWEndpoint.hostPort(
            host: NWEndpoint.Host(host),
            port: NWEndpoint.Port(rawValue: port)!
        )
        let params = NWParameters.udp
        #if !targetEnvironment(simulator)
        params.requiredInterfaceType = .wifi
        #endif
        let connection = NWConnection(to: endpoint, using: params)
        connection.start(queue: .global(qos: .userInitiated))
        connection.send(content: data, completion: .contentProcessed { error in
            if let error {
                udpLog.error("[UDP] Send error to \(host): \(error.localizedDescription)")
            }
            connection.cancel()
        })
    }

    /// Also send via BSD socket for broadcast (NWConnection can't broadcast easily)
    func sendBroadcastPacket(_ data: Data, to host: String, port: UInt16 = NetworkConstants.udpPort) {
        let fd = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP)
        guard fd >= 0 else { return }
        defer { close(fd) }

        var yes: Int32 = 1
        setsockopt(fd, SOL_SOCKET, SO_BROADCAST, &yes, socklen_t(MemoryLayout<Int32>.size))

        var destAddr = sockaddr_in()
        destAddr.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
        destAddr.sin_family = sa_family_t(AF_INET)
        destAddr.sin_port = port.bigEndian
        destAddr.sin_addr.s_addr = inet_addr(host)

        data.withUnsafeBytes { buf in
            withUnsafePointer(to: &destAddr) { addrPtr in
                addrPtr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockPtr in
                    sendto(fd, buf.baseAddress, data.count, 0, sockPtr,
                           socklen_t(MemoryLayout<sockaddr_in>.size))
                }
            }
        }
    }

    func sendAnnounce(payload: AnnouncePayload) {
        let payloadData = payload.encode()
        let packet = PacketBuilder.build(type: .announce, deviceId: deviceIdBytes, payload: payloadData)

        // Send via BSD socket for broadcast reliability
        sendBroadcastPacket(packet, to: NetworkConstants.multicastAddress)

        if let ip = NetworkUtils.getLocalIPAddress(),
           let broadcast = NetworkUtils.broadcastAddress(from: ip) {
            sendBroadcastPacket(packet, to: broadcast)
        }

        sendBroadcastPacket(packet, to: "255.255.255.255")
    }

    func sendAnnounceResponse(payload: AnnouncePayload, to host: String) {
        let payloadData = payload.encode()
        let packet = PacketBuilder.build(type: .announceResponse, deviceId: deviceIdBytes, payload: payloadData)
        sendPacket(packet, to: host)
    }

    func sendBye() {
        let packet = PacketBuilder.buildHeaderOnly(type: .bye, deviceId: deviceIdBytes)
        if let ip = NetworkUtils.getLocalIPAddress(),
           let broadcast = NetworkUtils.broadcastAddress(from: ip) {
            sendBroadcastPacket(packet, to: broadcast)
        }
        sendBroadcastPacket(packet, to: "255.255.255.255")
    }

    func sendText(_ payload: TextPayload, to host: String) {
        let payloadData = payload.encode()
        let packet = PacketBuilder.build(type: .text, deviceId: deviceIdBytes, payload: payloadData)
        sendPacket(packet, to: host)
    }

    func sendFileOffer(_ payload: FileOfferPayload, to host: String) {
        let payloadData = payload.encode()
        let packet = PacketBuilder.build(type: .fileOffer, deviceId: deviceIdBytes, payload: payloadData)
        sendPacket(packet, to: host)
    }

    func sendFileAccept(_ payload: FileAcceptPayload, to host: String) {
        let payloadData = payload.encode()
        let packet = PacketBuilder.build(type: .fileAccept, deviceId: deviceIdBytes, payload: payloadData)
        sendPacket(packet, to: host)
    }

    func sendFileReject(_ payload: FileRejectPayload, to host: String) {
        let payloadData = payload.encode()
        let packet = PacketBuilder.build(type: .fileReject, deviceId: deviceIdBytes, payload: payloadData)
        sendPacket(packet, to: host)
    }

    func sendSnippetShare(_ payload: SnippetSharePayload, to host: String) {
        let payloadData = payload.encode()
        let packet = PacketBuilder.build(type: .snippetShare, deviceId: deviceIdBytes, payload: payloadData)
        sendPacket(packet, to: host)
    }
}
