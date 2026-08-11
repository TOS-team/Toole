<script setup lang="ts">
// barre de titre de la fenêtre (Windows/Linux) : je détecte macOS pour
// réserver l'espace des boutons rouges natifs et je pilote min/max/close
import { ref, onMounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

const isMac = ref(false);

onMounted(() => {
  // je détecte la plateforme via l'API moderne, avec repli sur navigator
  const n = navigator as Navigator & { userAgentData?: { platform?: string } };
  const platform = n.userAgentData?.platform ?? navigator.platform ?? "";
  isMac.value = /Mac|iP(hone|ad|od)/.test(platform);
});

const appWindow = getCurrentWindow();

function minimize() {
  appWindow.minimize();
}

function close() {
  appWindow.close();
}
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