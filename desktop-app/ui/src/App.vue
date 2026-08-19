<script setup lang="ts">
// composant racine : il orchestre les pages (accueil, transferts, historique,
// paramètres), lance la découverte des appareils au montage et envoie les
// fichiers vers les appareils sélectionnés.
import { ref, computed, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "./tauri";
import { usePeersStore } from "./stores/peers";
import { useFilesStore } from "./stores/files";
import { useTransfersStore } from "./stores/transfers";
import { useUpdaterStore } from "./stores/updater";
import { useFirewallStore } from "./stores/firewall";
import Icon from "./components/Icon.vue";
import SidebarNav from "./components/SidebarNav.vue";
import PeerList from "./components/PeerList.vue";
import FirewallBanner from "./components/FirewallBanner.vue";
import AboutModal from "./components/AboutModal.vue";
import IncomingTransferModal from "./components/IncomingTransferModal.vue";
import HistoryPage from "./components/HistoryPage.vue";
import TransferPage from "./components/TransferPage.vue";
import HomePage from "./components/HomePage.vue";
import SettingsPage from "./components/SettingsPage.vue";
import TitleBar from "./components/TitleBar.vue";

const transfersStore = useTransfersStore();
const deviceId = ref("");
const peersStore = usePeersStore();
const filesStore = useFilesStore();
const aboutModal = ref<InstanceType<typeof AboutModal> | null>(null);
const view = ref("home");

// je n'autorise l'envoi que si j'ai des fichiers et au moins un appareil coché
const canSend = computed(
  () => filesStore.files.length > 0 && peersStore.selectedIds.size > 0,
);

const sendError = ref("");
// je ne garde que les transferts que J'AI envoyés : une erreur sur un
// transfert reçu doit rester sur sa carte (gérée par le store des transferts),
// pas s'afficher sous le bouton Transférer
const sentTransferIds = new Set<string>();
// je précise au survol pourquoi le bouton d'envoi est désactivé
const buttonTitle = computed(() => {
  if (canSend.value) return undefined;
  if (filesStore.files.length === 0) return "Ajoutez des fichiers d'abord";
  if (peersStore.selectedIds.size === 0) return "Sélectionnez un appareil";
  return undefined;
});

// j'envoie la liste des fichiers vers chaque appareil coché, puis je bascule
// sur la page des transferts pour suivre la progression
async function sendFiles() {
  sendError.value = "";
  if (!canSend.value) return;

  const paths = filesStore.files.map((f) => f.path);
  const names = filesStore.files.map((f) => f.name);

  for (const peer of peersStore.peers) {
    if (peersStore.selectedIds.has(peer.id)) {
      const peerAddr = `${peer.addr}:58200`;
      try {
        const transferId = await invoke<string>("send_files", {
          paths,
          peerAddr,
          peerId: peer.id,
        });
        sentTransferIds.add(transferId);
        transfersStore.upsert(transferId, { peer: peer.id, files: names });
      } catch (e) {
        sendError.value = `Envoi vers ${peer.id} : ${e}`;
        console.error("Erreur envoi vers", peer.id, ":", e);
      }
    }
  }
  view.value = "transfers";
}

// au montage je récupère l'identité de la machine, je démarre la découverte
// des appareils et je m'abonne aux événements du processus Rust
onMounted(async () => {
  try {
    deviceId.value = await invoke<string>("get_device_id");
  } catch (e) {
    console.error("get_device_id error:", e);
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
    await listen<{ transfer_id: string; error: string }>("tool://transfer/error", (event) => {
      if (!sentTransferIds.has(event.payload.transfer_id)) return;
      sendError.value = event.payload.error;
    });
  } catch (e) {
    console.error("transfer error listener:", e);
  }
  try {
    await transfersStore.startListening();
  } catch (e) {
    console.error("startListening error:", e);
  }
  // quand un appareil m'envoie des fichiers, je bascule sur la page des
  // transferts pour afficher la demande (boutons accepter / refuser)
  try {
    await listen("tool://transfer/incoming", () => {
      view.value = "transfers";
    });
  } catch (e) {
    console.error("incoming listener error:", e);
  }
  // je vérifie silencieusement les mises à jour au lancement : si une
  // nouvelle version existe, l'utilisateur la verra dans Paramètres
  useUpdaterStore().checkForUpdate(false).catch(console.error);
  // je détecte un pare-feu qui bloquerait les ports UDP (bannière si besoin)
  useFirewallStore().check();
});

// je coupe le polling et la découverte quand je quitte
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

    <SidebarNav :active="view" @navigate="view = $event" />

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
          <HomePage :hostname="deviceId" />
        </div>
      </main>

      <aside
        class="w-[240px] md:w-[280px] xl:w-[300px] flex flex-col rounded-2xl border border-outline-variant bg-surface-container/80 active-shadow relative z-10 flex-shrink-0 min-h-0 overflow-hidden"
      >
        <div class="flex-1 flex flex-col min-h-0 pt-5 md:pt-9 overflow-y-auto w-full">
          <FirewallBanner class="mx-6 mb-3" />
          <PeerList class="flex-1 min-h-0 w-full" />
        </div>

        <div class="p-4 md:p-6 pt-3 pb-4 mt-auto flex-shrink-0">
          <button
            type="button"
            :title="buttonTitle"
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
          <p
            v-if="sendError"
            class="mt-2 text-[11px] text-error text-center break-words"
          >
            {{ sendError }}
          </p>
        </div>
      </aside>
    </div>
  </div>

  <AboutModal ref="aboutModal" />
  <IncomingTransferModal />
</template>