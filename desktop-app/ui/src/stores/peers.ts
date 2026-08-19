// je gère la liste des appareils Toolé détectés et la sélection de ceux vers
// lesquels envoyer. La liste arrive du processus Rust par polling et/ou par
// événements, et la sélection est purement locale à l'interface.
import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { listen } from "@tauri-apps/api/event";
import type { Peer } from "../types";
import { invoke } from "../tauri";

export const usePeersStore = defineStore("peers", () => {
  const peers = ref<Peer[]>([]);
  const selectedIds = ref<Set<string>>(new Set());
  const discoveryError = ref("");

  // je ne renvoie que les appareils cochés par l'utilisateur
  const selectedPeers = computed(() =>
    peers.value.filter((p) => selectedIds.value.has(p.id)),
  );

  // je remplace la liste et je purge les sélections dont l'appareil a disparu.
  // Une liste vide est une fenêtre transitoire : start_discovery vide la liste
  // backend avant de la reconstruire, donc un poll tombant là purgerait toutes
  // les sélections à chaque refresh — je ne purge que si la liste a du contenu.
  function updatePeers(list: Peer[]) {
    const incoming = new Set(list.map((p) => p.id));
    if (incoming.size > 0) {
      for (const id of selectedIds.value) {
        if (!incoming.has(id)) selectedIds.value.delete(id);
      }
    }
    peers.value = list;
  }

  function toggleSelection(id: string) {
    const s = new Set(selectedIds.value);
    if (s.has(id)) s.delete(id);
    else s.add(id);
    selectedIds.value = s;
  }

  function selectAll(checked: boolean) {
    selectedIds.value = checked
      ? new Set(peers.value.map((p) => p.id))
      : new Set();
  }

  // je retire un appareil de la liste (pair manuel supprimé par l'utilisateur)
  // et je purge sa sélection éventuelle
  function removePeer(id: string) {
    peers.value = peers.value.filter((p) => p.id !== id);
    selectedIds.value.delete(id);
  }

  let pollTimer: ReturnType<typeof setInterval> | null = null;

  // j'interroge get_peers toutes les 2 secondes pour garder la liste à jour
  function startPolling() {
    stopPolling();
    pollTimer = setInterval(async () => {
      try {
        const list = await invoke<Peer[]>("get_peers");
        updatePeers(list);
      } catch (e) {
        console.error("Poll peers error:", e);
      }
    }, 2000);
  }

  function stopPolling() {
    if (pollTimer !== null) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  // je reçois une éventuelle erreur de découverte (port occupé, etc.)
  async function startListening() {
    await listen<string>("tool://discovery/error", (event) => {
      discoveryError.value = event.payload;
    });
  }

  return {
    peers,
    selectedIds,
    discoveryError,
    selectedPeers,
    updatePeers,
    toggleSelection,
    selectAll,
    removePeer,
    startPolling,
    stopPolling,
    startListening,
  };
});
