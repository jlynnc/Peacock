<p align="center">
  <img src="src-tauri/icons/icon.png" width="80" />
</p>

<h1 align="center">Peacock</h1>

<p align="center">
  跨平台局域网文件与消息传输工具<br/>
  <em>Cross-platform LAN file & message transfer</em>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.0-teal" />
  <img src="https://img.shields.io/badge/Tauri-v2-blue" />
  <img src="https://img.shields.io/badge/Vue-3-green" />
  <img src="https://img.shields.io/badge/Rust-orange" />
  <img src="https://img.shields.io/badge/license-MIT-lightgrey" />
</p>

---

## Screenshots / 截图

<!-- 替换为实际截图路径 / Replace with actual screenshot paths -->

| 设备发现 Devices | 即时聊天 Chat | 文件传输 Transfer | 片段 Snippets |
|:---:|:---:|:---:|:---:|
| ![devices](docs/screenshots/devices.png) | ![chat](docs/screenshots/chat.png) | ![transfer](docs/screenshots/transfer.png) | ![snippets](docs/screenshots/snippets.png) |

## Features / 功能

- **设备发现 / Device Discovery** — 自动发现同一局域网内的设备，无需手动配置
- **即时消息 / Instant Messaging** — 设备间实时文字聊天，消息持久化存储
- **文件传输 / File Transfer** — 拖拽发送文件和文件夹，支持断点续传、进度显示
- **片段管理 / Snippets** — 创建、编辑、搜索文本片段，一键分享到其他设备
- **双语界面 / Bilingual UI** — 中文 / English 自动切换
- **暗色主题 / Dark Theme** — 跟随系统 / 手动切换

## Tech Stack / 技术栈

| Layer | Technology |
|-------|-----------|
| Frontend | Vue 3 + TypeScript + TailwindCSS 4 |
| Backend | Rust + Tauri v2 |
| Database | SQLite (bundled) |
| Protocol | Custom binary (32-byte header, bincode) |
| Icons | Lucide |

## Network Architecture / 网络架构

```
UDP 52000   ── 设备发现 (组播 224.0.1.100 + 广播)
TCP 52001   ── 消息/信令
Dynamic TCP ── 文件传输 (64KB 分块)
```

发现策略：UDP 组播 → 子网广播 → TCP 探测 → 手动 IP

## Getting Started / 快速开始

### Prerequisites / 前置条件

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) >= 1.75
- [Tauri CLI](https://tauri.app/) v2

### Development / 开发

```bash
# 安装依赖 / Install dependencies
npm install

# 启动开发模式 / Start dev mode
npm run tauri dev
```

### Build / 构建

```bash
# 构建发布版本 / Build release
npx tauri build

# 产物 / Output (Windows)
src-tauri/target/release/peacock.exe
```

## Platforms / 平台支持

| Platform | Status |
|----------|--------|
| Windows | ✅ Ready |
| macOS | 🚧 In progress |
| iOS | 🚧 In progress |
| Android | 📋 Planned |
| Linux | 📋 Planned |

## Project Structure / 项目结构

```
src/                    # Vue 3 前端
├── components/         #   UI 组件 (chat, device, snippet, transfer, mobile)
├── stores/             #   Pinia 状态管理
├── types/              #   TypeScript 类型定义
├── i18n/               #   国际化 (zh-CN, en)
└── utils/              #   工具函数

src-tauri/src/          # Rust 后端
├── discovery/          #   设备发现 (UDP/TCP)
├── messaging/          #   消息系统
├── transfer/           #   文件传输
├── storage/            #   SQLite 数据库
└── protocol/           #   二进制协议
```

## License

MIT
