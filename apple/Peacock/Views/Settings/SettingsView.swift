import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var appState: AppState
    @State private var editingName: String = ""
    @State private var isEditingName = false

    var body: some View {
        List {
            // Device info
            Section {
                HStack(spacing: 14) {
                    ZStack {
                        RoundedRectangle(cornerRadius: 16)
                            .fill(
                                LinearGradient(
                                    colors: [Color.peacockTeal, Color.peacockTealDark],
                                    startPoint: .topLeading,
                                    endPoint: .bottomTrailing
                                )
                            )
                        Text("ME")
                            .font(.system(size: 16, weight: .bold))
                            .foregroundStyle(.white)
                    }
                    .frame(width: 56, height: 56)

                    VStack(alignment: .leading, spacing: 4) {
                        if isEditingName {
                            TextField("设备名称", text: $editingName, onCommit: {
                                appState.updateDeviceName(editingName)
                                isEditingName = false
                            })
                            .textFieldStyle(.roundedBorder)
                        } else {
                            Text(appState.deviceName)
                                .font(.system(size: 17, weight: .semibold))
                        }

                        HStack(spacing: 4) {
                            Text(NetworkUtils.currentPlatform.uppercased())
                                .font(.system(size: 11, weight: .medium))
                                .foregroundStyle(Color.peacockTeal)
                                .padding(.horizontal, 6)
                                .padding(.vertical, 2)
                                .background(Color.peacockTealLight, in: RoundedRectangle(cornerRadius: 4))

                            if let ip = NetworkUtils.getLocalIPAddress() {
                                Text(ip)
                                    .font(.system(size: 12))
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }

                    Spacer()

                    Button {
                        editingName = appState.deviceName
                        isEditingName.toggle()
                    } label: {
                        Image(systemName: "pencil")
                            .foregroundStyle(Color.peacockTeal)
                    }
                }
                .padding(.vertical, 4)
            }

            // Settings
            Section("传输") {
                HStack {
                    Label("下载目录", systemImage: "folder")
                    Spacer()
                    Text(appState.transferManager.downloadDir.lastPathComponent)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }

            Section("关于") {
                HStack {
                    Label("版本", systemImage: "info.circle")
                    Spacer()
                    Text("1.0.0")
                        .foregroundStyle(.secondary)
                }

                HStack {
                    Label("协议版本", systemImage: "network")
                    Spacer()
                    Text("PCOK v1")
                        .foregroundStyle(.secondary)
                }

                HStack {
                    Label("设备 ID", systemImage: "qrcode")
                    Spacer()
                    Text(String(appState.deviceId.prefix(8)) + "...")
                        .font(.system(size: 13, design: .monospaced))
                        .foregroundStyle(.secondary)
                }
            }
        }
        .navigationTitle("设置")
    }
}
