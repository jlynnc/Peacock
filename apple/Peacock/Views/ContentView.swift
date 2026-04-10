import SwiftUI

struct ContentView: View {
    @EnvironmentObject var appState: AppState

    enum Tab {
        case devices, snippets, settings
    }

    @State private var selectedTab: Tab = .devices

    var body: some View {
        TabView(selection: $selectedTab) {
            NavigationStack {
                DeviceListView()
            }
            .tabItem {
                Image(systemName: "wifi")
                Text("设备")
            }
            .tag(Tab.devices)
            .badge(appState.totalUnread)

            NavigationStack {
                SnippetListView()
            }
            .tabItem {
                Image(systemName: "doc.text")
                Text("片段")
            }
            .tag(Tab.snippets)

            NavigationStack {
                SettingsView()
            }
            .tabItem {
                Image(systemName: "gearshape")
                Text("设置")
            }
            .tag(Tab.settings)
        }
        .tint(Color.peacockTeal)
        .sheet(item: $appState.pendingFileOffers.first) { offer in
            FileOfferSheet(offer: offer)
                .environmentObject(appState)
        }
    }
}

// Extend Array element to be Identifiable for sheet binding
extension Array {
    var first: Element? {
        get { isEmpty ? nil : self[0] }
        set {
            if newValue == nil && !isEmpty {
                removeFirst()
            }
        }
    }
}
