import SwiftUI

struct SnippetEditorView: View {
    @EnvironmentObject var appState: AppState
    let snippetId: String

    @State private var title: String = ""
    @State private var content: String = ""
    @State private var note: String = ""
    @State private var saveStatus: SaveStatus = .idle
    @State private var showShareSheet = false
    @State private var showDeleteConfirm = false

    var snippet: Snippet? {
        appState.snippets.first { $0.id == snippetId }
    }

    var body: some View {
        VStack(spacing: 0) {
            // Toolbar
            HStack {
                if saveStatus != .idle {
                    HStack(spacing: 4) {
                        if saveStatus == .saving {
                            ProgressView()
                                .scaleEffect(0.6)
                        }
                        Text(saveStatus == .saving ? "保存中..." : "已保存")
                            .font(.system(size: 12))
                            .foregroundStyle(saveStatus == .saved ? Color.onlineGreen : .secondary)
                    }
                }

                Spacer()

                Button {
                    copyContent()
                } label: {
                    Image(systemName: "doc.on.doc")
                        .font(.system(size: 16))
                }

                Button {
                    showShareSheet = true
                } label: {
                    Image(systemName: "square.and.arrow.up")
                        .font(.system(size: 16))
                }

                Button(role: .destructive) {
                    showDeleteConfirm = true
                } label: {
                    Image(systemName: "trash")
                        .font(.system(size: 16))
                }
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
            .background(Color(.secondarySystemBackground))

            Divider()

            // Content editor
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    // Title
                    TextField("标题", text: $title)
                        .font(.system(size: 18, weight: .semibold))
                        .onChange(of: title, perform: { _ in scheduleSave() })

                    // Content with quick-copy chips rendered below
                    VStack(alignment: .leading, spacing: 8) {
                        TextEditor(text: $content)
                            .font(.system(size: 14, design: .monospaced))
                            .frame(minHeight: 200)
                            .scrollContentBackground(.hidden)
                            .onChange(of: content, perform: { _ in scheduleSave() })

                        // Quick copy chips
                        if !content.isEmpty {
                            let segments = parseChips(content)
                            if segments.contains(where: { if case .chip = $0 { return true }; return false }) {
                                FlowLayout(spacing: 6) {
                                    ForEach(Array(segments.enumerated()), id: \.offset) { _, segment in
                                        if case .chip(let text) = segment {
                                            Button {
                                                UIPasteboard.general.string = text
                                            } label: {
                                                Text(text)
                                                    .font(.system(size: 13))
                                                    .foregroundStyle(Color.peacockTeal)
                                                    .padding(.horizontal, 8)
                                                    .padding(.vertical, 4)
                                                    .background(Color.peacockTealLight, in: RoundedRectangle(cornerRadius: 6))
                                                    .overlay(
                                                        RoundedRectangle(cornerRadius: 6)
                                                            .strokeBorder(Color.peacockTeal.opacity(0.3), lineWidth: 0.5)
                                                    )
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Divider()

                    // Note
                    TextField("备注", text: $note)
                        .font(.system(size: 13))
                        .foregroundStyle(.secondary)
                        .onChange(of: note, perform: { _ in scheduleSave() })
                }
                .padding(16)
            }
        }
        .navigationTitle(title.isEmpty ? "片段" : title)
        .navigationBarTitleDisplayMode(.inline)
        .onAppear { loadSnippet() }
        .confirmationDialog("确定要删除这个片段吗？", isPresented: $showDeleteConfirm, titleVisibility: .visible) {
            Button("删除", role: .destructive) {
                appState.deleteSnippet(snippetId)
            }
        }
        .sheet(isPresented: $showShareSheet) {
            DevicePickerSheet(snippetId: snippetId)
                .environmentObject(appState)
        }
    }

    private func loadSnippet() {
        guard let s = snippet else { return }
        title = s.title
        content = s.content
        note = s.note
    }

    private func scheduleSave() {
        saveStatus = .saving
        // Debounce: save after 600ms
        NSObject.cancelPreviousPerformRequests(withTarget: self)
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.6) { [self] in
            performSave()
        }
    }

    private func performSave() {
        guard var s = snippet else { return }
        s.title = title
        s.content = content
        s.note = note
        appState.updateSnippet(s)
        saveStatus = .saved
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
            if saveStatus == .saved { saveStatus = .idle }
        }
    }

    private func copyContent() {
        #if os(iOS)
        UIPasteboard.general.string = content
        #elseif os(macOS)
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(content, forType: .string)
        #endif
    }

    private func parseChips(_ text: String) -> [ContentSegment] {
        var segments: [ContentSegment] = []
        let pattern = /\[\[(.+?)\]\]/
        var lastEnd = text.startIndex

        for match in text.matches(of: pattern) {
            let before = String(text[lastEnd..<match.range.lowerBound])
            if !before.isEmpty { segments.append(.plain(before)) }
            segments.append(.chip(String(match.1)))
            lastEnd = match.range.upperBound
        }
        let remaining = String(text[lastEnd...])
        if !remaining.isEmpty { segments.append(.plain(remaining)) }
        return segments
    }
}

enum SaveStatus {
    case idle, saving, saved
}

// MARK: - Flow Layout

struct FlowLayout: Layout {
    var spacing: CGFloat = 8

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let result = layout(proposal: proposal, subviews: subviews)
        return result.size
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let result = layout(proposal: proposal, subviews: subviews)
        for (index, position) in result.positions.enumerated() {
            subviews[index].place(at: CGPoint(x: bounds.minX + position.x, y: bounds.minY + position.y),
                                   proposal: .unspecified)
        }
    }

    private func layout(proposal: ProposedViewSize, subviews: Subviews) -> (size: CGSize, positions: [CGPoint]) {
        let maxWidth = proposal.width ?? .infinity
        var positions: [CGPoint] = []
        var x: CGFloat = 0
        var y: CGFloat = 0
        var lineHeight: CGFloat = 0

        for subview in subviews {
            let size = subview.sizeThatFits(.unspecified)
            if x + size.width > maxWidth && x > 0 {
                x = 0
                y += lineHeight + spacing
                lineHeight = 0
            }
            positions.append(CGPoint(x: x, y: y))
            lineHeight = max(lineHeight, size.height)
            x += size.width + spacing
        }

        return (CGSize(width: maxWidth, height: y + lineHeight), positions)
    }
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
                            Text(device.deviceName)
                                .font(.system(size: 15, weight: .medium))
                            Text(device.ipAddr)
                                .font(.system(size: 12))
                                .foregroundStyle(.secondary)
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
