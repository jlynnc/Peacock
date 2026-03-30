<p align="center">
  <img src="src-tauri/icons/icon_256.png" width="80" />
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
  <img src="https://img.shields.io/badge/Vue-3-green" />
  <img src="https://img.shields.io/badge/Rust-orange" />
  <img src="https://img.shields.io/badge/协议-MIT-lightgrey" />
</p>

---

## 截图

<!-- 替换为实际截图路径 -->

| 设备发现 | 即时聊天 | 文件传输 | 片段管理 |
|:---:|:---:|:---:|:---:|
| ![devices](docs/screenshots/devices.png) | ![chat](docs/screenshots/chat.png) | ![transfer](docs/screenshots/transfer.png) | ![snippets](docs/screenshots/snippets.png) |

## 亮点功能

### 快速复制 — 选中、标记、完成

Peacock 最具特色的功能。在片段编辑器中，选中任意文本后点击浮动的**标记按钮**，即可将选中内容保存为可复用的片段。无需复制粘贴，无需切换应用 — 选中即标记。

<!-- 替换为实际截图 -->
<p align="center">
  <img src="docs/screenshots/quick-copy.png" width="300" />
  <br/>
  <em>选中文本 → 点击浮动标记按钮 → 保存为片段</em>
</p>

非常适合收集 API Key、代码片段、会议笔记，或任何需要跨设备使用的文本。

## 功能特性

- **自动发现** — 同一局域网内的设备自动发现，零配置
- **即时消息** — 设备间实时文字聊天，消息持久化存储
- **文件传输** — 拖拽发送文件和文件夹，支持断点续传、进度显示
- **片段管理** — 创建、编辑、搜索文本片段，一键分享到其他设备
- **快速复制** — 在片段中选中文本，一键标记为可复用片段
- **双语界面** — 中文 / English，跟随系统自动切换
- **暗色主题** — 跟随系统 / 手动切换

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Vue 3 + TypeScript + TailwindCSS 4 |
| 后端 | Rust + Tauri v2 |
| 数据库 | SQLite (内置) |
| 协议 | 自定义二进制 (32 字节包头, bincode 序列化) |
| 图标 | Lucide |

## 网络架构

```
UDP 52000   ── 设备发现 (组播 224.0.1.100 + 广播)
TCP 52001   ── 消息 / 信令
动态 TCP    ── 文件传输 (64KB 分块)
```

发现策略：UDP 组播 → 子网广播 → TCP 探测 → 手动 IP

## 快速开始

### 前置条件

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) >= 1.75
- [Tauri CLI](https://tauri.app/) v2

### 开发

```bash
# 安装依赖
npm install

# 启动开发模式
npm run tauri dev
```

### 构建

```bash
# 构建发布版本
npx tauri build

# 产物 (Windows)
src-tauri/target/release/peacock.exe
```

## 平台支持

| 平台 | 状态 |
|------|------|
| Windows | ✅ 已完成 |
| macOS | 🚧 开发中 |
| iOS | 🚧 开发中 |
| Android | 📋 计划中 |
| Linux | 📋 计划中 |

## 项目结构

```
src/                    # Vue 3 前端
├── components/         #   UI 组件 (聊天、设备、片段、传输、移动端)
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

## 支持项目

如果 Peacock 对你有帮助，欢迎请我喝杯咖啡！

<p align="center">
  <a href="https://buymeacoffee.com/jlynnc">
    <img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black" />
  </a>
</p>

<!-- 如果设置了其他赞助平台，取消注释：
<p align="center">
  <a href="微信收款码图片路径"><img src="https://img.shields.io/badge/微信赞赏-07C160?style=for-the-badge&logo=wechat&logoColor=white" /></a>
  <a href="支付宝收款码图片路径"><img src="https://img.shields.io/badge/支付宝-00A1E9?style=for-the-badge&logo=alipay&logoColor=white" /></a>
</p>
-->

## 许可证

[MIT](LICENSE)
