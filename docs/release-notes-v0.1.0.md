# Peacock v0.1.0

The first public release of Peacock — a cross-platform LAN file & message transfer tool.

## Features

- **Auto Device Discovery** — Devices on the same Wi-Fi are found automatically via UDP broadcast + unicast response
- **Broadcast-Restricted Device Support** — Devices that cannot broadcast (e.g. iOS) are discovered via unicast response and shared through a restricted peers list
- **Instant Messaging** — Real-time text chat between devices via UDP unicast
- **File & Folder Transfer** — Drag & drop with 64KB chunked transfer, progress tracking, and resume support
- **Snippets** — Create, edit, search, and share text snippets across devices
- **Quick Copy** — Select text in a snippet and mark it for instant reuse
- **Windows Context Menu** — Right-click any file → "Send to Peacock"
- **Bilingual UI** — Chinese / English, auto-detected from system
- **Dark Theme** — Follow system or manual toggle
- **Self-Healing Network** — UDP sockets auto-rebuild on failure (handles iOS background suspend)

## Platforms

| Platform | Status |
|----------|--------|
| Windows | ✅ |
| Linux | ✅ |
| iOS | ✅ (Native Swift/SwiftUI) |
| macOS | Planned |
| Android | Planned |

## Downloads

- **Windows**: `peacock.exe` (portable) or `Peacock_0.1.0_x64-setup.exe` (installer)
- **Linux**: `Peacock_0.1.0_amd64.deb` (Debian/Ubuntu) or `Peacock-0.1.0-1.x86_64.rpm` (Fedora/RHEL)
- **iOS**: Pending App Store review

## Tech Stack

- **Windows/Linux**: Tauri v2 + Vue 3 + TypeScript + Rust + SQLite
- **iOS**: Swift + SwiftUI + Network.framework + SQLite

## Network Architecture

```
UDP 52000   — Device discovery + all messaging/signaling
Dynamic TCP — File data transfer only (64KB chunks)
```

---

**Full Changelog**: https://github.com/jlynnc/Peacock/commits/v0.1.0
