import SwiftUI

struct SnippetListView: View {
    @EnvironmentObject var appState: AppState
    @State private var searchText = ""
    @State private var renamingId: String?
    @State private var renameText = ""
    @State private var shareSnippetId: String?

    private var t: (String) -> String { appState.locale.t }
    @State private var navigationPath = NavigationPath()

    var filteredSnippets: [Snippet] {
        if searchText.isEmpty { return appState.snippets }
        return appState.snippets.filter {
            $0.title.localizedCaseInsensitiveContains(searchText) ||
            $0.content.localizedCaseInsensitiveContains(searchText)
        }
    }

    var body: some View {
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
                            appState.renameSnippet(snippet.id, title: renameText)
                            renamingId = nil
                        })
                        .font(.system(size: 15, weight: .semibold))
                        .textFieldStyle(.roundedBorder)
                        .onAppear {
                            renameText = snippet.title
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
        .searchable(text: $searchText, prompt: t("snippets.search"))
        .navigationTitle(t("tab.snippets"))
        .navigationDestination(for: String.self) { snippetId in
            SnippetEditorView(snippetId: snippetId)
        }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    appState.createSnippet()
                    // Auto-enter rename mode for the new snippet
                    if let newId = appState.selectedSnippetId {
                        renamingId = newId
                        renameText = "新建片段"
                    }
                } label: {
                    Image(systemName: "plus")
                }
            }
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
