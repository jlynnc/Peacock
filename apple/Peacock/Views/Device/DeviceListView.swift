import SwiftUI

struct DeviceListView: View {
    @EnvironmentObject var appState: AppState
    @State private var searchText = ""

    private var t: (String) -> String { appState.locale.t }

    var filteredDevices: [DeviceInfo] {
        let online = appState.discovery.onlineDevices
        if searchText.isEmpty { return online }
        return online.filter {
            $0.deviceName.localizedCaseInsensitiveContains(searchText) ||
            $0.ipAddr.contains(searchText)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            // Custom header: 设备 + ● count
            HStack {
                Text(t("tab.devices"))
                    .font(.system(size: 28, weight: .bold))
                Spacer()
                HStack(spacing: 4) {
                    Circle()
                        .fill(Color.onlineGreen)
                        .frame(width: 8, height: 8)
                    Text("\(appState.discovery.onlineCount)")
                        .font(.system(size: 15))
                        .foregroundStyle(.secondary)
                }
            }
            .padding(.horizontal, 16)
            .padding(.top, 8)
            .padding(.bottom, 12)

            // Search bar
            HStack {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                TextField(t("devices.search"), text: $searchText)
                    .font(.system(size: 16))
                if !searchText.isEmpty {
                    Button {
                        searchText = ""
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundStyle(.tertiary)
                    }
                }
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 8)
            .background(Color(.tertiarySystemFill), in: RoundedRectangle(cornerRadius: 10))
            .padding(.horizontal, 16)
            .padding(.bottom, 8)

            // Device list
            List {
                if filteredDevices.isEmpty {
                    VStack(spacing: 12) {
                        Image(systemName: "wifi.slash")
                            .font(.system(size: 40))
                            .foregroundStyle(.tertiary)
                        Text(t("devices.empty"))
                            .font(.headline)
                            .foregroundStyle(.secondary)
                        Text(t("devices.empty.hint"))
                            .font(.subheadline)
                            .foregroundStyle(.tertiary)
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 60)
                    .listRowBackground(Color.clear)
                    .listRowSeparator(.hidden)
                } else {
                    ForEach(filteredDevices) { device in
                        NavigationLink(value: device.deviceId) {
                            DeviceRowView(device: device)
                        }
                    }
                }
            }
            .listStyle(.plain)
        }
        .navigationBarHidden(true)
        .navigationDestination(for: String.self) { deviceId in
            ChatView(deviceId: deviceId)
        }
    }
}

struct DeviceRowView: View {
    @EnvironmentObject var appState: AppState
    let device: DeviceInfo

    var lastMessage: ChatMessage? {
        appState.conversations[device.deviceId]?.lastMessage
    }

    var unreadCount: Int {
        appState.conversations[device.deviceId]?.unreadCount ?? 0
    }

    var body: some View {
        HStack(spacing: 13) {
            // Avatar
            ZStack {
                RoundedRectangle(cornerRadius: 12)
                    .fill(
                        LinearGradient(
                            colors: [Color.peacockTeal, Color.peacockTealDark],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                Image(systemName: device.platformIcon)
                    .font(.system(size: 20))
                    .foregroundStyle(.white)
            }
            .frame(width: 48, height: 48)

            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Circle()
                        .fill(appState.discovery.isBroadcastRestricted(device.deviceId) ? Color.orange : Color.onlineGreen)
                        .frame(width: 8, height: 8)
                    Text(device.deviceName)
                        .font(.system(size: 16, weight: .semibold))
                        .lineLimit(1)
                    Spacer()
                    if let msg = lastMessage {
                        Text(FormatUtils.relativeTime(msg.date))
                            .font(.system(size: 12))
                            .foregroundStyle(.tertiary)
                    }
                }

                HStack {
                    if let msg = lastMessage {
                        Text(msg.msgType == .text ? msg.content : "[文件]")
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    } else {
                        Text(device.ipAddr)
                            .font(.system(size: 13))
                            .foregroundStyle(.tertiary)
                    }
                    Spacer()
                    if unreadCount > 0 {
                        Text("\(unreadCount)")
                            .font(.system(size: 12, weight: .semibold))
                            .foregroundStyle(.white)
                            .padding(.horizontal, 6)
                            .frame(minWidth: 20, minHeight: 20)
                            .background(Color.peacockTeal, in: Capsule())
                    }
                }
            }
        }
        .padding(.vertical, 4)
    }
}
