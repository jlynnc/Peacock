import Foundation
import SwiftUI

// MARK: - Locale Manager

@MainActor
final class LocaleManager: ObservableObject {
    @Published var current: AppLocale = .system

    var resolved: AppLocale {
        if current == .system {
            return Locale.current.language.languageCode?.identifier == "zh" ? .zhCN : .en
        }
        return current
    }

    func t(_ key: String) -> String {
        let table = resolved == .zhCN ? L10n.zhCN : L10n.en
        return table[key] ?? key
    }
}

enum AppLocale: String, CaseIterable {
    case system
    case zhCN = "zh-CN"
    case en

    var displayName: String {
        switch self {
        case .system: return "跟随系统 / Follow System"
        case .zhCN: return "简体中文"
        case .en: return "English"
        }
    }
}

// MARK: - String Tables

enum L10n {
    static let zhCN: [String: String] = [
        // Tabs
        "tab.devices": "设备",
        "tab.snippets": "片段",
        "tab.settings": "设置",

        // Devices
        "devices.search": "搜索设备...",
        "devices.searching": "正在搜索设备...",
        "devices.empty": "未发现设备",
        "devices.empty.hint": "确保其他设备在同一局域网中",
        "devices.online": "在线",

        // Chat
        "chat.input": "输入消息...",
        "chat.send_failed": "发送失败",
        "chat.photos": "相册",
        "chat.camera": "拍摄",
        "chat.snippets": "片段",
        "chat.files": "文件",
        "chat.select_snippet": "选择片段",

        // Snippets
        "snippets.search": "搜索片段...",
        "snippets.new": "新建片段",
        "snippets.empty": "暂无片段",
        "snippets.empty.hint": "点击右上角 + 创建新片段",
        "snippets.rename": "重命名",
        "snippets.share": "分享",
        "snippets.pin": "置顶",
        "snippets.delete": "删除",
        "snippets.copy": "复制内容",
        "snippets.note": "备注",
        "snippets.delete_confirm": "确定要删除这个片段吗？",
        "snippets.share_to": "分享到设备",
        "snippets.quick_copy": "标记为快速复制",
        "snippets.unmark": "取消标记",

        // Settings
        "settings.title": "设置",
        "settings.device_name": "设备名称",
        "settings.device_name.hint": "其他设备看到的名称",
        "settings.download_dir": "默认下载目录",
        "settings.auto_accept": "自动接收文件",
        "settings.auto_accept.hint": "自动接收其他设备发来的文件",
        "settings.max_concurrent": "最大并发传输",
        "settings.theme": "暗色主题",
        "settings.theme.system": "跟随系统",
        "settings.theme.light": "亮色",
        "settings.theme.dark": "暗色",
        "settings.language": "语言",
        "settings.about": "关于 Peacock",
        "settings.version": "版本",
        "settings.protocol": "协议版本",
        "settings.device_id": "设备 ID",
        "settings.transfer": "传输",
        "settings.appearance": "外观",
        "settings.general": "通用",

        // Transfer
        "transfer.accept": "接收",
        "transfer.reject": "拒绝",
        "transfer.save_as": "另存为",
        "transfer.open_folder": "打开目录",
        "transfer.completed": "已完成",
        "transfer.failed": "失败",
        "transfer.rejected": "已拒绝",
        "transfer.sending": "发送中",
        "transfer.receiving": "接收中",

        // Common
        "common.cancel": "取消",
        "common.confirm": "确认",
        "common.delete": "删除",
        "common.copy": "复制",
        "common.save": "保存",
    ]

    static let en: [String: String] = [
        // Tabs
        "tab.devices": "Devices",
        "tab.snippets": "Snippets",
        "tab.settings": "Settings",

        // Devices
        "devices.search": "Search devices...",
        "devices.searching": "Searching for devices...",
        "devices.empty": "No devices found",
        "devices.empty.hint": "Make sure other devices are on the same LAN",
        "devices.online": "Online",

        // Chat
        "chat.input": "Type a message...",
        "chat.send_failed": "Send failed",
        "chat.photos": "Photos",
        "chat.camera": "Camera",
        "chat.snippets": "Snippets",
        "chat.files": "Files",
        "chat.select_snippet": "Select Snippet",

        // Snippets
        "snippets.search": "Search snippets...",
        "snippets.new": "New Snippet",
        "snippets.empty": "No snippets",
        "snippets.empty.hint": "Tap + to create a new snippet",
        "snippets.rename": "Rename",
        "snippets.share": "Share",
        "snippets.pin": "Pin to Top",
        "snippets.delete": "Delete",
        "snippets.copy": "Copy Content",
        "snippets.note": "Note",
        "snippets.delete_confirm": "Delete this snippet?",
        "snippets.share_to": "Share to Device",
        "snippets.quick_copy": "Mark as Quick Copy",
        "snippets.unmark": "Unmark",

        // Settings
        "settings.title": "Settings",
        "settings.device_name": "Device Name",
        "settings.device_name.hint": "Name visible to other devices",
        "settings.download_dir": "Default Download Folder",
        "settings.auto_accept": "Auto-accept Files",
        "settings.auto_accept.hint": "Automatically accept files from other devices",
        "settings.max_concurrent": "Max Concurrent Transfers",
        "settings.theme": "Dark Theme",
        "settings.theme.system": "Follow System",
        "settings.theme.light": "Light",
        "settings.theme.dark": "Dark",
        "settings.language": "Language",
        "settings.about": "About Peacock",
        "settings.version": "Version",
        "settings.protocol": "Protocol Version",
        "settings.device_id": "Device ID",
        "settings.transfer": "Transfer",
        "settings.appearance": "Appearance",
        "settings.general": "General",

        // Transfer
        "transfer.accept": "Accept",
        "transfer.reject": "Reject",
        "transfer.save_as": "Save As",
        "transfer.open_folder": "Open Folder",
        "transfer.completed": "Completed",
        "transfer.failed": "Failed",
        "transfer.rejected": "Rejected",
        "transfer.sending": "Sending",
        "transfer.receiving": "Receiving",

        // Common
        "common.cancel": "Cancel",
        "common.confirm": "Confirm",
        "common.delete": "Delete",
        "common.copy": "Copy",
        "common.save": "Save",
    ]
}
