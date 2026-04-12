import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var appState: AppState

    @State private var editingName = ""
    @State private var isEditingName = false

    private var t: (String) -> String { appState.locale.t }

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Text(t("settings.title"))
                    .font(.system(size: 28, weight: .bold))
                Spacer()
            }
            .padding(.horizontal, 16)
            .padding(.top, 8)
            .padding(.bottom, 12)

            // Device info card
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
                    Text("Me")
                        .font(.system(size: 16, weight: .bold))
                        .foregroundStyle(.white)
                }
                .frame(width: 56, height: 56)

                VStack(alignment: .leading, spacing: 4) {
                    if isEditingName {
                        TextField(t("settings.device_name"), text: $editingName, onCommit: {
                            if !editingName.isEmpty {
                                appState.updateDeviceName(editingName)
                            }
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
            .padding(.horizontal, 16)
            .padding(.bottom, 4)

            Text(t("settings.device_name.hint"))
                .font(.system(size: 12))
                .foregroundStyle(.tertiary)
                .padding(.horizontal, 16)
                .padding(.bottom, 8)

        List {

            // Transfer
            Section(t("settings.transfer")) {
                HStack {
                    Label(t("settings.download_dir"), systemImage: "folder")
                    Spacer()
                    Text("Peacock Downloads")
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }

                Toggle(isOn: Binding(
                    get: { appState.autoAccept },
                    set: { appState.updateAutoAccept($0) }
                )) {
                    Label(t("settings.auto_accept"), systemImage: "arrow.down.doc")
                }
                .tint(Color.peacockTeal)

                Picker(selection: Binding(
                    get: { appState.maxConcurrent },
                    set: { appState.updateMaxConcurrent($0) }
                )) {
                    ForEach([1, 3, 5, 10, 20, 50], id: \.self) { value in
                        Text("\(value)").tag(value)
                    }
                } label: {
                    Label(t("settings.max_concurrent"), systemImage: "arrow.left.arrow.right")
                }
            }

            // Appearance
            Section(t("settings.appearance")) {
                Picker(selection: Binding(
                    get: { appState.theme },
                    set: { appState.updateTheme($0) }
                )) {
                    Text(t("settings.theme.system")).tag(AppTheme.system)
                    Text(t("settings.theme.light")).tag(AppTheme.light)
                    Text(t("settings.theme.dark")).tag(AppTheme.dark)
                } label: {
                    Label(t("settings.theme"), systemImage: "moon.circle")
                }

                Picker(selection: Binding(
                    get: { appState.locale.current },
                    set: { appState.updateLocale($0) }
                )) {
                    ForEach(AppLocale.allCases, id: \.self) { loc in
                        Text(loc.displayName).tag(loc)
                    }
                } label: {
                    Label(t("settings.language"), systemImage: "globe")
                }
            }

            // About
            Section(t("settings.about")) {
                HStack {
                    Label(t("settings.version"), systemImage: "info.circle")
                    Spacer()
                    Text("v0.1.0")
                        .foregroundStyle(.secondary)
                }

                HStack {
                    Label(t("settings.protocol"), systemImage: "network")
                    Spacer()
                    Text("PCOK v1")
                        .foregroundStyle(.secondary)
                }

                HStack {
                    Label(t("settings.device_id"), systemImage: "qrcode")
                    Spacer()
                    Text(String(appState.deviceId.prefix(8)) + "...")
                        .font(.system(size: 13, design: .monospaced))
                        .foregroundStyle(.secondary)
                }
            }
        }
        }
        .navigationBarHidden(true)
    }
}
