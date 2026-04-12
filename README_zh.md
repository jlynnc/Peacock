<p align="center">
  <img src="desktop/src-tauri/icons/icon_256.png" width="80" />
</p>

<h1 align="center">Peacock (孔雀)</h1>

<p align="center">
  跨平台局域网文件与消息传输工具<br/>
  无需服务器、无需联网、无需注册 — 连上即发。
</p>

<p align="center">
  <strong>简体中文</strong> | <a href="README.md">English</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/版本-0.1.0-teal" />
  <img src="https://img.shields.io/badge/Tauri-v2-blue" />
  <img src="https://img.shields.io/badge/Swift-iOS-orange" />
  <img src="https://img.shields.io/badge/Kotlin-Android-green" />
  <img src="https://img.shields.io/badge/协议-MIT-lightgrey" />
</p>

---

## 截图

| 片段管理 | 即时聊天 |
|:---:|:---:|
| ![snippets](docs/screenshots/snippets.png) | ![chat](docs/screenshots/chat.png) |

## 功能特性

- **自动发现** — 同一局域网内的设备通过 UDP 广播 + 单播回应自动发现，零配置
- **即时消息** — 设备间通过 UDP 实时文字聊天
- **文件传输** — 发送文件和文件夹，支持断点续传、进度显示
- **片段管理** — 创建、编辑、搜索文本片段，支持内联快速复制芯片
- **广播受限设备支持** — 无法广播的设备通过受限列表自动被其他设备发现
- **暗色主题** — 跟随系统 / 手动切换

## 技术栈

| 组件 | 桌面端 | iOS | Android |
|------|--------|-----|---------|
| UI | Vue 3 + TailwindCSS | SwiftUI | Jetpack Compose |
| 后端 | Rust + Tauri v2 | Swift | Kotlin |
| 数据库 | SQLite (rusqlite) | SQLite | SQLite |
| 协议 | 自定义二进制 (PCOK 包头 + bincode) | 相同 | 相同 |

## 网络架构

```
UDP 52000   ── 设备发现 + 消息 + 信令
动态 TCP    ── 文件数据传输 (256KB 分块)
```

发现规则：
1. 我广播 → 对方回应 → 我加对方到设备列表
2. 我收到广播 → 发现自己在对方 restricted_peers 里 → 我加对方
3. 收到广播不直接添加设备（只回 AnnounceResponse）

## 平台支持

| 平台 | 状态 | 技术 |
|------|------|------|
| Windows | ✅ 已发布 | Tauri v2 (Rust + Vue 3) |
| Linux | ✅ 已发布 | Tauri v2 (Rust + Vue 3) |
| iOS | ✅ App Store | 原生 Swift / SwiftUI |
| Android | ✅ 已发布 (APK) | 原生 Kotlin / Compose |
| macOS | 📋 计划中 | Swift (与 iOS 共享代码) |

## 下载

**[GitHub Releases](https://github.com/jlynnc/Peacock/releases/tag/v0.1.0)**

- `peacock.exe` — Windows (免安装)
- `Peacock_0.1.0_x64-setup.exe` — Windows (安装包)
- `Peacock_0.1.0_amd64.deb` — Linux (Debian/Ubuntu)
- `Peacock-0.1.0-1.x86_64.rpm` — Linux (Fedora/RHEL)
- `Peacock_0.1.0.apk` — Android
- iOS — 在 App Store 搜索 "Peacock"

## 项目结构

```
desktop/                # Tauri 桌面应用 (Windows + Linux)
├── src/                #   Vue 3 前端
└── src-tauri/src/      #   Rust 后端

apple/                  # iOS 应用 (原生 Swift)
├── Peacock/            #   SwiftUI 视图 + 网络层

android/                # Android 应用 (原生 Kotlin)
├── app/src/main/java/  #   Compose UI + 协议 + 网络层
```

## 从源码构建

### 桌面端 (Windows / Linux)

```bash
cd desktop
npm install
npx tauri dev          # 开发
npx tauri build        # 发布
```

### Android

```bash
cd android
./gradlew assembleRelease
```

### iOS

```bash
cd apple
# 在 Xcode 中打开，构建运行
```

## 支持项目

<p align="center">
  <a href="https://buymeacoffee.com/jlynnc">
    <img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black" />
  </a>
</p>

<details>
<summary>支付宝</summary>
<p align="center">
  <img src="docs/alipay-qr.jpg" width="200" />
</p>
</details>

## 许可证

[MIT](LICENSE)
