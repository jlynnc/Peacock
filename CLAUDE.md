# Peacock (孔雀) — 跨平台局域网文件/消息传输工具

## 项目概述
类似 LocalSend 的 LAN 传输工具，UI 风格参考微信/飞秋，使用 Tauri v2 + Vue 3 + TypeScript + Rust 构建。

## 技术栈
- **前端**: Vue 3 (Composition API) + Pinia + TailwindCSS 4 + Lucide Icons + TypeScript
- **后端**: Rust + Tauri v2 (桌面 + 移动端)
- **数据库**: SQLite (rusqlite bundled)
- **协议**: 自定义二进制协议 (magic "PCOK", 32字节包头, bincode 序列化)
- **构建**: Vite 8 + tauri-cli

## 网络架构
- **UDP 52000**: 设备发现 (组播 224.0.1.100 + 定向广播 + 有限广播)
- **TCP 52001**: 消息/信令
- **动态 TCP 端口**: 文件传输
- **发现策略**: UDP 组播 → 定向子网广播 → TCP 主动探测 → 手动 IP

## 项目结构

### 前端 (src/)
```
src/
├── App.vue                    # 根组件，挂载所有 store 和监听器
├── main.ts                    # 入口
├── styles/global.css          # 全局样式 + CSS 变量
├── types/                     # TypeScript 类型定义
│   ├── device.ts              # DeviceInfo, SelfInfo
│   ├── message.ts             # ChatMessage, Conversation
│   ├── transfer.ts            # TransferTask, FileOffer
│   ├── snippet.ts             # Snippet 类型
│   └── clipboard.ts
├── stores/                    # Pinia 状态管理
│   ├── device.ts              # 设备列表，监听 device-online/offline 事件
│   ├── chat.ts                # 聊天消息，监听 new-message/message-sent 事件
│   ├── transfer.ts            # 传输任务，监听 file-offer/transfer-progress 事件
│   ├── snippet.ts             # 片段管理
│   └── settings.ts            # 应用设置
├── utils/
│   ├── ipc.ts                 # Tauri invoke 封装
│   ├── format.ts              # 文件大小/速度/时间格式化
│   └── platform.ts            # 平台检测
└── components/
    ├── layout/                # 布局组件
    │   ├── AppLayout.vue      # 主布局 + 无边框窗口控制
    │   ├── AppHeader.vue      # 顶部 Peacock logo
    │   └── AppSidebar.vue     # 左侧标签(设备/片段) + 列表
    ├── device/                # 设备相关
    │   ├── DeviceList.vue     # 在线设备列表
    │   ├── DeviceItem.vue     # 设备列表项
    │   └── DeviceAvatar.vue   # 平台图标头像
    ├── chat/                  # 聊天相关
    │   ├── ChatWindow.vue     # 聊天窗口(含统一标题栏)
    │   ├── ChatBubble.vue     # 消息气泡
    │   ├── ChatInput.vue      # 输入框(支持拖拽文件)
    │   └── ChatFileCard.vue   # 文件传输卡片(进度/打开目录/删除)
    ├── transfer/              # 传输相关
    │   ├── FileOfferDialog.vue # 文件接收弹窗(接收/另存为/拒绝)
    │   ├── TransferPanel.vue  # 传输列表面板
    │   └── TransferItem.vue   # 单个传输项
    ├── snippet/               # 片段功能
    │   ├── SnippetPanel.vue   # 片段主面板(列表+编辑器)
    │   ├── SnippetList.vue    # 片段列表(标题+时间)
    │   └── SnippetEditor.vue  # 片段编辑器(自动保存)
    └── settings/
        └── SettingsModal.vue  # 设置弹窗
```

### 后端 (src-tauri/src/)
```
src-tauri/src/
├── lib.rs                     # Tauri 入口，注册命令，启动后台任务
├── main.rs                    # Windows 入口
├── error.rs                   # PeacockError 错误类型
├── state.rs                   # AppState (device_id, 网络配置, 各模块状态)
├── protocol/                  # 二进制协议
│   ├── types.rs               # PacketType 枚举, 载荷结构体, 常量
│   ├── header.rs              # 32字节包头 序列化/反序列化
│   └── wire.rs                # encode/decode, build_packet, read/write_packet
├── discovery/                 # 设备发现
│   ├── device.rs              # DeviceInfo, DiscoveryState
│   ├── beacon.rs              # UDP 广播信标 (3秒间隔)
│   ├── listener.rs            # UDP 监听 + 超时检测
│   ├── probe.rs               # TCP /24 子网主动扫描
│   └── commands.rs            # get_online_devices, get_self_info
├── messaging/                 # 消息系统
│   ├── server.rs              # TCP 监听服务器 (端口 52001)
│   ├── client.rs              # send_to_device() TCP 客户端
│   ├── handler.rs             # 按类型分发: 文本/文件提议/接受/拒绝
│   └── commands.rs            # send_message, get_message_history
├── transfer/                  # 文件传输
│   ├── tracker.rs             # TransferTask, TransferManager
│   ├── sender.rs              # 64KB 分块发送, 断点续传, 进度上报
│   ├── receiver.rs            # 接收 + .part 临时文件 + 重命名
│   └── commands.rs            # send_file/folder, accept, pause, resume, cancel
├── clipboard/                 # 剪贴板 (已停用)
│   └── commands.rs
└── storage/
    └── db.rs                  # SQLite: messages, known_devices, transfers, settings, snippets
```

## 关键设计决策
1. **tauri::async_runtime::spawn** — 不用 tokio::spawn，否则 Tauri setup 中会 panic
2. **bincode 1.x** — 二进制序列化，不用 2.x
3. **serde::Serialize for PeacockError** — impl 中用 `std::result::Result` 避免和自定义 Result 冲突
4. **use tauri::Manager** — 在 lib.rs 中访问 path()/manage()
5. **use tauri::Emitter** — 在需要 emit() 的文件中引入
6. **无边框窗口** — decorations: false, 自定义标题栏 + 窗口控制按钮

## UI 设计
- **风格**: Bandcamp Style — 白底、teal (#0d9488) 强调色、现代简洁
- **布局**: 左侧边栏(标签+设备列表) + 右侧内容区(聊天/片段编辑)
- **无边框**: 统一标题栏 = 设备名 + 搜索框(居中) + 窗口控制按钮
- **聊天气泡**: 发送方 teal 背景，接收方白色背景
- **原型文件**: ui-mockup.html (Bandcamp Style, 选定方案)

## 已完成功能
- 四层设备发现 (UDP组播 + 定向广播 + 有限广播 + TCP探测)
- 即时消息 + SQLite 持久化
- 文件/文件夹传输 (拖拽、进度条、断点续传、打开目录、删除)
- 片段功能 (新建、编辑、搜索、分享到设备、自动保存600ms)
- 设置面板 (设备名、下载目录、自动接收)
- UI 重构 (Bandcamp Style + 无边框窗口 + 统一标题栏)

## 待办清单

### 待修复
- 文件夹传输完成后"删除"按钮不可用
- 文件传输卡片不应有气泡包裹

### Phase 1 — 基础加固
- 数据库迁移机制 (db_version + 自动迁移)
- 设备改名 (备注名)
- 开机启动 + 系统托盘
- 传输并发限制 (默认10连接)

### Phase 2 — 体验增强
- 消息/文件接收系统通知
- Windows 右键菜单 "发送到 Peacock"
- 传输历史记录
- 片段拖拽排序
- 多语言 (中/英)
- 暗色主题

### Phase 3 — 跨平台
- macOS 桌面版
- iOS (模拟器 + 真机)
- Android
- Linux
- Web 浏览器版

### Phase 4 — iOS 专项
- Bonjour/NWBrowser 发现优化

## 构建命令
```bash
# 开发
npm run tauri dev

# 构建 release
npx tauri build

# 产物位置 (Windows)
src-tauri/target/release/peacock.exe

# iOS (Mac 上)
npx tauri ios init
npx tauri ios dev

# Android
npx tauri android init
npx tauri android dev
```

## Git 信息
- 用户: Julian <omicronbai@gmail.com>
- GitHub: jlynnc
- 仓库: github.com/jlynnc/Peacock (私有)
