<script setup lang="ts">
// zone de dépôt : sélection de fichiers par boîte de dialogue, glisser-déposer
// (drag & drop) ou collage du presse-papier. Je récupère les chemins locaux et
// je les pousse dans le store des fichiers.
import { ref, onMounted, onUnmounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { invoke } from "../tauri";
import { useFilesStore } from "../stores/files";
import type { FileEntry } from "../types";
import { formatSize, fileVisual } from "../utils";
import { convertFileSrc } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";

const filesStore = useFilesStore();
const isDragOver = ref(false);
const dropHint = ref(true);

// je propose la boîte de dialogue système pour choisir un ou plusieurs fichiers
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

// la boîte de dialogue système ne mélange pas fichiers et dossiers : j'ouvre
// un second sélecteur en mode répertoire pour pouvoir envoyer des dossiers
async function pickFolder() {
  try {
    const selected = await open({
      directory: true,
      title: "Choisir un dossier à envoyer",
    });
    const p = Array.isArray(selected) ? selected[0] : selected;
    if (!p) return;
    const name = p.split("/").pop() || p.split("\\").pop() || p;
    filesStore.addFiles([{ path: p, name }]);
    dropHint.value = false;
  } catch (e) {
    console.error("pick_folder error:", e);
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

// ecoute le drag-and-drop natif Tauri (v2) pour recup les chemins des fichiers
let unlisten: (() => void) | null = null;

onMounted(async () => {
  document.addEventListener("paste", onPaste);
  document.addEventListener("keydown", onKeydown);

  unlisten = await getCurrentWebview().onDragDropEvent((event) => {
    const { payload } = event;
    if (payload.type === "enter" || payload.type === "over") {
      isDragOver.value = true;
    } else if (payload.type === "leave") {
      isDragOver.value = false;
    } else if (payload.type === "drop") {
      isDragOver.value = false;
      const paths = payload.paths || [];
      if (!paths.length) return;
      const entries: FileEntry[] = paths.map((p) => ({
        path: p,
        name: p.split("/").pop() || p.split("\\").pop() || p,
      }));
      filesStore.addFiles(entries);
      dropHint.value = false;
    }
  });
});

// je retire mes écouteurs quand le composant disparaît
onUnmounted(() => {
  document.removeEventListener("paste", onPaste);
  document.removeEventListener("keydown", onKeydown);
  if (unlisten) unlisten();
});
</script>

<template>
  <div
    class="flex flex-col glass-panel rounded-2xl border border-dashed border-outline-variant hover:border-primary/50 transition-all duration-300 cursor-pointer group relative overflow-hidden bg-surface/30"
    :class="[
      'p-5 md:p-8',
      { '!border-primary !bg-primary/10': isDragOver },
    ]"
    @click="pickFiles"
    @dragenter="onDragEnter"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
  >
    <div
      class="absolute inset-0 bg-gradient-to-b from-primary/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"
    ></div>

    <div
      v-if="filesStore.files.length"
      class="relative z-10 flex items-center justify-between mb-3"
    >
      <span class="text-label-md font-label-md text-on-surface-variant">
        {{ filesStore.files.length }} {{ filesStore.files.length > 1 ? "fichiers" : "fichier" }}
      </span>
      <div class="flex items-center gap-3">
        <button
          type="button"
          aria-label="Ajouter un dossier"
          title="Choisir un dossier"
          class="inline-flex items-center gap-1.5 text-on-surface-variant hover:text-primary transition-colors text-label-md font-label-md cursor-pointer"
          @click.stop="pickFolder"
        >
          <Icon name="folder" :size="18" />
          Dossier
        </button>
        <button
          type="button"
          aria-label="Vider la liste"
          title="Retirer tous les fichiers"
          class="inline-flex items-center gap-1.5 text-on-surface-variant hover:text-error transition-colors text-label-md font-label-md cursor-pointer"
          @click.stop="filesStore.clearFiles()"
        >
          <Icon name="delete" :size="18" />
          Vider
        </button>
      </div>
    </div>

    <ul
      v-if="filesStore.files.length"
      class="relative z-10 w-full flex-1 min-h-0 flex flex-col gap-2 overflow-y-auto pr-1 mb-5"
    >
      <li
        v-for="f in filesStore.files"
        :key="f.path"
        class="flex items-center gap-2.5 px-3 py-2 text-[13px] bg-surface-container-high border border-outline rounded-lg"
      >
        <span v-if="f.isDir" class="shrink-0 w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
          <Icon name="folder" :size="20" class="text-primary" />
        </span>
        <span v-else-if="fileVisual(f.name).thumb" class="shrink-0 w-8 h-8 rounded-lg overflow-hidden bg-surface border border-outline flex items-center justify-center">
          <img
            :src="convertFileSrc(f.path)"
            :alt="f.name"
            class="w-full h-full object-cover"
            loading="lazy"
            draggable="false"
          />
        </span>
        <span v-else class="shrink-0 w-8 h-8 rounded-lg bg-surface/60 flex items-center justify-center">
          <Icon :name="fileVisual(f.name).icon" :size="20" class="text-on-surface-variant" />
        </span>
        <span class="flex-1 truncate text-on-surface">{{ f.name }}</span>
        <span v-if="!f.isDir && f.size != null" class="text-[11px] text-on-surface-variant shrink-0">
          {{ formatSize(f.size) }}
        </span>
        <button
          type="button"
          aria-label="Retirer"
          title="Retirer ce fichier"
          class="text-on-surface-variant hover:text-error transition-colors cursor-pointer"
          @click.stop="filesStore.removeFile(f.path)"
        >
          <Icon name="close" :size="16" />
        </button>
      </li>
    </ul>

    <div
      v-else
      class="relative z-10 flex-1 flex flex-col items-center justify-center text-center"
    >
      <div
        class="h-20 w-20 rounded-2xl bg-surface-container-high border border-outline flex items-center justify-center mb-6 group-hover:scale-105 group-hover:border-primary/30 group-hover:shadow-[0_0_15px_color:color-mix(in_srgb,var(--color-primary)_20%,transparent)] transition-all duration-300 relative"
      >
        <Icon name="upload-file" :size="32" class="text-on-surface group-hover:text-primary transition-colors" />
        <div
          class="absolute -bottom-2 -right-2 h-7 w-7 bg-primary rounded-full flex items-center justify-center shadow-lg group-hover:scale-110 transition-transform"
        >
          <Icon name="add" :size="16" class="text-on-primary" />
        </div>
      </div>
      <h2 class="text-headline-md font-headline-md text-on-background mb-3">
        Déposer des fichiers ici
      </h2>
      <p class="text-body-md font-body-md text-on-surface-variant max-w-sm">
        ou parcourir pour sélectionner.
        <span class="text-label-sm text-primary/80 mt-2 block">Chiffrement de bout en bout activé</span>
      </p>
      <button
        type="button"
        class="mt-5 inline-flex items-center gap-2 text-label-md font-label-md text-primary hover:text-primary/80 transition-colors cursor-pointer"
        @click.stop="pickFolder"
      >
        <Icon name="folder" :size="16" />
        Choisir un dossier
      </button>
    </div>
  </div>
</template>