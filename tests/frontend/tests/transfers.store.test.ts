// je teste le store d'historique des transferts (transfers.ts)
//
// deux angles :
//   - l'API directe (upsert, remove, clearHistory, activeCount) fractionnée
//     par un host MockUI
//   - le pipeline d'événements Tauri simulés (emit) jusqu'à la persistance
//     localStorage, pour vérifier que le store réagit aux bons événements

import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { emit, resetEmit } from "../src/mocks/tauri-event";
import { useTransfersStore } from "../../../desktop-app/ui/src/stores/transfers";

const KEY = "toole.transfers";

function freshStore() {
  setActivePinia(createPinia());
  return useTransfersStore();
}

beforeEach(() => {
  resetEmit();
  localStorage.clear();
});

describe("API directe du store", () => {
  it("démarre sans historique", () => {
    const store = freshStore();
    expect(store.transfers).toHaveLength(0);
    expect(store.activeCount).toBe(0);
  });

  it("upsert crée puis met à jour un transfert", () => {
    const store = freshStore();
    store.upsert("t1", { totalBytes: 1024 });
    expect(store.transfers).toHaveLength(1);
    expect(store.transfers[0]).toMatchObject({ id: "t1", status: "pending", totalBytes: 1024 });

    store.upsert("t1", { bytesSent: 512, status: "running" });
    expect(store.transfers).toHaveLength(1);
    expect(store.transfers[0]).toMatchObject({ bytesSent: 512, status: "running" });
  });

  it("activeCount ne compte que les transferts en cours", () => {
    const store = freshStore();
    store.upsert("a", { status: "running" });
    store.upsert("b", { status: "done" });
    store.upsert("c", { status: "running" });
    expect(store.activeCount).toBe(2);
    store.upsert("a", { status: "done" });
    expect(store.activeCount).toBe(1);
  });

  it("remove retire un transfert par id", () => {
    const store = freshStore();
    store.upsert("a", {});
    store.upsert("b", {});
    store.remove("a");
    expect(store.transfers.map((t) => t.id)).toEqual(["b"]);
  });

  it("clearHistory conserve uniquement pending/running", () => {
    const store = freshStore();
    store.upsert("done", { status: "done" });
    store.upsert("run", { status: "running" });
    store.upsert("err", { status: "error" });
    store.clearHistory();
    expect(store.transfers.map((t) => t.id)).toEqual(["run"]);
  });
});

describe("pipeline d'événements Tauri", () => {
  it("reagit à start puis progress", async () => {
    const store = freshStore();
    await store.startListening();

    emit("tool://transfer/start", "abc");
    expect(store.transfers.map((t) => t.id)).toContain("abc");
    expect(store.transfers[0].status).toBe("running");

    emit("tool://transfer/progress", {
      transfer_id: "abc",
      bytes_sent: 512,
      total_bytes: 2048,
      percent: 25,
    });
    expect(store.transfers[0]).toMatchObject({ bytesSent: 512, totalBytes: 2048, percent: 25, status: "running" });
  });

  it("fait avancer la progression par fichier", async () => {
    const store = freshStore();
    await store.startListening();
    emit("tool://transfer/start", "abc");

    emit("tool://transfer/file_progress", {
      transfer_id: "abc",
      file_name: "a.bin",
      file_bytes_sent: 100,
      file_total_bytes: 400,
      percent: 25,
    });
    emit("tool://transfer/file_progress", {
      transfer_id: "abc",
      file_name: "b.bin",
      file_bytes_sent: 200,
      file_total_bytes: 400,
      percent: 50,
    });
    const t = store.transfers[0];
    expect(t.fileProgress).toHaveLength(2);
    expect(t.fileProgress![1]).toMatchObject({ name: "b.bin", bytesSent: 200, percent: 50 });
  });

  it("termine un transfert sur done / cancel / error / received", async () => {
    const store = freshStore();
    await store.startListening();

    emit("tool://transfer/start", "t-done");
    emit("tool://transfer/done", "t-done");
    expect(store.transfers[0].status).toBe("done");
    expect(store.transfers[0].percent).toBe(100);

    emit("tool://transfer/start", "t-cancel");
    emit("tool://transfer/cancel", "t-cancel");
    expect(store.transfers[1].status).toBe("cancelled");

    emit("tool://transfer/start", "t-err");
    emit("tool://transfer/error", { transfer_id: "t-err", error: "connexion perdue" });
    expect(store.transfers[2].status).toBe("error");
    expect(store.transfers[2].error).toBe("connexion perdue");

    emit("tool://transfer/start", "t-rec");
    emit("tool://transfer/received", {
      transfer_id: "t-rec",
      peer: "pc-1",
      bytes: 4096,
      files: ["a.bin", "b.bin"],
    });
    expect(store.transfers[3].status).toBe("done");
    expect(store.transfers[3].peer).toBe("pc-1");
    expect(store.transfers[3].files).toEqual(["a.bin", "b.bin"]);
    expect(store.transfers[3].bytesSent).toBe(4096);
  });
});

describe("persistance localStorage", () => {
  it("ne persiste que les transferts terminés, en différé", async () => {
    vi.useFakeTimers();
    try {
      localStorage.clear();
      const store = freshStore();
      await store.startListening();

      // un transfert en cours ne doit pas être persisté
      emit("tool://transfer/start", "live");
      await vi.advanceTimersByTimeAsync(400);
      const during = localStorage.getItem(KEY);
      if (during) {
        // le watch peut avoir écrit une liste vide, mais jamais le transfert
        expect(JSON.parse(during)).not.toContainEqual(expect.objectContaining({ id: "live" }));
      }

      // terminé : il est persisté après le délai du debounce
      emit("tool://transfer/done", "live");
      await vi.advanceTimersByTimeAsync(400);
      const saved = JSON.parse(localStorage.getItem(KEY)!);
      expect(saved).toHaveLength(1);
      expect(saved[0]).toMatchObject({ id: "live", status: "done" });
    } finally {
      vi.useRealTimers();
    }
  });

  it("recharge l'historique persisté au démarrage", () => {
    localStorage.setItem(
      KEY,
      JSON.stringify([
        { id: "ancien", status: "done", percent: 100, bytesSent: 10, totalBytes: 10, speed: "Terminé", startTime: 1 },
      ]),
    );
    const store = freshStore();
    expect(store.transfers).toHaveLength(1);
    expect(store.transfers[0]).toMatchObject({ id: "ancien", status: "done", percent: 100 });
  });

  it("ignore un historique corrompu", () => {
    localStorage.setItem(KEY, "pas du json {");
    const store = freshStore();
    expect(store.transfers).toHaveLength(0);
  });
});