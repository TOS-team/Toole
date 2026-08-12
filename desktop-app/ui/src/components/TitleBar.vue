<script setup lang="ts">
// barre de titre de la fenêtre (Windows/Linux) : je détecte macOS pour
// réserver l'espace des boutons rouges natifs et je pilote min/max/close
import { ref, onMounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

const isMac = ref(false);
const isMaximized = ref(false);

// je détecte la plateforme via l'API moderne, avec repli sur navigator
function detectPlatform() {
  const n = navigator as Navigator & { userAgentData?: { platform?: string } };
  const platform = n.userAgentData?.platform ?? navigator.platform ?? "";
  return /Mac|iP(hone|ad|od)/.test(platform);
}

const appWindow = getCurrentWindow();

async function refreshMaximized() {
  try {
    isMaximized.value = await appWindow.isMaximized();
  } catch {
    /* fenêtre indisponible : je garde l'état courant */
  }
}

async function toggleMaximize() {
  await appWindow.toggleMaximize();
  await refreshMaximized();
}

function minimize() {
  appWindow.minimize();
}

function close() {
  appWindow.close();
}

onMounted(() => {
  isMac.value = detectPlatform();
  refreshMaximized();
  appWindow.onResized(refreshMaximized);
});
</script>

<template>
  <div
    data-tauri-drag-region
    class="h-8 flex-shrink-0 flex items-center select-none relative bg-background"
    :class="isMac ? 'justify-start pl-[80px]' : 'justify-end'"
  >
      <div v-if="!isMac" class="flex items-center h-full">
      <button
        type="button"
        title="Réduire"
        aria-label="Réduire"
        class="w-11 h-full flex items-center justify-center text-on-surface-variant hover:text-on-surface hover:bg-surface-variant cursor-pointer transition-colors"
        @click="minimize"
      >
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
          <path d="M0 5h10" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
      <button
        type="button"
        :title="isMaximized ? 'Restaurer' : 'Agrandir'"
        :aria-label="isMaximized ? 'Restaurer' : 'Agrandir'"
        class="w-11 h-full flex items-center justify-center text-on-surface-variant hover:text-on-surface hover:bg-surface-variant cursor-pointer transition-colors"
        @click="toggleMaximize"
      >
        <svg
          v-if="isMaximized"
          width="11" height="11" viewBox="0 0 12 12" fill="none"
        >
          <rect x="1.8" y="3.6" width="6.6" height="6.6" rx="1.1" fill="none" stroke="currentColor" stroke-width="1.2" />
          <path
            d="M4.2 3.6V2.7A1.2 1.2 0 0 1 5.4 1.5h3.9a1.2 1.2 0 0 1 1.2 1.2v3.9a1.2 1.2 0 0 1-1.2 1.2h-.9"
            fill="none" stroke="currentColor" stroke-width="1.2"
          />
        </svg>
        <svg v-else width="11" height="11" viewBox="0 0 12 12" fill="none">
          <rect x="1.5" y="1.5" width="9" height="9" rx="1.2" fill="none" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
      <button
        type="button"
        title="Fermer"
        aria-label="Fermer"
        class="w-11 h-full flex items-center justify-center text-on-surface-variant hover:text-white hover:bg-error cursor-pointer transition-colors"
        @click="close"
      >
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
          <path d="M0 0l10 10M10 0L0 10" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
    </div>
  </div>
</template>