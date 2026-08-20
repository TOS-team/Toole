// je teste le store de découverte des appareils (peers.ts)
//
// le store interroge le processus Rust via invoke("get_peers") à intervalle
// régulier (2s) et garde une sélection locale d'appareils

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { calledCommands, mockReply, resetMock } from "../src/mocks/tauri-core";
import { emit, resetEmit } from "../src/mocks/tauri-event";
import { usePeersStore } from "../../../desktop-app/ui/src/stores/peers";

function freshStore() {
  setActivePinia(createPinia());
  return usePeersStore();
}

const A = { id: "pc-a", addr: "192.168.1.20" };
const B = { id: "pc-b", addr: "192.168.1.30" };

beforeEach(() => {
  resetMock();
  resetEmit();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("liste des appareils", () => {
  it("démarre vide", () => {
    const store = freshStore();
    expect(store.peers).toHaveLength(0);
    expect(store.selectedPeers).toHaveLength(0);
  });

  it("updatePeers remplace la liste", () => {
    const store = freshStore();
    store.updatePeers([A, B]);
    expect(store.peers).toHaveLength(2);
    store.updatePeers([A]);
    expect(store.peers.map((p) => p.id)).toEqual(["pc-a"]);
  });
});

describe("sélection", () => {
  it("toggleSelection coche puis décoche", () => {
    const store = freshStore();
    store.updatePeers([A, B]);

    store.toggleSelection("pc-a");
    expect(store.selectedIds.has("pc-a")).toBe(true);
    expect(store.selectedPeers.map((p) => p.id)).toEqual(["pc-a"]);

    store.toggleSelection("pc-b");
    expect(store.selectedPeers).toHaveLength(2);

    store.toggleSelection("pc-a");
    expect(store.selectedPeers.map((p) => p.id)).toEqual(["pc-b"]);
  });

  it("selectAll coche ou vide toute la liste", () => {
    const store = freshStore();
    store.updatePeers([A, B, { id: "pc-c", addr: "192.168.1.40" }]);

    store.selectAll(true);
    expect(store.selectedIds.size).toBe(3);

    store.selectAll(false);
    expect(store.selectedIds.size).toBe(0);
  });

  it("updatePeers purge les sélections disparues", () => {
    const store = freshStore();
    store.updatePeers([A, B]);
    store.toggleSelection("pc-a");
    store.toggleSelection("pc-b");

    store.updatePeers([A]); // pc-b n'est plus visible
    expect(store.selectedIds.has("pc-a")).toBe(true);
    expect(store.selectedIds.has("pc-b")).toBe(false);
    expect(store.selectedPeers).toHaveLength(1);
  });

  it("removePeer retire un pair manuel et purge sa sélection", () => {
    const store = freshStore();
    store.updatePeers([
      { id: "manual-192.168.1.42", addr: "192.168.1.42" },
      A,
    ]);
    store.toggleSelection("manual-192.168.1.42");
    expect(store.selectedPeers).toHaveLength(1);

    store.removePeer("manual-192.168.1.42");
    expect(store.peers.map((p) => p.id)).toEqual(["pc-a"]);
    expect(store.selectedIds.size).toBe(0);
  });
});

describe("polling get_peers", () => {
  it("interroge le rust toutes les 2 secondes et met à jour", async () => {
    vi.useFakeTimers();
    mockReply("get_peers", [A, B]);
    const store = freshStore();

    store.startPolling();
    await vi.advanceTimersByTimeAsync(2000);
    expect(calledCommands()).toEqual(["get_peers"]);
    expect(store.peers).toHaveLength(2);

    mockReply("get_peers", [A]);
    await vi.advanceTimersByTimeAsync(2000);
    expect(calledCommands()).toEqual(["get_peers", "get_peers"]);
    expect(store.peers).toHaveLength(1);

    await vi.advanceTimersByTimeAsync(4000);
    expect(calledCommands()).toEqual(["get_peers", "get_peers", "get_peers", "get_peers"]);
    store.stopPolling();
  });

  it("stopPolling arrête les appels", async () => {
    vi.useFakeTimers();
    mockReply("get_peers", []);
    const store = freshStore();

    store.startPolling();
    await vi.advanceTimersByTimeAsync(2000);
    store.stopPolling();
    const before = calledCommands().length;
    await vi.advanceTimersByTimeAsync(10_000);
    expect(calledCommands()).toHaveLength(before);
  });

  it("propage une erreur de découverte via listen", async () => {
    const store = freshStore();
    await store.startListening();

    emit("tool://discovery/error", "port déjà pris");
    expect(store.discoveryError).toBe("port déjà pris");
  });
});