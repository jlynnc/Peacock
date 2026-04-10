# Peacock iOS/macOS Swift Rewrite Guide

This document provides everything needed to rewrite the Peacock iOS client in native Swift/SwiftUI, maintaining full protocol compatibility with the Windows/Linux Tauri version.

## Project Overview

Peacock is a LAN file & message transfer tool. The iOS version must interoperate with Windows devices running the Rust/Tauri version.

**Repo structure after rewrite:**
```
Peacock/
├── desktop/           # Existing Tauri (Win/Linux) — DON'T TOUCH
├── apple/             # NEW: Swift (iOS + macOS)
│   ├── Peacock/
│   │   ├── App/       # SwiftUI App entry
│   │   ├── Views/     # SwiftUI views
│   │   ├── Network/   # UDP discovery, TCP transfer
│   │   ├── Protocol/  # Packet header, payload encoding
│   │   ├── Storage/   # SQLite (snippets, settings)
│   │   └── Models/    # Data models
│   └── Peacock.xcodeproj
├── docs/
└── README.md
```

Move existing `src/`, `src-tauri/`, `package.json`, etc. into `desktop/`.

## Binary Protocol Specification

### Packet Header (32 bytes, big-endian)

```
Offset  Size  Field           Description
0       4     magic           "PCOK" [0x50, 0x43, 0x4F, 0x4B]
4       2     version         Protocol version (currently 1), big-endian u16
6       2     packet_type     Packet type enum, big-endian u16
8       16    device_id       UUID as raw 16 bytes
24      4     payload_length  Payload size in bytes, big-endian u32
28      4     reserved        Reserved, all zeros
```

**Swift implementation:**
```swift
struct PacketHeader {
    static let size = 32
    static let magic: [UInt8] = [0x50, 0x43, 0x4F, 0x4B]
    
    let version: UInt16
    let packetType: UInt16
    let deviceId: [UInt8]  // 16 bytes
    let payloadLength: UInt32
    
    func toData() -> Data {
        var data = Data(capacity: 32)
        data.append(contentsOf: Self.magic)
        data.append(contentsOf: withUnsafeBytes(of: version.bigEndian) { Array($0) })
        data.append(contentsOf: withUnsafeBytes(of: packetType.bigEndian) { Array($0) })
        data.append(contentsOf: deviceId)
        data.append(contentsOf: withUnsafeBytes(of: payloadLength.bigEndian) { Array($0) })
        data.append(contentsOf: [0, 0, 0, 0])  // reserved
        return data
    }
    
    static func from(data: Data) -> PacketHeader? {
        guard data.count >= 32 else { return nil }
        guard Array(data[0..<4]) == magic else { return nil }
        let version = UInt16(bigEndian: data[4..<6].withUnsafeBytes { $0.load(as: UInt16.self) })
        let packetType = UInt16(bigEndian: data[6..<8].withUnsafeBytes { $0.load(as: UInt16.self) })
        let deviceId = Array(data[8..<24])
        let payloadLength = UInt32(bigEndian: data[24..<28].withUnsafeBytes { $0.load(as: UInt32.self) })
        return PacketHeader(version: version, packetType: packetType, deviceId: deviceId, payloadLength: payloadLength)
    }
}
```

### Packet Types

```
Value  Name              Transport  Description
1      Announce          UDP broadcast  Device presence announcement
2      Bye               UDP broadcast  Device going offline
3      AnnounceResponse  UDP unicast    Response to Announce
10     Text              UDP unicast    Text message
20     FileOffer         UDP unicast    File transfer offer
21     FileAccept        UDP unicast    File transfer accepted
22     FileReject        UDP unicast    File transfer rejected
23     FileChunk         TCP            File data (not used as packet, raw TCP)
30     Clipboard         UDP unicast    Clipboard content (unused)
31     SnippetShare      UDP unicast    Snippet share
99     Ack               UDP unicast    Acknowledgment (unused)
```

### Payload Encoding: bincode 1.x

**CRITICAL:** Payloads are encoded with Rust's `bincode` crate v1.x. This uses a specific binary format:
- Strings: 8-byte little-endian length prefix (u64) + UTF-8 bytes
- u16/u32/u64: little-endian
- bool: 1 byte (0 or 1)
- Vec<T>: 8-byte little-endian length prefix (u64) + concatenated elements
- Structs: fields serialized in declaration order, no field names

**Swift bincode encoder/decoder must match this format exactly.**

Example for `TextPayload { message_id: String, text: String, timestamp: u64 }`:
```
[8 bytes: message_id length (LE u64)]
[N bytes: message_id UTF-8]
[8 bytes: text length (LE u64)]
[N bytes: text UTF-8]
[8 bytes: timestamp (LE u64)]
```

### Payload Structures

**AnnouncePayload:**
```
Fields (in order):
  device_name: String
  platform: String       // "ios", "macos", "windows", "linux", "android"
  tcp_port: u16
  features: u32          // 0xFFFF for all features
  restricted_peers: Vec<PeerInfo>  // may be empty
```

**PeerInfo:**
```
Fields (in order):
  device_id: String
  device_name: String
  ip_addr: String
  tcp_port: u16
  platform: String
```

**TextPayload:**
```
Fields (in order):
  message_id: String     // UUID v4
  text: String
  timestamp: u64         // milliseconds since Unix epoch
```

**FileOfferPayload:**
```
Fields (in order):
  transfer_id: String    // UUID v4
  file_name: String
  file_size: u64
  is_folder: bool
  file_count: u32
```

**FileAcceptPayload:**
```
Fields (in order):
  transfer_id: String
  receiver_port: u16     // TCP port the receiver is listening on
  resume_offset: u64     // byte offset for resume (0 for fresh transfer)
```

**FileRejectPayload:**
```
Fields (in order):
  transfer_id: String
```

**SnippetSharePayload:**
```
Fields (in order):
  title: String
  content: String
  tag: String
  note: String
```

## Network Architecture

### Ports
- **UDP 52000** — All discovery + messaging + signaling
- **TCP dynamic** — File data transfer only (receiver picks a random port)

### Device Discovery Mechanism

**Broadcast (every 10 seconds):**
1. Device sends UDP Announce to multicast 224.0.1.100:52000 AND subnet broadcast AND 255.255.255.255:52000
2. Payload includes `restricted_peers` list

**Unicast Response:**
1. When device B receives device A's broadcast, B sends UDP AnnounceResponse directly to A's IP:52000
2. This confirms B is alive to A

**Restricted Device Detection:**
1. If A receives B's AnnounceResponse but never receives B's broadcast → B is "broadcast-restricted"
2. A adds B to its `restricted_peers` list
3. A's future broadcasts include B in `restricted_peers`
4. Other devices (C) receiving A's broadcast learn about B

**Offline Detection:**
- 30 seconds without hearing from a device → mark offline
- Check every 15 seconds

**iOS Specifics:**
- iOS cannot send UDP broadcast/multicast (requires Apple entitlement)
- iOS CAN send UDP unicast (verified working)
- iOS CAN receive UDP broadcast from others
- Use NWConnection/NWListener for all networking — avoids BSD socket restrictions
- NWConnection handles interface selection automatically (no IP_BOUND_IF needed)

### Messaging (UDP Unicast)

All messages are sent as UDP packets to the target device's IP on port 52000. The packet format is: 32-byte header + bincode-encoded payload.

### File Transfer (TCP)

Flow:
1. Sender sends FileOffer via UDP
2. Receiver decides accept/reject
3. If accepted: receiver opens TCP listener on random port, sends FileAccept (with port number) via UDP
4. Sender TCP-connects to receiver's port and streams the file in 64KB chunks
5. For folders: sender first writes a bincode-encoded manifest (Vec of relative paths + sizes), then streams all files concatenated

Resume:
- Receiver checks for existing `.part` file
- If found, sends FileAccept with `resume_offset` = existing bytes
- Sender seeks to that offset before streaming

## UI Screens (SwiftUI)

### Tab Bar (3 tabs)
1. **Devices** — List of online devices with name, platform icon, IP, last message preview
2. **Snippets** — List of text snippets with left-swipe actions
3. **Settings** — Device name, download folder, theme, language, about

### Chat View (push from device tap)
- Message bubbles (sent right/teal, received left/gray)
- Input bar with text field + send button
- Plus button → panel: Photos, Camera, Snippets, Files
- Right-swipe to go back (native NavigationStack)

### Snippet Editor (push from snippet tap)
- Title at top
- Content area (editable)
- Quick copy: `[[text]]` rendered as tappable chips, tap to copy
- Toolbar: Copy content, Share, Delete
- Note field at bottom
- Auto-save after 600ms idle

### Snippet List
- Left-swipe: Rename, Share, Pin to top, Delete
- New snippet: stays in list, focuses rename input
- Search bar at top

## SQLite Schema

```sql
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS snippets (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL DEFAULT '新建片段',
    content TEXT NOT NULL DEFAULT '',
    tag TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    direction TEXT NOT NULL,
    content TEXT NOT NULL,
    msg_type TEXT NOT NULL DEFAULT 'text',
    timestamp INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'sent'
);
```

Settings keys: `device_id`, `device_name`, `download_dir`, `theme`, `locale`, `max_concurrent`, `db_version`

## Key Design Decisions

1. **Use NWConnection/NWListener** for all networking — not BSD sockets. This avoids all iOS restrictions.
2. **bincode 1.x compatibility** is critical — test by sending messages between Swift and Windows.
3. **Device ID** is a UUID v4 stored in SQLite settings. The raw 16 bytes go in every packet header.
4. **platform** string for iOS should be `"ios"`, for macOS `"macos"`.
5. **Auto-save snippets** with 600ms debounce.
6. **Quick copy chips**: content format is `plain text with [[marked text]] inline`. Parse `[[...]]` and render as tappable chips.

## Testing Checklist

- [ ] Windows discovers iOS (via UDP unicast response)
- [ ] iOS discovers Windows (via UDP broadcast reception)
- [ ] Text messages: iOS → Windows, Windows → iOS
- [ ] File transfer: Windows → iOS, iOS → Windows
- [ ] Snippet share: both directions
- [ ] Restricted peers list propagation
- [ ] Background/foreground resume
- [ ] Camera photo send
- [ ] Photo library send
- [ ] File picker send

## Dependencies (Swift Package Manager)

- **SQLite.swift** or **GRDB** — SQLite access
- No other external dependencies needed — use Foundation/Network.framework
