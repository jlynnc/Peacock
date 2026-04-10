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
        }
    }
}
