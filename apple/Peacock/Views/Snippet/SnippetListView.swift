import SwiftUI

struct SnippetListView: View {
    @EnvironmentObject var appState: AppState
    @State private var searchText = ""
    @State private var renamingId: String?
    @State private var renameText = ""
    @State private var shareSnippetId: String?
    @FocusState private var renameFieldFocused: Bool

    private var t: (String) -> String { appState.locale.t }

    var filteredSnippets: [Snippet] {
        if searchText.isEmpty { return appState.snippets }
        return appState.snippets.filter {
            $0.title.localizedCaseInsensitiveContains(searchText) ||
            $0.content.localizedCaseInsensitiveContains(searchText)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Text(t("tab.snippets"))
                    .font(.system(size: 28, weight: .bold))
                Spacer()
                Button {
                    appState.createSnippet()
                    if let newId = appState.selectedSnippetId {
                        renamingId = newId
                        renameText = "新建片段"
                    }
                } label: {
                    Image(systemName: "plus")
                        .font(.system(size: 20))
                        .foregroundStyle(Color.peacockTeal)
                }
            }
            .padding(.horizontal, 16)
            .padding(.top, 8)
            .padding(.bottom, 12)

            // Search
            HStack {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                TextField(t("snippets.search"), text: $searchText)
                    .font(.system(size: 16))
                if !searchText.isEmpty {
                    Button { searchText = "" } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundStyle(.tertiary)
                    }
                }
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 8)
            .background(Color(.tertiarySystemFill), in: RoundedRectangle(cornerRadius: 10))
            .padding(.horizontal, 16)
            .padding(.bottom, 8)

        List {
            if filteredSnippets.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "doc.text")
                        .font(.system(size: 40))
                        .foregroundStyle(.tertiary)
                    Text(t("snippets.empty"))
                        .font(.headline)
                        .foregroundStyle(.secondary)
                    Text(t("snippets.empty.hint"))
                        .font(.subheadline)
                        .foregroundStyle(.tertiary)
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 60)
                .listRowBackground(Color.clear)
                .listRowSeparator(.hidden)
            } else {
                ForEach(filteredSnippets) { snippet in
                    if renamingId == snippet.id {
                        // Inline rename
                        TextField("片段名称", text: $renameText, onCommit: {
                            let id = snippet.id
                            let name = renameText
                            // Defer to avoid mutating state during view update
                            DispatchQueue.main.async {
                                appState.renameSnippet(id, title: name)
                                renamingId = nil
                            }
                        })
                        .font(.system(size: 15, weight: .semibold))
                        .textFieldStyle(.roundedBorder)
                        .focused($renameFieldFocused)
                        .onAppear {
                            renameText = snippet.title
                            DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                                renameFieldFocused = true
                                // Select all text in the focused UITextField
                                DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                                    UIApplication.shared.sendAction(#selector(UIResponder.selectAll(_:)), to: nil, from: nil, for: nil)
                                }
                            }
                        }
                    } else {
                        NavigationLink(value: snippet.id) {
                            SnippetRowView(snippet: snippet)
                        }
                        .swipeActions(edge: .trailing, allowsFullSwipe: false) {
                            Button(role: .destructive) {
                                appState.deleteSnippet(snippet.id)
                            } label: {
                                Label(t("snippets.delete"), systemImage: "trash")
                            }

                            Button {
                                appState.pinSnippetToTop(snippet.id)
                            } label: {
                                Label(t("snippets.pin"), systemImage: "pin")
                            }
                            .tint(.orange)

                            Button {
                                shareSnippetId = snippet.id
                            } label: {
                                Label(t("snippets.share"), systemImage: "square.and.arrow.up")
                            }
                            .tint(Color.peacockTeal)

                            Button {
                                renamingId = snippet.id
                                renameText = snippet.title
                            } label: {
                                Label(t("snippets.rename"), systemImage: "pencil")
                            }
                            .tint(.blue)
                        }
                    }
                }
            }
        }
        .listStyle(.plain)
        }
        .navigationBarHidden(true)
        .navigationDestination(for: String.self) { snippetId in
            SnippetEditorView(snippetId: snippetId)
        }
        .sheet(item: $shareSnippetId) { snippetId in
            DevicePickerSheet(snippetId: snippetId)
                .environmentObject(appState)
        }
    }
}

// Make String work with sheet(item:)
extension String: @retroactive Identifiable {
    public var id: String { self }
}

struct SnippetRowView: View {
    let snippet: Snippet

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text(snippet.title)
                    .font(.system(size: 15, weight: .semibold))
                    .lineLimit(1)
                Spacer()
                Text(FormatUtils.relativeTime(snippet.date))
                    .font(.system(size: 12))
                    .foregroundStyle(.tertiary)
            }
            if !snippet.content.isEmpty {
                Text(snippet.content.replacingOccurrences(of: "[[", with: "").replacingOccurrences(of: "]]", with: "").prefix(60))
                    .font(.system(size: 13))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
        .padding(.vertical, 4)
    }
}
