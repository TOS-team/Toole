// je teste le store de pare-feu (firewall.ts) : détection d'un pare-feu
// actif avec ports UDP fermés, sans appels réseau ni droits système

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { calledCommands, mockReply, resetMock } from "../src/mocks/tauri-core";
import { useFirewallStore } from "../../../desktop-app/ui/src/stores/firewall";

function freshStore() {
  setActivePinia(createPinia());
  return useFirewallStore();
}

const STATUS_BLOQUE: {
  os: string;
  active: boolean;
  ports_open: boolean;
  commands: string[];
} = {
  os: "linux",
  active: true,
  ports_open: false,
  commands: ["sudo ufw allow 58199/udp", "sudo ufw allow 58200/udp"],
};

const STATUS_OK: {
  os: string;
  active: boolean;
  ports_open: boolean;
  commands: string[];
} = {
  os: "linux",
  active: true,
  ports_open: true,
  commands: [],
};

beforeEach(() => {
  resetMock();
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("store firewall", () => {
  it("ne vérifie qu'une seule fois", async () => {
    mockReply("check_firewall", STATUS_OK);
    const store = freshStore();

    await store.check();
    await store.check();

    expect(calledCommands().filter((c) => c === "check_firewall")).toHaveLength(1);
  });

  it("signale quand le pare-feu bloque les ports", async () => {
    mockReply("check_firewall", STATUS_BLOQUE);
    const store = freshStore();

    await store.check();

    expect(store.status).toEqual(STATUS_BLOQUE);
    expect(store.needsAction).toBe(true);
  });

  it("ne signale rien quand les ports sont ouverts", async () => {
    mockReply("check_firewall", STATUS_OK);
    const store = freshStore();

    await store.check();

    expect(store.needsAction).toBe(false);
  });

  it("reste silencieux si la commande échoue", async () => {
    // je n'enregistre aucune réponse : invoke() rejette (commande inconnue)
    const store = freshStore();

    await store.check();

    expect(store.status).toBeNull();
    expect(store.needsAction).toBe(false);
  });
});