<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "./tauri";
import { usePeersStore } from "./stores/peers";
import { useFilesStore } from "./stores/files";
import { useTransfersStore } from "./stores/transfers";
import Icon from "./components/Icon.vue";
import SidebarNav from "./components/SidebarNav.vue";
import PeerList from "./components/PeerList.vue";
import AboutModal from "./components/AboutModal.vue";
import HistoryPage from "./components/HistoryPage.vue";
import TransferPage from "./components/TransferPage.vue";
import HomePage from "./components/HomePage.vue";
import SettingsPage from "./components/SettingsPage.vue";
import TitleBar from "./components/TitleBar.vue";

const transfersStore = useTransfersStore();
const hostname = ref("");
const peersStore = usePeersStore();
const filesStore = useFilesStore();
const aboutModal = ref<InstanceType<typeof AboutModal> | null>(null);
const view = ref("home");

const canSend = computed(
  () => filesStore.files.length > 0 && peersStore.selectedHostnames.size > 0,
);

async function sendFiles() {
  if (!canSend.value) return;

  const paths = filesStore.files.map((f) => f.path);
  const names = filesStore.files.map((f) => f.name);

  for (const peer of peersStore.peers) {
    if (peersStore.selectedHostnames.has(peer.hostname)) {
      const peerAddr = `${peer.addr}:58200`;
      try {
        const transferId = await invoke<string>("send_files", {
          paths,
          peerAddr,
        });
        transfersStore.upsert(transferId, { peer: peer.hostname, files: names });
        console.log("Transfert démarré vers", peer.hostname, ":", transferId);
      } catch (e) {
        console.error("Erreur envoi vers", peer.hostname, ":", e);
      }
    }
  }
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
  try {
    await peersStore.startListening();
  } catch (e) {
    console.error("peers startListening error:", e);
  }
  try {
    await transfersStore.startListening();
  } catch (e) {
    console.error("startListening error:", e);
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
  <div
    class="w-full h-full flex flex-col overflow-hidden rounded-2xl bg-background relative"
  >
    <TitleBar />

    <div class="flex-1 min-h-0 flex flex-row gap-2 md:gap-3 p-2 md:p-3 relative">
    <div
      class="absolute inset-0 pointer-events-none"
      :style="{
        background: `radial-gradient(700px 480px at 20% 0%, rgba(var(--glow-color), calc(0.35 * var(--glow-opacity))), transparent 62%),
          radial-gradient(600px 420px at 85% 100%, rgba(var(--glow-color), calc(0.25 * var(--glow-opacity))), transparent 62%)`,
      }"
    ></div>

    <SidebarNav :hostname="hostname" :active="view" @navigate="view = $event" />

      <main
        class="flex-1 min-w-0 flex flex-col relative z-10 overflow-hidden rounded-2xl border border-outline-variant bg-surface-container-lowest active-shadow"
      >
        <div class="absolute inset-0 tech-grid opacity-[0.05] pointer-events-none"></div>

        <div v-if="view === 'history'" class="flex-1 min-h-0 flex flex-col">
          <HistoryPage />
        </div>

        <div v-else-if="view === 'transfers'" class="flex-1 min-h-0 flex flex-col">
          <TransferPage />
        </div>

        <div v-else-if="view === 'settings'" class="flex-1 min-h-0 flex flex-col">
          <SettingsPage @open-about="aboutModal?.open()" />
        </div>

        <div v-else class="flex-1 min-h-0 flex flex-col">
          <HomePage :hostname="hostname" />
        </div>
      </main>

      <aside
        class="w-[240px] md:w-[280px] xl:w-[300px] flex flex-col rounded-2xl border border-outline-variant bg-surface-container/80 active-shadow relative z-10 flex-shrink-0 min-h-0 overflow-hidden"
      >
        <div class="flex-1 flex flex-col min-h-0 pt-5 md:pt-9 overflow-y-auto w-full">
          <PeerList class="flex-1 min-h-0 w-full" />
        </div>

        <div class="p-4 md:p-6 pt-3 pb-4 mt-auto flex-shrink-0">
          <button
            type="button"
            class="w-full h-12 bg-surface-container-high border border-outline rounded-xl flex items-center justify-center gap-2 transition-colors"
            :class="
              canSend
                ? 'text-on-surface hover:bg-surface-variant cursor-pointer'
                : 'text-on-surface-variant opacity-50 cursor-not-allowed'
            "
            :disabled="!canSend"
            @click="sendFiles"
          >
            <Icon name="send" :size="18" />
            <span class="text-label-md font-label-md">Transférer</span>
          </button>
        </div>
      </aside>
    </div>
  </div>

  <AboutModal ref="aboutModal" />
</template>