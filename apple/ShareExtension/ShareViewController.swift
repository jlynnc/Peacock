import UIKit
import UniformTypeIdentifiers

/// Receives items shared from other apps, copies them into the App Group inbox,
/// then hands off to the main Peacock app to pick a device and send.
class ShareViewController: UIViewController {

    private let label = UILabel()

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .systemBackground

        label.text = "正在准备…"
        label.textColor = .secondaryLabel
        label.font = .systemFont(ofSize: 15)
        label.textAlignment = .center
        label.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(label)
        NSLayoutConstraint.activate([
            label.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            label.centerYAnchor.constraint(equalTo: view.centerYAnchor)
        ])
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        processAttachments()
    }

    // MARK: - Attachment handling

    private func processAttachments() {
        let items = (extensionContext?.inputItems as? [NSExtensionItem]) ?? []
        let providers = items.flatMap { $0.attachments ?? [] }
        guard !providers.isEmpty else {
            finish()
            return
        }

        let group = DispatchGroup()
        for provider in providers {
            group.enter()
            handle(provider) { group.leave() }
        }

        group.notify(queue: .main) { [weak self] in
            self?.label.text = "正在打开 Peacock…"
            self?.openMainApp()
            self?.finish()
        }
    }

    private func handle(_ provider: NSItemProvider, completion: @escaping () -> Void) {
        // Files, photos and videos all resolve through a file representation.
        if provider.hasItemConformingToTypeIdentifier(UTType.item.identifier) {
            provider.loadFileRepresentation(forTypeIdentifier: UTType.item.identifier) { url, _ in
                // The temp URL is only valid inside this closure, so copy immediately.
                if let url { ShareInbox.store(url) }
                completion()
            }
            return
        }

        if provider.hasItemConformingToTypeIdentifier(UTType.url.identifier) {
            provider.loadItem(forTypeIdentifier: UTType.url.identifier) { item, _ in
                if let url = item as? URL, let data = url.absoluteString.data(using: .utf8) {
                    ShareInbox.store(data: data, name: "link-\(UUID().uuidString.prefix(6)).txt")
                }
                completion()
            }
            return
        }

        if provider.hasItemConformingToTypeIdentifier(UTType.plainText.identifier) {
            provider.loadItem(forTypeIdentifier: UTType.plainText.identifier) { item, _ in
                if let text = item as? String, let data = text.data(using: .utf8) {
                    ShareInbox.store(data: data, name: "text-\(UUID().uuidString.prefix(6)).txt")
                }
                completion()
            }
            return
        }

        completion()
    }

    // MARK: - Hand off to the main app

    private func openMainApp() {
        guard let url = URL(string: "\(ShareInbox.urlScheme)://\(ShareInbox.shareHost)") else { return }
        if !openViaResponderChain(url) {
            extensionContext?.open(url, completionHandler: nil)
        }
    }

    /// Share extensions have no UIApplication.shared, so reach it through the responder chain.
    private func openViaResponderChain(_ url: URL) -> Bool {
        var responder: UIResponder? = self
        while let current = responder {
            if let app = current as? UIApplication {
                app.open(url, options: [:], completionHandler: nil)
                return true
            }
            responder = current.next
        }
        return false
    }

    private func finish() {
        extensionContext?.completeRequest(returningItems: [], completionHandler: nil)
    }
}
