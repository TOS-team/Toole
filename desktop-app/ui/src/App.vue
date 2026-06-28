<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invoke } from "./tauri";
import { usePeersStore } from "./stores/peers";
import { useFilesStore } from "./stores/files";
import WelcomeHeader from "./components/WelcomeHeader.vue";
import FileDropZone from "./components/FileDropZone.vue";
import PeerList from "./components/PeerList.vue";
import AboutModal from "./components/AboutModal.vue";

const hostname = ref("");
const peersStore = usePeersStore();
const filesStore = useFilesStore();
const aboutModal = ref<InstanceType<typeof AboutModal> | null>(null);

const canSend = computed(
  () => filesStore.files.length > 0 && peersStore.selectedHostnames.size > 0,
);

function sendFiles() {
  // TODO: implementer l'envoi QUIC
  console.log("Envoi pas encore implemente");
}

const appWindow = getCurrentWebviewWindow();

function closeApp() {
  appWindow.close().catch((err) => {
    console.error("window.close error:", err);
    invoke("close_window").catch(console.error);
  });
}

function minimizeApp() {
  appWindow.minimize().catch(console.error);
}

onMounted(async () => {
  try {
    hostname.value = await invoke<string>("get_hostname");
  } catch (e) {
    console.error("get_hostname error:", e);
  }
  try {
    await invoke("start_discovery");
    peersStore.startPolling();
  } catch (e) {
    console.error("start_discovery error:", e);
  }
});

onUnmounted(() => {
  peersStore.stopPolling();
  invoke("stop_discovery").catch(console.error);
});

window.addEventListener("beforeunload", () => {
  peersStore.stopPolling();
  invoke("stop_discovery").catch(console.error);
});
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="titlebar" data-tauri-drag-region="deep">
      <div class="flex items-center justify-self-start"></div>
      <span class="text-[13px] font-bold">Toolé</span>
      <div class="flex items-center justify-self-end">
        <button
          type="button"
          aria-label="Réduire"
          title="Réduire la fenêtre"
          class="titlebar-btn"
          data-tauri-no-drag
          @click="minimizeApp"
        >
          &minus;
        </button>
        <button
          type="button"
          aria-label="Fermer"
          title="Fermer l'application"
          class="titlebar-btn"
          data-tauri-no-drag
          @click="closeApp"
        >
          &times;
        </button>
      </div>
    </div>
    <div class="flex flex-col gap-2.5 p-4 flex-1 min-h-0 max-w-[600px] mx-auto w-full">
      <WelcomeHeader :hostname="hostname" @open-about="aboutModal?.open()" />
      <FileDropZone />
      <PeerList />
      <button
        type="button"
        class="w-full py-2.5 rounded-lg font-semibold text-sm flex items-center justify-center gap-2
               transition disabled:opacity-30 disabled:cursor-not-allowed
               enabled:bg-red-brand/90 enabled:hover:bg-red-brand enabled:text-white
               shadow-[0_4px_14px_rgba(232,40,43,0.3)]"
        :disabled="!canSend"
        @click="sendFiles"
      >
        <img src="/assets/icons/envoyer.png" alt="" class="w-4 h-4" style="filter: brightness(0) invert(1)" />
        Envoyer les fichiers
      </button>
    </div>
  </div>
  <AboutModal ref="aboutModal" />
</template>
