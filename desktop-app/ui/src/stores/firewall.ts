// je détecte l'état du pare-feu système (ufw/firewalld sous Linux, règles
// Windows) pour avertir l'utilisateur quand les ports UDP de Toolé sont
// bloqués. Lecture seule : l'ouverture se fait à l'installation ou par
// l'utilisateur, les commandes sont affichées dans la bannière
import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "../tauri";

export interface FirewallStatus {
  os: string;
  active: boolean;
  ports_open: boolean;
  commands: string[];
}

export const useFirewallStore = defineStore("firewall", () => {
  const status = ref<FirewallStatus | null>(null);
  const checked = ref(false);

  // je vérifie une seule fois au démarrage : l'état du pare-feu ne change
  // pas en cours de session
  async function check() {
    if (checked.value) return;
    try {
      status.value = await invoke<FirewallStatus>("check_firewall");
    } catch (e) {
      console.error("check_firewall error:", e);
    } finally {
      checked.value = true;
    }
  }

  // pare-feu actif ET ports UDP non autorisés : il faut agir
  const needsAction = computed(
    () => !!status.value && status.value.active && !status.value.ports_open,
  );

  return { status, checked, check, needsAction };
});