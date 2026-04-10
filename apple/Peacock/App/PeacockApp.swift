import SwiftUI

@main
struct PeacockApp: App {
    @StateObject private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(appState)
                .task { appState.start() }
                .onDisappear { appState.stop() }
        }
    }
}
