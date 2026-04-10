import SwiftUI

struct ContentView: View {
    @EnvironmentObject var appState: AppState

    enum Tab {
        case devices, snippets, settings
    }

    @State private var selectedTab: Tab = .devices

    private var t: (String) -> String { appState.locale.t }

    var body: some View {
        TabView(selection: $selectedTab) {
            NavigationStack {
                DeviceListView()
            }
            .tabItem {
                Image(systemName: "wifi")
                Text(t("tab.devices"))
            }
            .tag(Tab.devices)
            .badge(appState.totalUnread)

            NavigationStack {
                SnippetListView()
            }
            .tabItem {
                Image(systemName: "doc.text")
                Text(t("tab.snippets"))
            }
            .tag(Tab.snippets)

            NavigationStack {
                SettingsView()
            }
            .tabItem {
                Image(systemName: "gearshape")
                Text(t("tab.settings"))
            }
            .tag(Tab.settings)
        }
        .tint(Color.peacockTeal)
    }
}
