<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "../tauri";
import { useFilesStore } from "../stores/files";
import type { FileEntry } from "../types";
import { formatSize } from "../utils";

const filesStore = useFilesStore();
const isDragOver = ref(false);
const dropHint = ref(true);

async function pickFiles() {
  try {
    const selected = await open({
      multiple: true,
      title: "Choisir des fichiers à envoyer",
    });
    if (!selected) return;
    const entries: FileEntry[] = selected.map((p) => ({
      path: p,
      name: p.split("/").pop() || p.split("\\").pop() || p,
    }));
    filesStore.addFiles(entries);
    dropHint.value = false;
  } catch (e) {
    console.error("pick_files error:", e);
  }
}

function onDragEnter(e: DragEvent) {
  e.preventDefault();
  isDragOver.value = true;
}

function onDragOver(e: DragEvent) {
  e.preventDefault();
  if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
  isDragOver.value = true;
}

function onDragLeave() {
  isDragOver.value = false;
}

// extrait des chemins de fichiers depuis du texte (file:// ou /path)
function extractPathsFromText(text: string): FileEntry[] {
  const entries: FileEntry[] = [];
  for (const line of text.split("\n")) {
    let p = line.trim();
    if (!p) continue;
    if (p.startsWith("file://")) p = p.slice(7);
    if (p.startsWith("/")) {
      entries.push({
        path: p,
        name: p.split("/").pop() || p.split("\\").pop() || p,
      });
    }
  }
  return entries;
}

// fallback Rust pour Ctrl+V (lit le presse-papier systeme)
async function onKeydown(e: KeyboardEvent) {
  if (!(e.ctrlKey || e.metaKey) || e.key !== "v") return;
  if ((e.target as HTMLElement)?.tagName === "INPUT") return;
  e.preventDefault();
  try {
    const text = await invoke<string>("read_clipboard");
    if (!text) return;
    const entries = extractPathsFromText(text);
    if (!entries.length) return;
    filesStore.addFiles(entries);
    dropHint.value = false;
  } catch (err) {
    console.error("clipboard read error:", err);
  }
}

// coller des fichiers depuis le presse-papier (HTML5)
function onPaste(e: ClipboardEvent) {
  const dt = e.clipboardData;
  if (!dt) return;
  let text = dt.getData("text/plain") || dt.getData("text/uri-list") || "";
  if (!text) {
    // fallback File objects
    if (dt.files.length) {
      const entries: FileEntry[] = [];
      for (const f of Array.from(dt.files)) {
        entries.push({ path: (f as any).path || f.name, name: f.name });
      }
      if (!entries.length) return;
      e.preventDefault();
      filesStore.addFiles(entries);
      dropHint.value = false;
    }
    return;
  }
  const entries = extractPathsFromText(text);
  if (!entries.length) return;
  e.preventDefault();
  filesStore.addFiles(entries);
  dropHint.value = false;
}

// ecoute l'evenement Tauri pour les fichiers glisses-deposes
let unlisten: (() => void) | null = null;

onMounted(async () => {
  document.addEventListener("paste", onPaste);
  document.addEventListener("keydown", onKeydown);

  unlisten = await listen<string[]>("dropped-files", (event) => {
    const paths = event.payload;
    if (!paths || !paths.length) return;
    const entries: FileEntry[] = paths.map((p) => ({
      path: p,
      name: p.split("/").pop() || p.split("\\").pop() || p,
    }));
    filesStore.addFiles(entries);
    dropHint.value = false;
  });
});

onUnmounted(() => {
  document.removeEventListener("paste", onPaste);
  document.removeEventListener("keydown", onKeydown);
  if (unlisten) unlisten();
});
</script>

<template>
  <div
    class="panel flex-[0.45] flex flex-col relative min-h-[72px] rounded-xl p-3.5 transition-colors duration-150"
    :class="{
      '!border-red-brand !bg-red-brand/8 !shadow-[0_0_0_3px_rgba(232,40,43,0.14),0_4px_24px_rgba(0,0,0,0.4)]': isDragOver,
    }"
    @dragenter="onDragEnter"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
  >
    <div class="flex items-center mb-1.5">
      <span class="panel-label text-white/60">Fichiers à envoyer</span>
    </div>
    <div
      v-if="dropHint && !filesStore.files.length"
      class="flex-1 flex flex-col items-center justify-center gap-2 text-muted text-[13px] font-light pointer-events-none"
    >
      <div
        class="w-14 h-14 border-2 border-red-brand/40 rounded-xl flex items-center justify-center"
      >
        <div
          class="w-7 h-7 bg-red-brand"
          style="mask-image: url(/assets/icons/drag-drop.png); mask-size: contain; mask-repeat: no-repeat; -webkit-mask-image: url(/assets/icons/drag-drop.png); -webkit-mask-size: contain; -webkit-mask-repeat: no-repeat;"
        ></div>
      </div>
      <div class="text-white/60 text-[13px] font-light">
        Déposez vos fichiers ici
      </div>
    </div>

    <ul
      v-if="filesStore.files.length"
      class="list-none flex flex-col gap-1.5 overflow-y-auto flex-1"
    >
      <li
        v-for="f in filesStore.files"
        :key="f.path"
        class="flex items-center gap-2.5 px-2.5 py-2 text-[13px] font-medium
               bg-white/4 border border-white/7 rounded-lg
               transition-colors duration-150 hover:bg-white/7"
      >
        <span class="flex-1 truncate">{{ f.name }}</span>
        <span v-if="f.size != null" class="text-[11px] text-muted shrink-0">{{ formatSize(f.size) }}</span>
        <button
          type="button"
          aria-label="Retirer"
          title="Retirer ce fichier"
          class="bg-none border-none text-muted cursor-pointer text-[15px] leading-none
                 px-1 py-0.5 rounded transition-colors duration-150 hover:text-red-brand hover:bg-red-soft"
          @click="filesStore.removeFile(f.path); dropHint = !filesStore.files.length"
        >
          &times;
        </button>
      </li>
    </ul>

    <button
      type="button"
      aria-label="Ajouter des fichiers"
      title="Ajouter des fichiers"
      class="absolute bottom-3 right-3 w-8 h-8 rounded-full
             border border-red-brand/40 bg-red-soft
             backdrop-blur-md text-white text-lg
             cursor-pointer flex items-center justify-center
             shadow-[0_4px_18px_rgba(232,40,43,0.25),inset_0_1px_0_rgba(255,255,255,0.14)]
             transition-all duration-200 hover:bg-red-brand/26 hover:scale-105 active:scale-95"
      @click="pickFiles"
    >
      +
    </button>
  </div>
</template>
