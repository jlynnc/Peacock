import SwiftUI

struct SnippetEditorView: View {
    @EnvironmentObject var appState: AppState
    let snippetId: String

    @State private var content: String = ""
    @State private var note: String = ""
    @State private var saveStatus: SaveStatus = .idle
    @State private var showShareSheet = false
    @State private var showDeleteConfirm = false
    @State private var selectedRange: NSRange = NSRange(location: 0, length: 0)

    var snippet: Snippet? {
        appState.snippets.first { $0.id == snippetId }
    }

    var body: some View {
        VStack(spacing: 0) {
            // Toolbar
            HStack(spacing: 12) {
                if saveStatus != .idle {
                    HStack(spacing: 4) {
                        if saveStatus == .saving {
                            ProgressView().scaleEffect(0.6)
                        }
                        Text(saveStatus == .saving ? "保存中..." : "已保存")
                            .font(.system(size: 12))
                            .foregroundStyle(saveStatus == .saved ? Color.onlineGreen : .secondary)
                    }
                }

                Spacer()

                Button { copyContent() } label: {
                    Image(systemName: "doc.on.doc").font(.system(size: 16))
                }
                Button { showShareSheet = true } label: {
                    Image(systemName: "square.and.arrow.up").font(.system(size: 16))
                }
                Button(role: .destructive) { showDeleteConfirm = true } label: {
                    Image(systemName: "trash").font(.system(size: 16))
                }
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
            .background(Color(.secondarySystemBackground))

            Divider()

            // Content — always editable, chips rendered inline
            // Content area (expands)
            ScrollView {
                ChipTextViewRepresentable(
                    text: $content,
                    selectedRange: $selectedRange,
                    onTextChange: { scheduleSave() }
                )
                .frame(minHeight: 200)
                .padding(16)
            }

            // Note — fixed at bottom, always one line
            Divider()
            HStack {
                Image(systemName: "note.text")
                    .font(.system(size: 12))
                    .foregroundStyle(.tertiary)
                TextField("备注", text: $note)
                    .font(.system(size: 13))
                    .foregroundStyle(.secondary)
                    .onChange(of: note, perform: { _ in scheduleSave() })
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
        }
        .navigationTitle(snippet?.title ?? "片段")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear { loadSnippet() }
        .confirmationDialog("确定要删除这个片段吗？", isPresented: $showDeleteConfirm, titleVisibility: .visible) {
            Button("删除", role: .destructive) { appState.deleteSnippet(snippetId) }
        }
        .sheet(isPresented: $showShareSheet) {
            DevicePickerSheet(snippetId: snippetId).environmentObject(appState)
        }
    }

    private func loadSnippet() {
        guard let s = snippet else { return }
        content = s.content
        note = s.note
    }

    private func scheduleSave() {
        saveStatus = .saving
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.6) { performSave() }
    }

    private func performSave() {
        guard var s = snippet else { return }
        s.content = content
        s.note = note
        appState.updateSnippet(s)
        saveStatus = .saved
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
            if saveStatus == .saved { saveStatus = .idle }
        }
    }

    private func copyContent() {
        let plain = content
            .replacingOccurrences(of: "[[", with: "")
            .replacingOccurrences(of: "]]", with: "")
        UIPasteboard.general.string = plain
    }

}

enum SaveStatus {
    case idle, saving, saved
}

// MARK: - Device Picker Sheet

struct DevicePickerSheet: View {
    @EnvironmentObject var appState: AppState
    @Environment(\.dismiss) var dismiss
    let snippetId: String

    var body: some View {
        NavigationStack {
            List(appState.discovery.onlineDevices) { device in
                Button {
                    if let snippet = appState.snippets.first(where: { $0.id == snippetId }) {
                        appState.shareSnippet(to: device.deviceId, snippet: snippet)
                    }
                    dismiss()
                } label: {
                    HStack(spacing: 12) {
                        Image(systemName: device.platformIcon)
                            .font(.system(size: 20))
                            .foregroundStyle(Color.peacockTeal)
                            .frame(width: 36, height: 36)
                            .background(Color.peacockTealLight, in: RoundedRectangle(cornerRadius: 8))
                        VStack(alignment: .leading) {
                            Text(device.deviceName).font(.system(size: 15, weight: .medium))
                            Text(device.ipAddr).font(.system(size: 12)).foregroundStyle(.secondary)
                        }
                    }
                }
            }
            .navigationTitle("分享到设备")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("取消") { dismiss() }
                }
            }
        }
        .presentationDetents([.medium])
    }
}
