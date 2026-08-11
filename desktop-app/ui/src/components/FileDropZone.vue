<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { invoke } from "../tauri";
import { useFilesStore } from "../stores/files";
import type { FileEntry } from "../types";
import { formatSize } from "../utils";
import Icon from "./Icon.vue";

const props = defineProps<{ compact?: boolean }>();

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
      props.compact ? 'p-3' : 'p-5 md:p-8',
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

    <ul
      v-if="filesStore.files.length"
      class="relative z-10 w-full flex flex-col gap-2 overflow-y-auto pr-1 mb-5 max-h-[150px]"
    >
      <li
        v-for="f in filesStore.files"
        :key="f.path"
        class="flex items-center gap-2.5 px-3 py-2 text-[13px] bg-surface-container-high border border-outline rounded-lg"
      >
        <span class="flex-1 truncate text-on-surface">{{ f.name }}</span>
        <span v-if="f.size != null" class="text-[11px] text-on-surface-variant shrink-0">
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
        v-if="!props.compact"
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
        {{ props.compact ? "Fichiers à envoyer" : "Déposer des fichiers ici" }}
      </h2>
      <p class="text-body-md font-body-md text-on-surface-variant max-w-sm">
        {{ props.compact ? "Cliquez pour ajouter" : "ou parcourir pour sélectionner." }}
        <span v-if="!props.compact" class="text-label-sm text-primary/80 mt-2 block">Chiffrement de bout en bout activé</span>
      </p>
    </div>
  </div>
</template>