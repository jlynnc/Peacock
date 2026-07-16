import SwiftUI

/// Shown when the Share Extension hands files to the app: pick a device to send them to.
struct SharedFilesPicker: View {
    @EnvironmentObject var appState: AppState
    @Environment(\.dismiss) var dismiss

    private var t: (String) -> String { appState.locale.t }

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // What's being sent
                if !appState.pendingSharedFiles.isEmpty {
                    HStack(spacing: 10) {
                        Image(systemName: appState.pendingSharedFiles.count > 1
                              ? "doc.on.doc"
                              : FormatUtils.fileIcon(for: appState.pendingSharedFiles[0].lastPathComponent))
                            .font(.system(size: 20))
                            .foregroundStyle(Color.peacockTeal)
                            .frame(width: 40, height: 40)
                            .background(Color.peacockTealLight, in: RoundedRectangle(cornerRadius: 10))

                        VStack(alignment: .leading, spacing: 2) {
                            Text(appState.pendingSharedFiles.count > 1
                                 ? "\(appState.pendingSharedFiles.count) 个文件"
                                 : appState.pendingSharedFiles[0].lastPathComponent)
                                .font(.system(size: 15, weight: .medium))
                                .lineLimit(1)
                            Text(totalSizeText)
                                .font(.system(size: 12))
                                .foregroundStyle(.secondary)
                        }
                        Spacer()
                    }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 12)
                    .background(Color(.secondarySystemBackground))
                }

                Divider()

                if appState.discovery.onlineDevices.isEmpty {
                    VStack(spacing: 12) {
                        Image(systemName: "wifi.slash")
                            .font(.system(size: 36))
                            .foregroundStyle(.tertiary)
                        Text(t("devices.empty"))
                            .font(.headline)
                            .foregroundStyle(.secondary)
                        Text(t("devices.empty.hint"))
                            .font(.subheadline)
                            .foregroundStyle(.tertiary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    List(appState.discovery.onlineDevices) { device in
                        Button {
                            appState.sendSharedFiles(to: device.deviceId)
                            dismiss()
                        } label: {
                            HStack(spacing: 12) {
                                ZStack {
                                    RoundedRectangle(cornerRadius: 10)
                                        .fill(
                                            LinearGradient(
                                                colors: [Color.peacockTeal, Color.peacockTealDark],
                                                startPoint: .topLeading,
                                                endPoint: .bottomTrailing
                                            )
                                        )
                                    Image(systemName: device.platformIcon)
                                        .font(.system(size: 18))
                                        .foregroundStyle(.white)
                                }
                                .frame(width: 40, height: 40)

                                VStack(alignment: .leading, spacing: 2) {
                                    Text(device.deviceName)
                                        .font(.system(size: 15, weight: .medium))
                                    Text(device.ipAddr)
                                        .font(.system(size: 12))
                                        .foregroundStyle(.secondary)
                                }
                                Spacer()
                                Image(systemName: "paperplane.fill")
                                    .font(.system(size: 13))
                                    .foregroundStyle(Color.peacockTeal)
                            }
                        }
                    }
                    .listStyle(.plain)
                }
            }
            .navigationTitle(t("snippets.share_to"))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(t("common.cancel")) {
                        appState.discardSharedFiles()
                        dismiss()
                    }
                }
            }
        }
        .presentationDetents([.medium, .large])
    }

    private var totalSizeText: String {
        let total = appState.pendingSharedFiles.reduce(UInt64(0)) { sum, url in
            let size = (try? FileManager.default.attributesOfItem(atPath: url.path)[.size] as? UInt64) ?? 0
            return sum + (size ?? 0)
        }
        return FormatUtils.fileSize(total)
    }
}
