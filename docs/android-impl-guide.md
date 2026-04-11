# Peacock Android 实现要点

> 基于 iOS Swift 版本的实战经验，给 Android 实现的踩坑指南和关键要点。

## 项目结构建议

```
Peacock/
├── desktop/           # Tauri (Win/Linux) — 不要碰
├── apple/             # Swift (iOS) — 不要碰
├── android/           # Kotlin/Jetpack Compose
│   ├── app/
│   │   ├── src/main/java/com/jlynnc/peacock/
│   │   │   ├── PeacockApp.kt           # Application
│   │   │   ├── MainActivity.kt
│   │   │   ├── protocol/               # 二进制协议
│   │   │   ├── network/                # UDP/TCP
│   │   │   ├── storage/                # SQLite (Room)
│   │   │   ├── model/                  # 数据模型
│   │   │   └── ui/                     # Jetpack Compose UI
│   │   └── AndroidManifest.xml
│   └── build.gradle.kts
└── docs/
```

## 二进制协议 — 最容易出错的地方

### 包头 (32 字节, Big-Endian)

```
Offset  Size  Field           Endian
0       4     magic           "PCOK" [0x50, 0x43, 0x4F, 0x4B]
4       2     version         Big-Endian u16 (值=1)
6       2     packet_type     Big-Endian u16
8       16    device_id       UUID 原始 16 字节
24      4     payload_length  Big-Endian u32
28      4     reserved        全零
```

用 `ByteBuffer` 或手动字节操作，注意 Java 默认就是 Big-Endian，这点比 Swift 方便。

### Payload 编码: bincode 1.x (Little-Endian!)

**⚠️ 关键：包头是 Big-Endian，Payload 里的整数是 Little-Endian！** 这是 Rust bincode 的格式。

```
字符串: [8字节 LE u64 长度] + [UTF-8 字节]
u16:   2字节 Little-Endian
u32:   4字节 Little-Endian
u64:   8字节 Little-Endian
bool:  1字节 (0 或 1)
Vec<T>: [8字节 LE u64 数量] + [连续元素]
```

**iOS 踩坑经验：** 我们最初用 `Data.withUnsafeBytes { buf.load(fromByteOffset:as:) }` 读整数，在 Data 子切片上因为内存对齐问题直接崩溃。Android 上用 `ByteBuffer.order(ByteOrder.LITTLE_ENDIAN)` 就没这个问题，但要注意 ByteBuffer 的 position 管理。

### Payload 结构体（字段按声明顺序序列化，无字段名）

**AnnouncePayload:**
```
device_name: String, platform: String, tcp_port: u16, features: u32, restricted_peers: Vec<PeerInfo>
```

**PeerInfo:**
```
device_id: String, device_name: String, ip_addr: String, tcp_port: u16, platform: String
```

**TextPayload:**
```
message_id: String (UUID), text: String, timestamp: u64 (毫秒)
```

**FileOfferPayload:**
```
transfer_id: String, file_name: String, file_size: u64, is_folder: bool, file_count: u32
```

**FileAcceptPayload:**
```
transfer_id: String, receiver_port: u16, resume_offset: u64
```

**FileRejectPayload:**
```
transfer_id: String
```

**SnippetSharePayload:**
```
title: String, content: String, tag: String, note: String
```

## 网络架构 — 全部基于 UDP 52000

### 核心原则

- **没有 TCP 服务器** — `server.rs` 是空文件，TCP 52001 已废弃
- **所有消息/信令走 UDP 52000** — Text、FileOffer、FileAccept、FileReject、SnippetShare
- **TCP 只用于文件数据传输** — 接收方动态开端口
- **不需要 TCP probe** — 发现完全靠 UDP

### 设备发现流程

```
Windows 发 UDP 广播 (multicast 224.0.1.100 + subnet broadcast + 255.255.255.255)
    ↓ port 52000
Android 收到 Announce
    ↓
Android 发 UDP 单播 AnnounceResponse 回 Windows IP:52000
    ↓
双方互相发现 ✅
```

Android 也应该尝试发广播（Android 没有 iOS 的 multicast entitlement 限制，应该能直接发）。

### UDP 监听

```kotlin
// 绑定 0.0.0.0:52000
val socket = DatagramSocket(52000)
socket.broadcast = true
socket.reuseAddress = true

// 加入组播组
val multicastSocket = MulticastSocket(52000)
multicastSocket.joinGroup(InetAddress.getByName("224.0.1.100"))
```

**⚠️ Android 权限：**
```xml
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.CHANGE_WIFI_MULTICAST_STATE" />
```

**⚠️ Wi-Fi Multicast Lock：** Android 默认会过滤组播包以省电。必须获取 `WifiManager.MulticastLock`：
```kotlin
val wifiManager = getSystemService(Context.WIFI_SERVICE) as WifiManager
val multicastLock = wifiManager.createMulticastLock("peacock")
multicastLock.setReferenceCounted(true)
multicastLock.acquire()
// App 退出时 release
```

### UDP 发送

Android 可以直接发广播，不需要特殊 entitlement：
```kotlin
val socket = DatagramSocket()
socket.broadcast = true
val broadcastAddr = InetAddress.getByName("255.255.255.255")
socket.send(DatagramPacket(data, data.size, broadcastAddr, 52000))
```

### 前后台生命周期 — iOS 踩坑最深的地方

**iOS 的问题：** 锁屏后系统暂停网络，UDP socket 失效，回到前台后收不到广播。

**Android 也有类似问题：** Doze 模式 + App Standby 会限制网络。解决方案：

```kotlin
// 在 Activity.onResume() 中重建 UDP 监听
override fun onResume() {
    super.onResume()
    discoveryService.restart() // 关闭旧 socket，创建新的
}
```

或者用 `LifecycleObserver`：
```kotlin
ProcessLifecycleOwner.get().lifecycle.addObserver(object : DefaultLifecycleObserver {
    override fun onStart(owner: LifecycleOwner) {
        // App 回到前台
        discoveryService.restart()
    }
})
```

## 文件传输 (TCP)

### 发送流程
1. 创建 send task，发 FileOffer (UDP)
2. 等对方 FileAccept (UDP) → 拿到 receiver_port
3. TCP 连接到 对方IP:receiver_port
4. 64KB 分块发送文件数据

### 接收流程
1. 收到 FileOffer (UDP) → 创建 receive task
2. 用户点接收 → 开 TCP ServerSocket（随机端口）
3. 发 FileAccept (UDP) 告诉对方端口号
4. accept() 等待连接 → 接收文件数据
5. 写入 .part 临时文件 → 完成后重命名

**iOS 踩坑：** `NWListener` (Apple 的 TCP listener API) 在模拟器上报 "Invalid argument"，最终改用 BSD socket。Android 用 `ServerSocket` 就行，简单可靠。

### 断点续传
- 检查是否有 .part 文件
- 有的话 FileAccept 里带 `resume_offset = 已有字节数`
- 发送方 seek 到该位置开始发

### 文件夹传输
1. 发送方先写 manifest: `[u64 LE manifest_len][JSON manifest]`
2. manifest 格式: `[{"relative_path": "a/b.txt", "size": 1234}, ...]`
3. 然后按 manifest 顺序依次发送每个文件的原始字节

### 重名处理
文件已存在时加后缀: `file(1).txt`, `file(2).txt`

## SQLite 数据库

```sql
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    direction TEXT NOT NULL,    -- "sent" / "received"
    content TEXT NOT NULL,
    msg_type TEXT NOT NULL DEFAULT 'text',  -- "text" / "file" / "snippet"
    timestamp INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'sent'
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE snippets (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL DEFAULT '新建片段',
    content TEXT NOT NULL DEFAULT '',
    tag TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    copy_count INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE known_devices (
    device_id TEXT PRIMARY KEY,
    device_name TEXT NOT NULL,
    ip_addr TEXT NOT NULL,
    tcp_port INTEGER NOT NULL,
    platform TEXT NOT NULL,
    last_seen INTEGER NOT NULL
);
```

Settings keys: `device_id`, `device_name`, `download_dir`, `auto_accept`, `max_concurrent`, `theme`, `locale`, `db_version`

Android 建议用 Room，但直接 SQLiteOpenHelper 也行。

## UI (Jetpack Compose)

### 三个 Tab
1. **设备列表** — 在线设备，点击进入聊天
2. **片段** — 列表 + 编辑器，支持 `[[快速复制]]` 芯片
3. **设置** — 设备名、下载目录、自动接收、主题、语言

### 聊天界面
- 消息气泡：发送方 teal 背景靠右 + "Me" 头像，接收方灰色靠左 + 设备名 + 平台图标
- 输入栏：+ 号 → 展开面板（相册、相机、文件、片段）
- 文件卡片：文件名 + 大小 + 进度条 + 接收/拒绝/分享按钮

### 片段编辑器 — 快速复制芯片

这是最复杂的 UI 组件。`[[text]]` 在编辑器里内联渲染为绿色芯片：

- **始终所见即所得**，没有编辑/阅读模式切换
- **`[[]]` 不显示** — 存储格式有方括号，显示时隐藏
- **芯片不可编辑** — 但周围文字可以正常编辑
- **点击芯片** → 复制到剪贴板 + 闪烁动画
- **长按芯片** → 上下文菜单（复制 / 取消标记）
- **选中文字 → 标记** → 系统选择菜单自定义 action，或浮动按钮
- **删除芯片** → 整个 `[[text]]` 一起删

iOS 上用 UITextView + NSAttributedString + 自定义 attribute 实现。Android 上建议用 `EditText` + `SpannableStringBuilder` + `ClickableSpan` / `ReplacementSpan`。

### 主题
- 跟随系统 / 亮色 / 暗色
- 主色调 teal: `#0d9488`

### i18n
- 简体中文 / English
- 存 SQLite `settings.locale`

## platform 字符串

Android 端 platform 设为 `"android"`，其他平台的值：
- `"ios"`, `"macos"`, `"windows"`, `"linux"`

## 关键常量

```
UDP_PORT = 52000
TCP_PORT = 52001  // 实际未使用，但 AnnouncePayload 里要填
MULTICAST_ADDR = "224.0.1.100"
BEACON_INTERVAL = 10 秒
OFFLINE_TIMEOUT = 30 秒
TIMEOUT_CHECK_INTERVAL = 15 秒
FILE_CHUNK_SIZE = 65536 (64KB)
FEATURES = 0xFFFF
PROTOCOL_VERSION = 1
```

## 总结：iOS 版本的主要踩坑点（Android 注意避免）

1. **内存对齐** — 读 payload 整数时不要用 unsafe 指针 load，逐字节拼装最安全
2. **NWListener 不可靠** — Android 用 ServerSocket 没这问题
3. **CheckedContinuation 重复 resume** — Android 用回调或 Flow，注意不要多次 complete
4. **前后台切换** — socket 失效要重建，这是两个平台都有的问题
5. **广播权限** — Android 需要 MulticastLock，iOS 需要 multicast entitlement
6. **模拟器 vs 真机** — 网络行为不同，要在真机上测试
