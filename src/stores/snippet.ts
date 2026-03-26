import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { type UnlistenFn } from "@tauri-apps/api/event";
import type { Snippet } from "@/types/snippet";
import {
  getSnippets as ipcGetSnippets,
  createSnippet as ipcCreateSnippet,
  updateSnippet as ipcUpdateSnippet,
  deleteSnippet as ipcDeleteSnippet,
  copySnippetCount as ipcCopySnippetCount,
  shareSnippet as ipcShareSnippet,
} from "@/utils/ipc";
import { isTauri } from "@/utils/platform";
import { i18n } from "@/i18n";

export const useSnippetStore = defineStore("snippet", () => {
  const snippets = ref<Snippet[]>([]);
  const selectedId = ref<string | null>(null);
  const searchQuery = ref<string>("");
  /** Set to the id of a newly created snippet so the list can focus its title */
  const renamingId = ref<string | null>(null);

  let unlisteners: UnlistenFn[] = [];

  const filteredSnippets = computed(() => {
    if (!searchQuery.value) return snippets.value;
    const q = searchQuery.value.toLowerCase();
    return snippets.value.filter(
      (s) =>
        s.title.toLowerCase().includes(q) ||
        s.content.toLowerCase().includes(q) ||
        s.note.toLowerCase().includes(q),
    );
  });

  const selectedSnippet = computed(() => {
    if (!selectedId.value) return null;
    return snippets.value.find((s) => s.id === selectedId.value) || null;
  });

  async function loadSnippets() {
    if (!isTauri()) return;
    try {
      const data = await ipcGetSnippets();
      snippets.value = data as Snippet[];
    } catch (e) {
      console.error("Failed to load snippets:", e);
    }
  }

  async function createNew() {
    const t = i18n.global.t;
    const id = await ipcCreateSnippet(t('snippet.newSnippet'), "", "", "");
    const now = Math.floor(Date.now() / 1000);
    snippets.value.unshift({
      id,
      title: t('snippet.newSnippet'),
      content: "",
      tag: "",
      note: "",
      copy_count: 0,
      created_at: now,
      updated_at: now,
    });
    selectedId.value = id;
    renamingId.value = id;
  }

  async function saveSnippet(
    id: string,
    fields: Partial<Pick<Snippet, "title" | "content" | "note">>,
  ) {
    const s = snippets.value.find((s) => s.id === id);
    if (!s) return;
    const title = fields.title ?? s.title;
    const content = fields.content ?? s.content;
    const note = fields.note ?? s.note;
    await ipcUpdateSnippet(id, title, content, s.tag, note);
    s.title = title;
    s.content = content;
    s.note = note;
    s.updated_at = Math.floor(Date.now() / 1000);
  }

  async function removeSnippet(id: string) {
    await ipcDeleteSnippet(id);
    snippets.value = snippets.value.filter((s) => s.id !== id);
    if (selectedId.value === id) {
      selectedId.value =
        snippets.value.length > 0 ? snippets.value[0].id : null;
    }
  }

  async function incrementCopyCount(id: string) {
    await ipcCopySnippetCount(id);
    const s = snippets.value.find((s) => s.id === id);
    if (s) s.copy_count++;
  }

  async function shareToDevice(deviceId: string, snippet: Snippet) {
    await ipcShareSnippet(
      deviceId,
      snippet.title,
      snippet.content,
      snippet.tag,
      snippet.note,
    );
  }

  /** Accept a shared snippet offer — create it locally */
  async function createFromShare(title: string, content: string, tag: string, note: string) {
    if (!isTauri()) return;
    const id = await ipcCreateSnippet(title, content, tag, note);
    await loadSnippets();
    selectedId.value = id;
  }

  async function startListening() {
    if (!isTauri()) return;
    // snippet-offer is handled by chat store (shows in chat as a card)
    // No auto-save here anymore
    unlisteners = [];
  }

  function stopListening() {
    for (const u of unlisteners) u();
    unlisteners = [];
  }

  return {
    snippets,
    selectedId,
    searchQuery,
    renamingId,
    filteredSnippets,
    selectedSnippet,
    loadSnippets,
    createNew,
    saveSnippet,
    removeSnippet,
    incrementCopyCount,
    shareToDevice,
    createFromShare,
    startListening,
    stopListening,
  };
});
