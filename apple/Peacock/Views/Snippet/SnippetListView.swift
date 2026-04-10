import SwiftUI

struct SnippetListView: View {
    @EnvironmentObject var appState: AppState
    @State private var searchText = ""

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
                    Text("暂无片段")
                        .font(.headline)
                        .foregroundStyle(.secondary)
                    Text("点击右上角 + 创建新片段")
                        .font(.subheadline)
                        .foregroundStyle(.tertiary)
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 60)
                .listRowBackground(Color.clear)
                .listRowSeparator(.hidden)
            } else {
                ForEach(filteredSnippets) { snippet in
                    NavigationLink(value: snippet.id) {
                        SnippetRowView(snippet: snippet)
                    }
                    .swipeActions(edge: .trailing, allowsFullSwipe: false) {
                        Button(role: .destructive) {
                            appState.deleteSnippet(snippet.id)
                        } label: {
                            Label("删除", systemImage: "trash")
                        }
                    }
                }
            }
        }
        .listStyle(.plain)
        .searchable(text: $searchText, prompt: "搜索片段")
        .navigationTitle("片段")
        .navigationDestination(for: String.self) { snippetId in
            SnippetEditorView(snippetId: snippetId)
        }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    appState.createSnippet()
                } label: {
                    Image(systemName: "plus")
                }
            }
        }
    }
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
                Text(snippet.content.prefix(60))
                    .font(.system(size: 13))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
        .padding(.vertical, 4)
    }
}
