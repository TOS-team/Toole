import { defineStore } from "pinia";
import { ref } from "vue";
import type { FileEntry } from "../types";
import { invoke } from "../tauri";

export const useFilesStore = defineStore("files", () => {
  const files = ref<FileEntry[]>([]);

  function addFiles(entries: FileEntry[]) {
    const existing = new Set(files.value.map((f) => f.path));
    const newEntries: FileEntry[] = [];
    for (const e of entries) {
      if (!e.path || existing.has(e.path)) continue;
      existing.add(e.path);
      files.value.push(e);
      newEntries.push(e);
    }
    if (newEntries.length) fetchSizes();
  }

  async function fetchSizes() {
    const paths = files.value.map((f) => f.path);
    if (!paths.length) return;
    try {
      const sizes = await invoke<number[]>("get_file_sizes", { paths });
      for (let i = 0; i < files.value.length; i++) {
        files.value[i].size = sizes[i];
      }
    } catch (e) {
      console.error("get_file_sizes error:", e);
    }
  }

  function removeFile(path: string) {
    files.value = files.value.filter((f) => f.path !== path);
  }

  function clearFiles() {
    files.value = [];
  }

  return { files, addFiles, removeFile, clearFiles, fetchSizes };
});
