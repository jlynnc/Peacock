# Peacock v0.1.0

The first public release of Peacock — a cross-platform LAN file & message transfer tool.

## Features

- **Auto Device Discovery** — Devices on the same Wi-Fi are found automatically via UDP multicast, broadcast, and TCP probing
- **Instant Messaging** — Real-time text chat between devices with SQLite-backed history
- **File & Folder Transfer** — Drag & drop with 64KB chunked transfer, progress tracking, and resume support
- **Snippets** — Create, edit, search, and share text snippets across devices
- **Quick Copy** — Select text in a snippet and tap the floating mark button to save it instantly
- **Windows Context Menu** — Right-click any file → "Send to Peacock"
- **Bilingual UI** — Chinese / English, auto-detected from system
- **Dark Theme** — Follow system or manual toggle
- **Concurrent Transfer Limit** — Configurable via settings (default: 10)
- **Database Migrations** — Automatic schema upgrades on version updates

## Platforms

| Platform | Status |
|----------|--------|
| Windows | ✅ |
| iOS | ✅ |
| macOS | Coming soon |
| Android | Planned |
| Linux | Planned |

## Downloads

- **Windows**: `peacock.exe` (portable) or `Peacock_0.1.0_x64-setup.exe` (installer)
- **iOS**: Available on the App Store

## Tech Stack

Tauri v2 + Vue 3 + TypeScript + Rust + SQLite

## Known Issues

- Folder transfer delete button may not work after completion
- iOS snippet drag reorder is experimental
- Windows SmartScreen may show a warning for downloaded exe files sent via context menu

---

**Full Changelog**: https://github.com/jlynnc/Peacock/commits/v0.1.0
