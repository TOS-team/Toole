import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { listen } from "@tauri-apps/api/event";
import type { Peer } from "../types";
import { invoke } from "../tauri";

export const usePeersStore = defineStore("peers", () => {
  const peers = ref<Peer[]>([]);
  const selectedIds = ref<Set<string>>(new Set());
  const discoveryError = ref("");

  const selectedPeers = computed(() =>
    peers.value.filter((p) => selectedIds.value.has(p.id)),
  );

  function updatePeers(list: Peer[]) {
    const incoming = new Set(list.map((p) => p.id));
    for (const id of selectedIds.value) {
      if (!incoming.has(id)) selectedIds.value.delete(id);
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

  let pollTimer: ReturnType<typeof setInterval> | null = null;

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
    startPolling,
    stopPolling,
    startListening,
  };
});
