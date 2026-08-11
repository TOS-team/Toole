// je garde la liste des fichiers que l'utilisateur veut envoyer, avec leur
// taille récupérée côté Rust. Je dédoublonne par chemin et j'ignore les
// chemins vides.
import { defineStore } from "pinia";
import { ref } from "vue";
import type { FileEntry } from "../types";
import { invoke } from "../tauri";

export const useFilesStore = defineStore("files", () => {
  const files = ref<FileEntry[]>([]);

  // j'ajoute des entrées sans doublon, puis je recharge les tailles si la
  // liste a effectivement grandi
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

  // je demande à Rust la taille et le type (fichier/dossier) de chaque entrée
  async function fetchSizes() {
    const paths = files.value.map((f) => f.path);
    if (!paths.length) return;
    try {
      const infos = await invoke<{ size: number; is_dir: boolean }[]>(
        "get_file_infos",
        { paths },
      );
      for (let i = 0; i < files.value.length; i++) {
        files.value[i].size = infos[i].size;
        files.value[i].isDir = infos[i].is_dir;
      }
    } catch (e) {
      console.error("get_file_infos error:", e);
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
