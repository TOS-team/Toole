import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Peer } from "../types";
import { invoke } from "../tauri";

export const usePeersStore = defineStore("peers", () => {
  const peers = ref<Peer[]>([]);
  const selectedHostnames = ref<Set<string>>(new Set());

  const selectedPeers = computed(() =>
    peers.value.filter((p) => selectedHostnames.value.has(p.hostname)),
  );

  function updatePeers(list: Peer[]) {
    const incoming = new Set(list.map((p) => p.hostname));
    for (const h of selectedHostnames.value) {
      if (!incoming.has(h)) selectedHostnames.value.delete(h);
    }
    peers.value = list;
  }

  function toggleSelection(hostname: string) {
    const s = new Set(selectedHostnames.value);
    if (s.has(hostname)) s.delete(hostname);
    else s.add(hostname);
    selectedHostnames.value = s;
  }

  function selectAll(checked: boolean) {
    selectedHostnames.value = checked
      ? new Set(peers.value.map((p) => p.hostname))
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

  return {
    peers,
    selectedHostnames,
    selectedPeers,
    updatePeers,
    toggleSelection,
    selectAll,
    startPolling,
    stopPolling,
  };
});
