import SwiftUI

@main
struct PeacockApp: App {
    @StateObject private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(appState)
                .environmentObject(appState.locale)
                .preferredColorScheme(appState.theme.colorScheme)
                .task { appState.start() }
                .onDisappear { appState.stop() }
                .onOpenURL { url in
                    // peacock://share — the Share Extension dropped files in the App Group inbox
                    if url.scheme == ShareInbox.urlScheme, url.host == ShareInbox.shareHost {
                        appState.loadSharedInbox()
                    }
                }
        }
    }
}
