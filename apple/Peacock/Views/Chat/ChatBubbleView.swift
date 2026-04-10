import SwiftUI

struct ChatBubbleView: View {
    @EnvironmentObject var appState: AppState
    let message: ChatMessage

    var isSent: Bool { message.direction == .sent }

    var device: DeviceInfo? {
        appState.discovery.getDevice(message.deviceId)
    }

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            if isSent { Spacer(minLength: 60) }

            // Received: avatar on left
            if !isSent {
                avatarView(for: device)
            }

            VStack(alignment: isSent ? .trailing : .leading, spacing: 2) {
                // Sender name (only for received)
                if !isSent {
                    Text(device?.deviceName ?? "Unknown")
                        .font(.system(size: 12, weight: .medium))
                        .foregroundStyle(.secondary)
                }

                // Message content
                switch message.msgType {
                case .text:
                    textBubble
                case .file:
                    ChatFileCardView(transferId: message.content)
                case .snippet:
                    ChatSnippetCardView(offerId: message.content, direction: message.direction)
                }

                // Time + status
                HStack(spacing: 4) {
                    Text(timeString)
                        .font(.chatTime)
                        .foregroundStyle(.tertiary)
                    if isSent {
                        statusIcon
                    }
                }
            }

            // Sent: avatar on right
            if isSent {
                meAvatarView
            }

            if !isSent { Spacer(minLength: 60) }
        }
        .id(message.id)
    }

    // MARK: - Avatar

    private func avatarView(for device: DeviceInfo?) -> some View {
        ZStack {
            RoundedRectangle(cornerRadius: 8)
                .fill(Color(.tertiarySystemFill))
            Image(systemName: device?.platformIcon ?? "desktopcomputer")
                .font(.system(size: 16))
                .foregroundStyle(.secondary)
        }
        .frame(width: 36, height: 36)
    }

    private var meAvatarView: some View {
        ZStack {
            RoundedRectangle(cornerRadius: 8)
                .fill(Color.peacockTeal)
            Text("Me")
                .font(.system(size: 12, weight: .bold))
                .foregroundStyle(.white)
        }
        .frame(width: 36, height: 36)
    }

    // MARK: - Bubble

    private var textBubble: some View {
        Text(message.content)
            .font(.chatBody)
            .foregroundStyle(isSent ? Color.bubbleSentText : .primary)
            .padding(.horizontal, 14)
            .padding(.vertical, 10)
            .background(
                isSent ? Color.bubbleSent : Color(.tertiarySystemFill),
                in: RoundedRectangle(cornerRadius: 18)
            )
            .overlay(
                RoundedRectangle(cornerRadius: 18)
                    .strokeBorder(
                        isSent ? Color.peacockTeal.opacity(0.2) : Color(.separator).opacity(0.3),
                        lineWidth: 0.5
                    )
            )
    }

    private var timeString: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm"
        return formatter.string(from: message.date)
    }

    @ViewBuilder
    private var statusIcon: some View {
        switch message.status {
        case .sending:
            ProgressView()
                .scaleEffect(0.5)
        case .sent:
            Image(systemName: "checkmark")
                .font(.system(size: 10))
                .foregroundStyle(.tertiary)
        case .failed:
            Image(systemName: "exclamationmark.circle")
                .font(.system(size: 10))
                .foregroundStyle(Color.dangerRed)
        }
    }
}

// MARK: - File Card

struct ChatFileCardView: View {
    @EnvironmentObject var appState: AppState
    let transferId: String

    var task: TransferTask? {
        appState.transferManager.getTask(transferId)
    }

    var body: some View {
        if let task {
            VStack(alignment: .leading, spacing: 8) {
                HStack(spacing: 10) {
                    Image(systemName: FormatUtils.fileIcon(for: task.fileName))
                        .font(.system(size: 22))
                        .foregroundStyle(Color.peacockTeal)
                        .frame(width: 42, height: 42)
                        .background(Color.peacockTealLight, in: RoundedRectangle(cornerRadius: 10))

                    VStack(alignment: .leading, spacing: 2) {
                        Text(task.fileName)
                            .font(.system(size: 14, weight: .medium))
                            .lineLimit(1)
                        Text(FormatUtils.fileSize(task.fileSize))
                            .font(.system(size: 12))
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                }

                if task.status == .active {
                    ProgressView(value: task.progress)
                        .tint(Color.peacockTeal)
                    HStack {
                        Text(FormatUtils.speed(task.speedBps))
                            .font(.system(size: 11))
                            .foregroundStyle(.secondary)
                        Spacer()
                        Text("\(FormatUtils.fileSize(task.transferredBytes)) / \(FormatUtils.fileSize(task.fileSize))")
                            .font(.system(size: 11))
                            .foregroundStyle(.secondary)
                    }
                }

                if task.status == .pending && task.direction == .receive {
                    HStack(spacing: 8) {
                        Button("接收") { appState.acceptTransfer(transferId) }
                            .font(.system(size: 13, weight: .medium))
                            .foregroundStyle(.white)
                            .padding(.horizontal, 14)
                            .padding(.vertical, 6)
                            .background(Color.peacockTeal, in: RoundedRectangle(cornerRadius: 8))

                        Button("拒绝") { appState.rejectTransfer(transferId) }
                            .font(.system(size: 13))
                            .foregroundStyle(Color.dangerRed)
                    }
                }

                if task.status == .completed {
                    Label("已完成", systemImage: "checkmark.circle.fill")
                        .font(.system(size: 12))
                        .foregroundStyle(Color.onlineGreen)
                }

                if task.status == .failed {
                    Label("失败", systemImage: "xmark.circle.fill")
                        .font(.system(size: 12))
                        .foregroundStyle(Color.dangerRed)
                }

                if task.status == .rejected {
                    Label("已拒绝", systemImage: "nosign")
                        .font(.system(size: 12))
                        .foregroundStyle(.secondary)
                }
            }
            .padding(12)
            .frame(minWidth: 260, maxWidth: 340)
            .background(Color(.secondarySystemBackground), in: RoundedRectangle(cornerRadius: 14))
        } else {
            Text("[文件]")
                .foregroundStyle(.secondary)
        }
    }
}

// MARK: - Snippet Card

struct ChatSnippetCardView: View {
    @EnvironmentObject var appState: AppState
    let offerId: String
    let direction: MessageDirection

    var offer: SnippetOffer? {
        appState.pendingSnippetOffers.first { $0.id == offerId }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 8) {
                Image(systemName: "doc.text")
                    .foregroundStyle(Color.peacockTeal)
                Text(offer?.title ?? "片段")
                    .font(.system(size: 14, weight: .medium))
            }

            if let offer {
                Text(String(offer.content.prefix(80)))
                    .font(.system(size: 13))
                    .foregroundStyle(.secondary)
                    .lineLimit(2)

                if direction == .received {
                    HStack(spacing: 8) {
                        Button("保存") { appState.acceptSnippetOffer(offerId) }
                            .font(.system(size: 13, weight: .medium))
                            .foregroundStyle(.white)
                            .padding(.horizontal, 12)
                            .padding(.vertical, 5)
                            .background(Color.peacockTeal, in: RoundedRectangle(cornerRadius: 6))

                        Button("忽略") { appState.rejectSnippetOffer(offerId) }
                            .font(.system(size: 13))
                            .foregroundStyle(.secondary)
                    }
                }
            } else {
                Text(direction == .sent ? "已发送" : "已处理")
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
            }
        }
        .padding(12)
        .frame(minWidth: 220, maxWidth: 320)
        .background(Color(.secondarySystemBackground), in: RoundedRectangle(cornerRadius: 14))
    }
}
