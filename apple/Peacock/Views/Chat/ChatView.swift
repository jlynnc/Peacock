import SwiftUI
import PhotosUI
import UniformTypeIdentifiers

struct ChatView: View {
    @EnvironmentObject var appState: AppState
    let deviceId: String

    @State private var messageText = ""
    @State private var showPlusPanel = false
    @State private var showPhotoPicker = false
    @State private var showFilePicker = false
    @State private var showCamera = false
    @State private var showSnippetPicker = false
    @State private var selectedPhotos: [PhotosPickerItem] = []

    private var t: (String) -> String { appState.locale.t }

    var device: DeviceInfo? {
        appState.discovery.getDevice(deviceId)
    }

    var messages: [ChatMessage] {
        appState.conversations[deviceId]?.messages ?? []
    }

    var body: some View {
        VStack(spacing: 0) {
            // Messages
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 12) {
                        ForEach(messages) { message in
                            ChatBubbleView(message: message)
                        }
                    }
                    .padding()
                }
                .onChange(of: messages.count, perform: { _ in
                    if let last = messages.last {
                        withAnimation {
                            proxy.scrollTo(last.id, anchor: .bottom)
                        }
                    }
                })
            }

            Divider()

            // Input area
            VStack(spacing: 0) {
                HStack(alignment: .center, spacing: 8) {
                    // Plus button
                    Button {
                        withAnimation(.easeInOut(duration: 0.2)) {
                            showPlusPanel.toggle()
                        }
                    } label: {
                        Image(systemName: showPlusPanel ? "xmark.circle.fill" : "plus.circle.fill")
                            .font(.system(size: 26))
                            .foregroundStyle(Color.peacockTeal)
                            .frame(width: 36, height: 36)
                    }

                    // Text field
                    TextField(t("chat.input"), text: $messageText, axis: .vertical)
                        .textFieldStyle(.plain)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 8)
                        .frame(minHeight: 36)
                        .background(Color(.tertiarySystemFill), in: RoundedRectangle(cornerRadius: 18))
                        .lineLimit(1...5)

                    // Send button
                    Button {
                        send()
                    } label: {
                        Image(systemName: "arrow.up.circle.fill")
                            .font(.system(size: 26))
                            .foregroundStyle(messageText.isEmpty ? Color(.tertiaryLabel) : Color.peacockTeal)
                            .frame(width: 36, height: 36)
                    }
                    .disabled(messageText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
                .padding(.horizontal, 12)
                .padding(.vertical, 6)

                // Plus panel
                if showPlusPanel {
                    PlusPanelView(
                        titles: [t("chat.photos"), t("chat.camera"), t("chat.files"), t("chat.snippets")],
                        onPhotos: { showPhotoPicker = true },
                        onCamera: { showCamera = true },
                        onFiles: { showFilePicker = true },
                        onSnippets: { showSnippetPicker = true }
                    )
                    .transition(.move(edge: .bottom).combined(with: .opacity))
                }
            }
            .background(Color(.systemBackground))
        }
        .navigationTitle(device?.deviceName ?? "")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .principal) {
                VStack(spacing: 2) {
                    Text(device?.deviceName ?? "")
                        .font(.headline)
                    if device?.isOnline == true {
                        HStack(spacing: 4) {
                            Circle()
                                .fill(Color.onlineGreen)
                                .frame(width: 6, height: 6)
                            Text("在线")
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }
        }
        .onAppear {
            appState.loadHistory(for: deviceId)
            appState.clearUnread(for: deviceId)
            appState.selectedDeviceId = deviceId
        }
        .onDisappear {
            if appState.selectedDeviceId == deviceId {
                appState.selectedDeviceId = nil
            }
        }
        .photosPicker(isPresented: $showPhotoPicker, selection: $selectedPhotos,
                      maxSelectionCount: 10, matching: .images)
        .onChange(of: selectedPhotos, perform: { items in
            guard !items.isEmpty else { return }
            handlePhotos(items)
        })
        .fileImporter(isPresented: $showFilePicker, allowedContentTypes: [.item],
                      allowsMultipleSelection: true) { result in
            handleFiles(result)
        }
        .sheet(isPresented: $showSnippetPicker) {
            SnippetPickerSheet(deviceId: deviceId)
                .environmentObject(appState)
        }
        .fullScreenCover(isPresented: $showCamera) {
            CameraPicker { image in
                handleCameraImage(image)
            }
            .ignoresSafeArea()
        }
    }

    private func send() {
        let text = messageText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return }
        appState.sendMessage(to: deviceId, text: text)
        messageText = ""
    }

    private func handlePhotos(_ items: [PhotosPickerItem]) {
        for item in items {
            item.loadTransferable(type: Data.self) { result in
                if case .success(let data) = result, let data {
                    let tempDir = FileManager.default.temporaryDirectory
                    let fileName = "photo_\(UUID().uuidString).jpg"
                    let tempURL = tempDir.appendingPathComponent(fileName)
                    try? data.write(to: tempURL)
                    Task { @MainActor in
                        appState.sendFile(to: deviceId, url: tempURL)
                    }
                }
            }
        }
        selectedPhotos = []
    }

    private func handleFiles(_ result: Result<[URL], Error>) {
        guard case .success(let urls) = result else { return }
        for url in urls {
            guard url.startAccessingSecurityScopedResource() else { continue }
            defer { url.stopAccessingSecurityScopedResource() }

            // Copy to temp to ensure access
            let tempURL = FileManager.default.temporaryDirectory.appendingPathComponent(url.lastPathComponent)
            try? FileManager.default.copyItem(at: url, to: tempURL)
            appState.sendFile(to: deviceId, url: tempURL)
        }
    }

    private func handleCameraImage(_ image: UIImage) {
        guard let data = image.jpegData(compressionQuality: 0.8) else { return }
        let tempDir = FileManager.default.temporaryDirectory
        let fileName = "camera_\(UUID().uuidString).jpg"
        let tempURL = tempDir.appendingPathComponent(fileName)
        try? data.write(to: tempURL)
        appState.sendFile(to: deviceId, url: tempURL)
    }
}

// MARK: - Camera Picker

#if os(iOS)
struct CameraPicker: UIViewControllerRepresentable {
    let onImageCaptured: (UIImage) -> Void
    @Environment(\.dismiss) private var dismiss

    func makeUIViewController(context: Context) -> UIImagePickerController {
        let picker = UIImagePickerController()
        if UIImagePickerController.isSourceTypeAvailable(.camera) {
            picker.sourceType = .camera
        }
        picker.delegate = context.coordinator
        return picker
    }

    func updateUIViewController(_ uiViewController: UIImagePickerController, context: Context) {}

    func makeCoordinator() -> Coordinator {
        Coordinator(onImageCaptured: onImageCaptured, dismiss: dismiss)
    }

    class Coordinator: NSObject, UIImagePickerControllerDelegate, UINavigationControllerDelegate {
        let onImageCaptured: (UIImage) -> Void
        let dismiss: DismissAction

        init(onImageCaptured: @escaping (UIImage) -> Void, dismiss: DismissAction) {
            self.onImageCaptured = onImageCaptured
            self.dismiss = dismiss
        }

        func imagePickerController(_ picker: UIImagePickerController,
                                   didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]) {
            if let image = info[.originalImage] as? UIImage {
                onImageCaptured(image)
            }
            dismiss()
        }

        func imagePickerControllerDidCancel(_ picker: UIImagePickerController) {
            dismiss()
        }
    }
}
#endif

// MARK: - Plus Panel

struct PlusPanelView: View {
    var titles: [String] = ["相册", "拍摄", "文件", "片段"]
    let onPhotos: () -> Void
    let onCamera: () -> Void
    let onFiles: () -> Void
    let onSnippets: () -> Void

    var body: some View {
        HStack(spacing: 20) {
            PlusPanelItem(icon: "photo", title: titles[0], color: .blue, action: onPhotos)
            PlusPanelItem(icon: "camera", title: titles[1], color: .orange, action: onCamera)
            PlusPanelItem(icon: "doc", title: titles[2], color: .purple, action: onFiles)
            PlusPanelItem(icon: "doc.text", title: titles[3], color: .green, action: onSnippets)
        }
        .padding(.vertical, 16)
        .padding(.horizontal, 20)
        .background(Color(.systemBackground))
    }
}

struct PlusPanelItem: View {
    let icon: String
    let title: String
    let color: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(spacing: 8) {
                ZStack {
                    RoundedRectangle(cornerRadius: 14)
                        .fill(color.opacity(0.1))
                    Image(systemName: icon)
                        .font(.system(size: 24))
                        .foregroundStyle(color)
                }
                .frame(width: 56, height: 56)

                Text(title)
                    .font(.system(size: 12))
                    .foregroundStyle(.primary)
            }
        }
    }
}

// MARK: - Snippet Picker Sheet

struct SnippetPickerSheet: View {
    @EnvironmentObject var appState: AppState
    @Environment(\.dismiss) var dismiss
    let deviceId: String

    var body: some View {
        NavigationStack {
            List(appState.snippets) { snippet in
                Button {
                    appState.shareSnippet(to: deviceId, snippet: snippet)
                    dismiss()
                } label: {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(snippet.title)
                            .font(.headline)
                        Text(snippet.content.prefix(80))
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .lineLimit(2)
                    }
                }
            }
            .navigationTitle(appState.locale.t("chat.select_snippet"))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("取消") { dismiss() }
                }
            }
        }
        .presentationDetents([.medium, .large])
    }
}

// MARK: - File Offer Sheet

struct FileOfferSheet: View {
    @EnvironmentObject var appState: AppState
    let offer: FileOfferInfo

    var body: some View {
        VStack(spacing: 20) {
            Image(systemName: offer.isFolder ? "folder.fill" : FormatUtils.fileIcon(for: offer.fileName))
                .font(.system(size: 48))
                .foregroundStyle(Color.peacockTeal)

            Text(offer.fromDeviceName)
                .font(.headline)
            Text("想发送\(offer.isFolder ? "文件夹" : "文件")给你")
                .foregroundStyle(.secondary)

            VStack(spacing: 4) {
                Text(offer.fileName)
                    .font(.system(size: 16, weight: .semibold))
                Text(FormatUtils.fileSize(offer.fileSize))
                    .foregroundStyle(.secondary)
                if offer.isFolder {
                    Text("\(offer.fileCount) 个文件")
                        .foregroundStyle(.secondary)
                }
            }

            HStack(spacing: 16) {
                Button("拒绝") {
                    appState.rejectTransfer(offer.transferId)
                }
                .foregroundStyle(Color.dangerRed)
                .padding(.horizontal, 24)
                .padding(.vertical, 10)
                .background(Color.dangerRed.opacity(0.1), in: RoundedRectangle(cornerRadius: 10))

                Button("接收") {
                    appState.acceptTransfer(offer.transferId)
                }
                .foregroundStyle(.white)
                .padding(.horizontal, 24)
                .padding(.vertical, 10)
                .background(Color.peacockTeal, in: RoundedRectangle(cornerRadius: 10))
            }
        }
        .padding(24)
        .presentationDetents([.height(320)])
    }
}
